use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(unique)]
    pub session_token: String,
    pub device_info: String,
    pub ip_address: String,
    pub location: Option<String>,
    pub user_agent: String,
    pub is_current: bool,
    pub last_activity: ChronoDateTimeUtc,
    pub created_at: ChronoDateTimeUtc,
    pub expires_at: ChronoDateTimeUtc,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionResponse {
    pub id: Uuid,
    pub device_info: String,
    pub ip_address: String,
    pub location: Option<String>,
    pub user_agent: String,
    pub is_current: bool,
    pub last_activity: ChronoDateTimeUtc,
    pub created_at: ChronoDateTimeUtc,
    pub platform: String,
    pub browser: String,
}

impl From<Model> for UserSessionResponse {
    fn from(session: Model) -> Self {
        // Extract platform and browser from user_agent
        let (platform, browser) = parse_user_agent(&session.user_agent);

        Self {
            id: session.id,
            device_info: session.device_info,
            ip_address: session.ip_address,
            location: session.location,
            user_agent: session.user_agent,
            is_current: session.is_current,
            last_activity: session.last_activity,
            created_at: session.created_at,
            platform,
            browser,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RevokeSessionRequest {
    pub session_id: Uuid,
}

fn parse_user_agent(user_agent: &str) -> (String, String) {
    let ua = user_agent.to_lowercase();

    // Simple platform detection
    let platform = if ua.contains("windows") {
        "Windows"
    } else if ua.contains("macintosh") || ua.contains("mac os") {
        "macOS"
    } else if ua.contains("linux") {
        "Linux"
    } else if ua.contains("android") {
        "Android"
    } else if ua.contains("iphone") || ua.contains("ipad") {
        "iOS"
    } else {
        "Unknown"
    }.to_string();

    // Simple browser detection
    let browser = if ua.contains("chrome") && !ua.contains("edge") {
        "Chrome"
    } else if ua.contains("firefox") {
        "Firefox"
    } else if ua.contains("safari") && !ua.contains("chrome") {
        "Safari"
    } else if ua.contains("edge") {
        "Edge"
    } else if ua.contains("opera") {
        "Opera"
    } else {
        "Unknown"
    }.to_string();

    (platform, browser)
}