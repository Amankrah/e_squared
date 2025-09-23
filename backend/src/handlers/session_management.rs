use actix_web::{web, HttpResponse, Result, HttpRequest, HttpMessage};
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user_session::{self, Entity as UserSessionEntity, UserSessionResponse};
use crate::utils::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<UserSessionResponse>,
}

pub async fn get_active_sessions(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get current session token from Authorization header or session
    let current_token = extract_current_session_token(&req);

    // Query all active sessions for the user
    let sessions = UserSessionEntity::find()
        .filter(user_session::Column::UserId.eq(user_id))
        .filter(user_session::Column::ExpiresAt.gt(Utc::now()))
        .order_by_desc(user_session::Column::LastActivity)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let session_responses: Vec<UserSessionResponse> = sessions
        .into_iter()
        .map(|mut session| {
            // Set is_current based on token comparison
            session.is_current = current_token
                .as_ref()
                .map(|token| token == &session.session_token)
                .unwrap_or(false);

            UserSessionResponse::from(session)
        })
        .collect();

    Ok(HttpResponse::Ok().json(SessionsResponse {
        sessions: session_responses,
    }))
}

pub async fn revoke_session(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let session_id_str = path.into_inner();
    let session_id = Uuid::parse_str(&session_id_str)
        .map_err(|_| AppError::BadRequest("Invalid session ID format".to_string()))?;

    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Find the session
    let session = UserSessionEntity::find()
        .filter(user_session::Column::Id.eq(session_id))
        .filter(user_session::Column::UserId.eq(user_id))
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    let session = session.ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

    // Check if this is the current session
    let current_token = extract_current_session_token(&req);
    let is_current_session = current_token
        .as_ref()
        .map(|token| token == &session.session_token)
        .unwrap_or(false);

    if is_current_session {
        return Err(AppError::BadRequest("Cannot revoke current session".to_string()));
    }

    // Delete the session
    UserSessionEntity::delete_by_id(session_id)
        .exec(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Session revoked successfully"
    })))
}

pub async fn revoke_all_sessions(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get current session token to preserve it
    let current_token = extract_current_session_token(&req);

    if let Some(token) = current_token {
        // Delete all sessions except the current one
        UserSessionEntity::delete_many()
            .filter(user_session::Column::UserId.eq(user_id))
            .filter(user_session::Column::SessionToken.ne(token))
            .exec(db.as_ref())
            .await
            .map_err(|e| AppError::DatabaseError(e))?;
    } else {
        // If no current token, delete all sessions
        UserSessionEntity::delete_many()
            .filter(user_session::Column::UserId.eq(user_id))
            .exec(db.as_ref())
            .await
            .map_err(|e| AppError::DatabaseError(e))?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All other sessions revoked successfully"
    })))
}

fn extract_current_session_token(req: &HttpRequest) -> Option<String> {
    // Try to get token from Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // Could also check session store if using session-based auth
    None
}