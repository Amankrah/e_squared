use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub totp_secret: Option<String>,
    pub totp_enabled: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::user_profile::Entity")]
    UserProfile,
}

impl Related<super::user_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Setup2FARequest {
    #[validate(length(equal = 6, message = "Code must be 6 digits"))]
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Verify2FARequest {
    #[validate(length(equal = 6, message = "Code must be 6 digits"))]
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setup2FAResponse {
    pub secret: String,
    pub qr_code: String,
    pub manual_entry_key: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Disable2FARequest {
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub totp_enabled: bool,
    pub created_at: ChronoDateTimeUtc,
}

impl From<Model> for UserResponse {
    fn from(user: Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            is_active: user.is_active,
            is_verified: user.is_verified,
            totp_enabled: user.totp_enabled,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}