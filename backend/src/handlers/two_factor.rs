use actix_web::{web, HttpResponse, Result};
use bcrypt::verify;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use totp_rs::{Algorithm, TOTP, Secret};
use qrcode::{QrCode, render::svg};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    user::{ActiveModel as UserActiveModel, Entity as UserEntity},
    Setup2FARequest, Verify2FARequest, Setup2FAResponse, Disable2FARequest,
};
use crate::utils::errors::AppError;

pub async fn setup_2fa(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = user_id.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    if user.totp_enabled {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    // Generate a new secret (32 random bytes)
    use rand::RngCore;
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    let secret = Secret::Raw(secret_bytes.to_vec());
    let secret_str = secret.to_encoded().to_string();

    // Create TOTP instance
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret.to_bytes().unwrap(),
        Some("E²Trading".to_string()),
        user.email.clone(),
    ).map_err(|_| AppError::InternalServerError)?;

    // Generate QR code
    let qr_code_url = totp.get_url();
    let code = QrCode::new(&qr_code_url).map_err(|_| AppError::InternalServerError)?;
    let svg_string = code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();

    // Encode QR code as base64
    let qr_code_base64 = general_purpose::STANDARD.encode(svg_string.as_bytes());

    // Store the secret temporarily (not enabled yet)
    let mut user_active_model: UserActiveModel = user.into();
    user_active_model.totp_secret = Set(Some(secret_str.clone()));
    user_active_model.updated_at = Set(Utc::now());

    user_active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    let response = Setup2FAResponse {
        secret: secret_str,
        qr_code: format!("data:image/svg+xml;base64,{}", qr_code_base64),
        manual_entry_key: secret.to_encoded().to_string(),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn verify_2fa_setup(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
    req: web::Json<Setup2FARequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user_id = user_id.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    let secret = user.totp_secret.clone().ok_or(AppError::ValidationError(
        validator::ValidationErrors::new()
    ))?;

    // Verify the TOTP code
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret.clone()).to_bytes().unwrap(),
        Some("E²Trading".to_string()),
        user.email.clone(),
    ).map_err(|_| AppError::InternalServerError)?;

    if !totp.check_current(&req.code).map_err(|_| AppError::InvalidCredentials)? {
        return Err(AppError::InvalidCredentials);
    }

    // Enable 2FA
    let mut user_active_model: UserActiveModel = user.into();
    user_active_model.totp_enabled = Set(true);
    user_active_model.updated_at = Set(Utc::now());

    user_active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Two-factor authentication enabled successfully"
    })))
}

pub async fn verify_2fa(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
    req: web::Json<Verify2FARequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user_id = user_id.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    if !user.totp_enabled {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    let secret = user.totp_secret.clone().ok_or(AppError::InternalServerError)?;

    // Verify the TOTP code
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(secret).to_bytes().unwrap(),
        Some("E²Trading".to_string()),
        user.email,
    ).map_err(|_| AppError::InternalServerError)?;

    if !totp.check_current(&req.code).map_err(|_| AppError::InvalidCredentials)? {
        return Err(AppError::InvalidCredentials);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Code verified successfully"
    })))
}

pub async fn disable_2fa(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
    req: web::Json<Disable2FARequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user_id = user_id.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    if !user.totp_enabled {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    // Verify password before disabling 2FA
    let is_valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::PasswordVerificationError)?;

    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    // Disable 2FA and remove secret
    let mut user_active_model: UserActiveModel = user.into();
    user_active_model.totp_enabled = Set(false);
    user_active_model.totp_secret = Set(None);
    user_active_model.updated_at = Set(Utc::now());

    user_active_model.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Two-factor authentication disabled successfully"
    })))
}

pub async fn get_2fa_status(
    db: web::Data<DatabaseConnection>,
    user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = user_id.into_inner();

    let user = UserEntity::find_by_id(user_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::UserNotFound)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "enabled": user.totp_enabled,
        "has_secret": user.totp_secret.is_some()
    })))
}