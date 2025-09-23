use actix_web::{web, HttpResponse, Result, cookie::{Cookie, SameSite}};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use tracing;

use crate::models::{
    user::{self, ActiveModel as UserActiveModel, Entity as UserEntity},
    ChangePasswordRequest, CreateUserRequest, LoginRequest, UserResponse,
};
use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthService {
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub fn generate_token(&self, user_id: Uuid, email: &str) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::TokenCreation)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::InvalidToken)
    }
}



pub async fn signup(
    db: web::Data<DatabaseConnection>,
    auth_service: web::Data<AuthService>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate().map_err(AppError::ValidationError)?;

    // Check if user already exists
    let existing_user = UserEntity::find()
        .filter(user::Column::Email.eq(&req.email))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_user.is_some() {
        return Err(AppError::EmailAlreadyExists);
    }

    // Hash password
    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|_| AppError::PasswordHashError)?;

    // Create user
    let new_user = UserActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(req.email.clone()),
        password_hash: Set(password_hash),
        is_active: Set(true),
        is_verified: Set(false),
        totp_secret: Set(None),
        totp_enabled: Set(false),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    tracing::info!("About to insert user into database");
    
    // Store the values before inserting since we might not get them back
    let user_id = new_user.id.clone().unwrap();
    let user_email = new_user.email.clone().unwrap();
    let user_password_hash = new_user.password_hash.clone().unwrap();
    let user_is_active = new_user.is_active.clone().unwrap();
    let user_is_verified = new_user.is_verified.clone().unwrap();
    let user_totp_secret = new_user.totp_secret.clone().unwrap();
    let user_totp_enabled = new_user.totp_enabled.clone().unwrap();
    let user_created_at = new_user.created_at.clone().unwrap();
    let user_updated_at = new_user.updated_at.clone().unwrap();
    
    let insert_result = new_user.insert(db.get_ref()).await;
    
    match &insert_result {
        Ok(user) => tracing::info!("Database insert successful, got user with ID: {}", user.id),
        Err(e) => {
            tracing::error!("Database insert failed: {:?}", e);
            // If it's UnpackInsertId, the insert actually succeeded, just construct the user manually
            let error_debug = format!("{:?}", e);
            if error_debug.contains("UnpackInsertId") {
                tracing::info!("Insert succeeded but couldn't unpack ID, constructing user manually");
            } else {
                tracing::error!("Actual database error, not UnpackInsertId: {}", error_debug);
                return Err(AppError::InternalServerError)
            }
        }
    }
    
    // Try to get the user from insert result, or construct manually if UnpackInsertId
    let user = match insert_result {
        Ok(user) => user,
        Err(_) => {
            // Construct user manually from our stored values
            user::Model {
                id: user_id,
                email: user_email,
                password_hash: user_password_hash,
                is_active: user_is_active,
                is_verified: user_is_verified,
                totp_secret: user_totp_secret,
                totp_enabled: user_totp_enabled,
                created_at: user_created_at,
                updated_at: user_updated_at,
            }
        }
    };

    tracing::info!("User created successfully with ID: {}", user.id);

    // Generate JWT token
    let token = auth_service.generate_token(user.id, &user.email)
        .map_err(|e| {
            tracing::error!("Failed to generate JWT token for user {}: {:?}", user.id, e);
            e
        })?;

    tracing::info!("JWT token generated successfully for user: {}", user.id);

    // Return response with user data and token
    let response = serde_json::json!({
        "user": UserResponse::from(user.clone()),
        "token": token
    });

    tracing::info!("Response JSON created successfully for user: {}", user.id);

    let cookie = Cookie::build("auth_token", token.clone())
        .path("/")
        .max_age(actix_web::cookie::time::Duration::days(7))
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .same_site(SameSite::Lax)
        .finish();

    tracing::info!("Cookie created successfully for user: {}", user.id);

    let http_response = HttpResponse::Created()
        .cookie(cookie)
        .json(response);

    tracing::info!("HTTP response created successfully for user: {}", user.id);

    Ok(http_response)
}

pub async fn login(
    db: web::Data<DatabaseConnection>,
    auth_service: web::Data<AuthService>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // Validate request
    req.validate().map_err(AppError::ValidationError)?;

    // Find user by email
    let user = UserEntity::find()
        .filter(user::Column::Email.eq(&req.email))
        .one(db.get_ref())
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

    // Return response with user data and token
    let response = serde_json::json!({
        "user": UserResponse::from(user),
        "token": token
    });

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", token)
                .path("/")
                .max_age(actix_web::cookie::time::Duration::days(7))
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(SameSite::Lax)
                .finish()
        )
        .json(response))
}

pub async fn get_csrf_token() -> Result<HttpResponse, AppError> {
    // Generate a simple CSRF token (in production, this should be more sophisticated)
    let token = Uuid::new_v4().to_string();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "csrf_token": token
    })))
}

pub async fn logout() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .http_only(true)
                .secure(false)
                .same_site(SameSite::Lax)
                .finish()
        )
        .json(serde_json::json!({
            "message": "Logged out successfully"
        })))
}

pub async fn change_password(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
    req: web::Json<ChangePasswordRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user = UserEntity::find_by_id(user_id.into_inner())
        .one(db.get_ref())
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

    user_active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

pub async fn get_current_user(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user = UserEntity::find_by_id(user_id.into_inner())
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    let response = UserResponse::from(user);
    Ok(HttpResponse::Ok().json(response))
}


pub async fn get_current_user_optional(
    req: actix_web::HttpRequest,
    db: web::Data<DatabaseConnection>,
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
                    // Token is valid, get the user
                    let user_id = uuid::Uuid::parse_str(&claims.sub)
                        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

                    let user = UserEntity::find_by_id(user_id)
                        .one(db.get_ref())
                        .await
                        .map_err(AppError::DatabaseError)?;

                    match user {
                        Some(user) => {
                            let response = UserResponse::from(user);
                            Ok(HttpResponse::Ok().json(serde_json::json!({
                                "authenticated": true,
                                "user": response
                            })))
                        }
                        None => {
                            // User not found, but token was valid - this shouldn't happen
                            Ok(HttpResponse::Ok().json(serde_json::json!({
                                "authenticated": false,
                                "user": null
                            })))
                        }
                    }
                }
                Err(_) => {
                    // Token is invalid
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "authenticated": false,
                        "user": null
                    })))
                }
            }
        }
        None => {
            // No token found
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "authenticated": false,
                "user": null
            })))
        }
    }
}