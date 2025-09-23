use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub join_date: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserProfileRequest {
    pub name: String,
    #[validate(email)]
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserProfileRequest {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub join_date: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

impl From<Model> for UserProfileResponse {
    fn from(profile: Model) -> Self {
        Self {
            id: profile.id,
            user_id: profile.user_id,
            name: profile.name,
            email: profile.email,
            phone: profile.phone,
            location: profile.location,
            bio: profile.bio,
            join_date: profile.join_date,
            avatar_url: profile.avatar_url,
            is_verified: profile.is_verified,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}