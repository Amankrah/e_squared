use actix_web::{web, HttpResponse, Result, cookie::{Cookie, SameSite}, HttpRequest};
use actix_session::Session;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use once_cell::sync::Lazy;
use base64;
use rand;

use crate::models::user::{
    self, ActiveModel as UserActiveModel, Entity as UserEntity, Model,
    ChangePasswordRequest, CreateUserRequest, LoginRequest, UserResponse,
};
use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
}

#[derive(Clone)]
pub struct AuthService {
    pub jwt_secret: String,
}

// Rate limiting storage - in production, use Redis
static RATE_LIMIT_STORAGE: Lazy<std::sync::Mutex<HashMap<String, (usize, std::time::Instant)>>> =
    Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

// Password strength validation - using manual checks since Rust regex doesn't support lookahead
static LOWERCASE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-z]").unwrap()
});
static UPPERCASE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[A-Z]").unwrap()
});
static DIGIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d").unwrap()
});
static SPECIAL_CHAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[@$!%*?&_]").unwrap()
});
static VALID_PASSWORD_CHARS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z\d@$!%*?&_]{12,}$").unwrap()
});

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn generate_token(&self, user_id: Uuid, email: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = now
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;
        let issued_at = now.timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration,
            iat: issued_at,
            iss: "e_squared_platform".to_string(),
            aud: "e_squared_users".to_string(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());

        encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|e| {
            error!("Token generation failed: {:?}", e);
            AppError::TokenCreation
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["e_squared_platform"]);
        validation.set_audience(&["e_squared_users"]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 60; // 60 seconds leeway for clock skew

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| {
            warn!("Token verification failed: {:?}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => AppError::InvalidToken,
                jsonwebtoken::errors::ErrorKind::InvalidAudience => AppError::InvalidToken,
                _ => AppError::InvalidToken,
            }
        })
    }
}

// Security helper functions
fn sanitize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_email_format(email: &str) -> Result<(), ValidationError> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::new("invalid_email"));
    }
    Ok(())
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Check minimum length
    if password.len() < 12 {
        return Err(ValidationError::new("password_too_short"));
    }
    
    // Check that password only contains allowed characters
    if !VALID_PASSWORD_CHARS_REGEX.is_match(password) {
        return Err(ValidationError::new("password_invalid_chars"));
    }
    
    // Check for required character types
    if !LOWERCASE_REGEX.is_match(password) {
        return Err(ValidationError::new("password_missing_lowercase"));
    }
    
    if !UPPERCASE_REGEX.is_match(password) {
        return Err(ValidationError::new("password_missing_uppercase"));
    }
    
    if !DIGIT_REGEX.is_match(password) {
        return Err(ValidationError::new("password_missing_digit"));
    }
    
    if !SPECIAL_CHAR_REGEX.is_match(password) {
        return Err(ValidationError::new("password_missing_special"));
    }
    
    Ok(())
}

fn check_rate_limit(ip: &str, max_attempts: usize, window_minutes: u64) -> Result<(), AppError> {
    let mut storage = RATE_LIMIT_STORAGE.lock().unwrap();
    let now = std::time::Instant::now();
    let window = std::time::Duration::from_secs(window_minutes * 60);

    // Clean up old entries
    storage.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < window);

    // Check current IP
    match storage.get_mut(ip) {
        Some((count, timestamp)) => {
            if now.duration_since(*timestamp) < window {
                if *count >= max_attempts {
                    return Err(AppError::RateLimitError(format!(
                        "Too many attempts. Try again in {} minutes",
                        window_minutes
                    )));
                }
                *count += 1;
            } else {
                *count = 1;
                *timestamp = now;
            }
        }
        None => {
            storage.insert(ip.to_string(), (1, now));
        }
    }

    Ok(())
}

fn get_client_ip(req: &HttpRequest) -> String {
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_string()
}

