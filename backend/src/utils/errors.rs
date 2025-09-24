use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use std::fmt;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(DbErr),
    ValidationError(ValidationErrors),
    EmailAlreadyExists,
    InvalidCredentials,
    UserNotFound,
    AccountDeactivated,
    PasswordHashError,
    PasswordVerificationError,
    TokenCreation,
    InvalidToken,
    MissingToken,
    ProfileNotFound,
    InternalServerError,
    Unauthorized(String),
    BadRequest(String),
    NotFound(String),
    EncryptionError(String),
    DecryptionError(String),
    Forbidden(String),
    ExternalServiceError(String),
    RateLimitError(String),
    Banned(String),
    ParseError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::ValidationError(err) => write!(f, "Validation error: {}", err),
            AppError::EmailAlreadyExists => write!(f, "Email already exists"),
            AppError::InvalidCredentials => write!(f, "Invalid credentials"),
            AppError::UserNotFound => write!(f, "User not found"),
            AppError::AccountDeactivated => write!(f, "Account is deactivated"),
            AppError::PasswordHashError => write!(f, "Failed to hash password"),
            AppError::PasswordVerificationError => write!(f, "Failed to verify password"),
            AppError::TokenCreation => write!(f, "Failed to create token"),
            AppError::InvalidToken => write!(f, "Invalid or expired token"),
            AppError::MissingToken => write!(f, "Missing authentication token"),
            AppError::ProfileNotFound => write!(f, "Profile not found"),
            AppError::InternalServerError => write!(f, "Internal server error"),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            AppError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            AppError::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
            AppError::Banned(msg) => write!(f, "Banned: {}", msg),
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(_) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Database operation failed"
                }))
            }
            AppError::ValidationError(err) => {
                let error_messages: Vec<String> = err
                    .field_errors()
                    .iter()
                    .flat_map(|(field, errors)| {
                        errors.iter().map(move |error| {
                            format!("{}: {}", field, error.message.as_ref().unwrap_or(&"Invalid value".into()))
                        })
                    })
                    .collect();

                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Validation failed",
                    "message": error_messages.join(", ")
                }))
            }
            AppError::EmailAlreadyExists => {
                HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Email already exists",
                    "message": "An account with this email already exists"
                }))
            }
            AppError::InvalidCredentials => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid credentials",
                    "message": "Email or password is incorrect"
                }))
            }
            AppError::UserNotFound => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found",
                    "message": "The requested user does not exist"
                }))
            }
            AppError::AccountDeactivated => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Account deactivated",
                    "message": "Your account has been deactivated"
                }))
            }
            AppError::PasswordHashError | AppError::PasswordVerificationError => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Password processing failed"
                }))
            }
            AppError::TokenCreation => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create authentication token"
                }))
            }
            AppError::InvalidToken => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Invalid token",
                    "message": "The provided token is invalid or expired"
                }))
            }
            AppError::MissingToken => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Missing token",
                    "message": "Authentication token is required"
                }))
            }
            AppError::ProfileNotFound => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Profile not found",
                    "message": "The requested profile does not exist"
                }))
            }
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "An unexpected error occurred"
                }))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": msg
                }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Bad request",
                    "message": msg
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Not found",
                    "message": msg
                }))
            }
            AppError::EncryptionError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Encryption failed",
                    "message": msg
                }))
            }
            AppError::DecryptionError(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Decryption failed",
                    "message": msg
                }))
            }
            AppError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Forbidden",
                    "message": msg
                }))
            }
            AppError::ExternalServiceError(msg) => {
                HttpResponse::BadGateway().json(serde_json::json!({
                    "error": "External service error",
                    "message": msg
                }))
            }
            AppError::RateLimitError(msg) => {
                HttpResponse::TooManyRequests().json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "message": msg
                }))
            }
            AppError::Banned(msg) => {
                HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Banned",
                    "message": msg
                }))
            }
            AppError::ParseError(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Parse error",
                    "message": msg
                }))
            }
        }
    }
}