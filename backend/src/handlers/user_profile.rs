use actix_web::{web, HttpResponse, Result, HttpRequest};
use actix_session::{Session, SessionExt};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::models::{
    user_profile::{
        self, ActiveModel as UserProfileActiveModel, Entity as UserProfileEntity,
    },
    CreateUserProfileRequest, UpdateUserProfileRequest, UserProfileResponse,
};
use crate::utils::errors::AppError;

/// Extract authenticated user ID from session
fn get_user_id_from_session(req: &HttpRequest) -> Result<Uuid, AppError> {
    let session = req.get_session();

    if let Ok(Some(user_id_str)) = session.get::<String>("user_id") {
        if let Ok(Some(authenticated)) = session.get::<bool>("authenticated") {
            if authenticated {
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    return Ok(user_id);
                }
            }
        }
    }

    Err(AppError::Unauthorized("Authentication required".to_string()))
}

pub async fn create_profile(
    db: web::Data<Arc<DatabaseConnection>>,
    http_req: HttpRequest,
    req: web::Json<CreateUserProfileRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user_id = get_user_id_from_session(&http_req)?;

    let existing_profile = UserProfileEntity::find()
        .filter(user_profile::Column::UserId.eq(user_id))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_profile.is_some() {
        return Err(AppError::BadRequest(
            "Profile already exists for this user".to_string()
        ));
    }

    let join_date = Utc::now().format("%B %Y").to_string();
    let profile_id = Uuid::new_v4();

    let new_profile = UserProfileActiveModel {
        id: Set(profile_id),
        user_id: Set(user_id),
        name: Set(req.name.clone()),
        email: Set(req.email.clone()),
        phone: Set(req.phone.clone()),
        location: Set(req.location.clone()),
        bio: Set(req.bio.clone()),
        join_date: Set(join_date),
        avatar_url: Set(req.avatar_url.clone()),
        is_verified: Set(false),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    // Insert without returning (SQLite doesn't support returning the inserted row easily)
    UserProfileEntity::insert(new_profile)
        .exec_without_returning(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Fetch the created profile
    let profile = UserProfileEntity::find_by_id(profile_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::InternalServerError)?;

    let response = UserProfileResponse::from(profile);

    Ok(HttpResponse::Created().json(response))
}

pub async fn get_profile(
    db: web::Data<Arc<DatabaseConnection>>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&http_req)?;

    let profile = UserProfileEntity::find()
        .filter(user_profile::Column::UserId.eq(user_id))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    match profile {
        Some(profile) => {
            let response = UserProfileResponse::from(profile);
            Ok(HttpResponse::Ok().json(response))
        }
        None => {
            Err(AppError::NotFound("Profile not found".to_string()))
        }
    }
}

pub async fn get_profile_by_id(
    db: web::Data<Arc<DatabaseConnection>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let profile_id = path.into_inner();

    let profile = UserProfileEntity::find_by_id(profile_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::ProfileNotFound)?;

    // All profiles are viewable in this simplified version

    let response = UserProfileResponse::from(profile);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn update_profile(
    db: web::Data<Arc<DatabaseConnection>>,
    http_req: HttpRequest,
    req: web::Json<UpdateUserProfileRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate().map_err(AppError::ValidationError)?;

    let user_id = get_user_id_from_session(&http_req)?;

    let profile = UserProfileEntity::find()
        .filter(user_profile::Column::UserId.eq(user_id))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::ProfileNotFound)?;

    let mut profile_active_model: UserProfileActiveModel = profile.into();

    if let Some(name) = &req.name {
        profile_active_model.name = Set(name.clone());
    }
    if let Some(email) = &req.email {
        profile_active_model.email = Set(email.clone());
    }
    if req.phone.is_some() {
        profile_active_model.phone = Set(req.phone.clone());
    }
    if req.location.is_some() {
        profile_active_model.location = Set(req.location.clone());
    }
    if req.bio.is_some() {
        profile_active_model.bio = Set(req.bio.clone());
    }
    if req.avatar_url.is_some() {
        profile_active_model.avatar_url = Set(req.avatar_url.clone());
    }

    profile_active_model.updated_at = Set(Utc::now());

    let updated_profile = profile_active_model
        .update(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let response = UserProfileResponse::from(updated_profile);
    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_profile(
    db: web::Data<Arc<DatabaseConnection>>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&http_req)?;

    let profile = UserProfileEntity::find()
        .filter(user_profile::Column::UserId.eq(user_id))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::ProfileNotFound)?;

    UserProfileEntity::delete_by_id(profile.id)
        .exec(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Profile deleted successfully"
    })))
}