pub async fn signup(
    db: web::Data<Arc<DatabaseConnection>>,
    auth_service: web::Data<AuthService>,
    http_req: HttpRequest,
    session: Session,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting
    let client_ip = get_client_ip(&http_req);
    check_rate_limit(&client_ip, 5, 15)?; // 5 attempts per 15 minutes

    // Validate request
    req.validate().map_err(AppError::ValidationError)?;

    // Additional security validations
    let sanitized_email = sanitize_email(&req.email);
    validate_email_format(&sanitized_email)
        .map_err(|_| AppError::BadRequest("Invalid email format".to_string()))?;
    validate_password_strength(&req.password)
        .map_err(|_| AppError::BadRequest(
            "Password must be at least 12 characters with uppercase, lowercase, number, and special character (@$!%*?&_)".to_string()
        ))?;

    // Check if user already exists using sanitized email
    let existing_user = UserEntity::find()
        .filter(user::Column::Email.eq(&sanitized_email))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_user.is_some() {
        return Err(AppError::EmailAlreadyExists);
    }

    // Hash password
    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|_| AppError::PasswordHashError)?;

    // Create user with sanitized email
    let user_id = Uuid::new_v4();
    let new_user = UserActiveModel {
        id: Set(user_id),
        email: Set(sanitized_email.clone()),
        password_hash: Set(password_hash),
        is_active: Set(true),
        is_verified: Set(false),
        totp_secret: Set(None),
        totp_enabled: Set(false),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    info!("Creating new user account for email: {}", sanitized_email);

    // Use insert_one which doesn't expect auto-generated ID
    UserEntity::insert(new_user)
        .exec_without_returning(db.as_ref().as_ref())
        .await
        .map_err(|e| {
            error!("Database insert failed for user {}: {:?}", user_id, e);
            AppError::DatabaseError(e)
        })?;

    // Fetch the created user
    let user = UserEntity::find_by_id(user_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::InternalServerError)?;

    info!("User created successfully with ID: {}", user.id);

    // Generate JWT token
    let token = auth_service.generate_token(user.id, &user.email)
        .map_err(|e| {
            error!("Failed to generate JWT token for user {}: {:?}", user.id, e);
            e
        })?;

    // Store session information
    session.insert("user_id", user.id.to_string())
        .map_err(|_| AppError::InternalServerError)?;
    session.insert("authenticated", true)
        .map_err(|_| AppError::InternalServerError)?;

    info!("JWT token generated successfully for user: {}", user.id);

    // Return response with user data and token
    let response = serde_json::json!({
        "user": UserResponse::from(user.clone()),
        "token": token,
        "message": "Account created successfully"
    });

    info!("User registration completed for: {}", user.id);

    let cookie = Cookie::build("auth_token", token.clone())
        .path("/")
        .max_age(actix_web::cookie::time::Duration::hours(24))
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .same_site(SameSite::Strict)
        .finish();

    Ok(HttpResponse::Created()
        .cookie(cookie)
        .json(response))
}

pub async fn login(
    db: web::Data<Arc<DatabaseConnection>>,
    auth_service: web::Data<AuthService>,
    http_req: HttpRequest,
    session: Session,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting - stricter for login attempts
    let client_ip = get_client_ip(&http_req);
    check_rate_limit(&client_ip, 5, 10)?; // 5 attempts per 10 minutes

    // Validate request
    req.validate().map_err(AppError::ValidationError)?;

    // Sanitize email
    let sanitized_email = sanitize_email(&req.email);

    // Find user by email using sanitized email
    let user = UserEntity::find()
        .filter(user::Column::Email.eq(&sanitized_email))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::InvalidCredentials)?;

    // Check if user is active
    if !user.is_active {
        return Err(AppError::AccountDeactivated);
    }

    // Verify password
    let is_valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::PasswordVerificationError)?;

    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    // Generate JWT token
    let token = auth_service.generate_token(user.id, &user.email)?;

    // Store session information
    session.insert("user_id", user.id.to_string())
        .map_err(|_| AppError::InternalServerError)?;
    session.insert("authenticated", true)
        .map_err(|_| AppError::InternalServerError)?;

    info!("User login successful: {}", user.id);

    // Return response with user data and token
    let response = serde_json::json!({
        "user": UserResponse::from(user),
        "token": token,
        "message": "Login successful"
    });

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", token)
                .path("/")
                .max_age(actix_web::cookie::time::Duration::hours(24))
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(response))
}

pub async fn get_csrf_token(session: Session) -> Result<HttpResponse, AppError> {
    // Generate a cryptographically secure CSRF token
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    use base64::{Engine as _, engine::general_purpose};
    let csrf_token = general_purpose::STANDARD.encode(random_bytes);

    // Store CSRF token in session for validation
    session.insert("csrf_token", csrf_token.clone())
        .map_err(|_| AppError::InternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "csrf_token": csrf_token
    })))
}

pub async fn logout(
    session: Session,
    user_id: Option<web::ReqData<Uuid>>,
) -> Result<HttpResponse, AppError> {
    if let Some(user_id) = user_id {
        info!("User logout requested for: {}", user_id.into_inner());
    } else {
        info!("Logout requested without authentication");
    }

    // Clear session data
    session.clear();

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(serde_json::json!({
            "message": "Logged out successfully"
        })))
}

pub async fn change_password(
    db: web::Data<Arc<DatabaseConnection>>,
    user_id: web::ReqData<Uuid>,
    http_req: HttpRequest,
    session: Session,
    req: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    // Rate limiting for password changes
    let client_ip = get_client_ip(&http_req);
    check_rate_limit(&client_ip, 3, 30)?; // 3 attempts per 30 minutes

    // Validate request
    req.validate().map_err(AppError::ValidationError)?;

    // Validate new password strength
    validate_password_strength(&req.new_password)
        .map_err(|_| AppError::BadRequest(
            "New password must be at least 12 characters with uppercase, lowercase, number, and special character (@$!%*?&_)".to_string()
        ))?;

    let user_id_value = user_id.into_inner();
    let user = UserEntity::find_by_id(user_id_value)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    let is_valid = verify(&req.current_password, &user.password_hash)
        .map_err(|_| AppError::PasswordVerificationError)?;

    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    let new_password_hash = hash(&req.new_password, DEFAULT_COST)
        .map_err(|_| AppError::PasswordHashError)?;

    let mut user_active_model: UserActiveModel = user.into();
    user_active_model.password_hash = Set(new_password_hash);
    user_active_model.updated_at = Set(Utc::now());

    user_active_model.update(db.as_ref().as_ref()).await
        .map_err(AppError::DatabaseError)?;

    info!("Password changed successfully for user: {}", user_id_value);

    // Clear current session to force re-authentication with new password
    session.clear();

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Strict)
                .finish()
        )
        .json(serde_json::json!({
            "message": "Password changed successfully. Please log in again."
        })))
}

pub async fn get_current_user(
    db: web::Data<Arc<DatabaseConnection>>,
    user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id_value = user_id.into_inner();
    let user = UserEntity::find_by_id(user_id_value)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    let response = UserResponse::from(user);
    Ok(HttpResponse::Ok().json(response))
}


pub async fn get_current_user_optional(
    req: actix_web::HttpRequest,
    db: web::Data<Arc<DatabaseConnection>>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    // Try to get the token from cookies
    let token = req
        .cookie("auth_token")
        .and_then(|cookie| Some(cookie.value().to_string()));

    match token {
        Some(token_value) => {
            // Try to verify the token
            match auth_service.verify_token(&token_value) {
                Ok(claims) => {
                    // Token is valid, try to get the user
                    match uuid::Uuid::parse_str(&claims.sub) {
                        Ok(user_id) => {
                            // Try to get user from database
                            match UserEntity::find_by_id(user_id).one(db.as_ref().as_ref()).await {
                                Ok(Some(user)) => {
                                    let response = UserResponse::from(user);
                                    Ok(HttpResponse::Ok().json(serde_json::json!({
                                        "authenticated": true,
                                        "user": response
                                    })))
                                }
                                Ok(None) => {
                                    // User not found, but token was valid - treat as unauthenticated
                                    tracing::warn!("Token valid but user not found: {}", user_id);
                                    Ok(HttpResponse::Ok().json(serde_json::json!({
                                        "authenticated": false,
                                        "user": null
                                    })))
                                }
                                Err(db_error) => {
                                    // Database error - log it but don't expose details to client
                                    tracing::error!("Database error during auth check: {:?}", db_error);
                                    Ok(HttpResponse::Ok().json(serde_json::json!({
                                        "authenticated": false,
                                        "user": null
                                    })))
                                }
                            }
                        }
                        Err(parse_error) => {
                            // Invalid UUID in token - treat as unauthenticated
                            tracing::warn!("Invalid UUID in token: {:?}", parse_error);
                            Ok(HttpResponse::Ok().json(serde_json::json!({
                                "authenticated": false,
                                "user": null
                            })))
                        }
                    }
                }
                Err(token_error) => {
                    // Token is invalid or expired - this is expected, don't log as error
                    tracing::debug!("Token validation failed: {:?}", token_error);
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "authenticated": false,
                        "user": null
                    })))
                }
            }
        }
        None => {
            // No token found - this is expected for unauthenticated requests
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "authenticated": false,
                "user": null
            })))
        }
    }
}