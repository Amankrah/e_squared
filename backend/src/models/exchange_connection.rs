use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "exchange_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub exchange_name: String,
    pub display_name: String,
    pub encrypted_api_key: String,      // Base64 encoded encrypted data
    pub encrypted_api_secret: String,   // Base64 encoded encrypted data
    pub encrypted_passphrase: Option<String>, // Base64 encoded encrypted passphrase (for exchanges like KuCoin, OKX)
    pub api_key_nonce: String,         // Base64 encoded nonce for API key
    pub api_secret_nonce: String,      // Base64 encoded nonce for API secret
    pub passphrase_nonce: Option<String>, // Base64 encoded nonce for passphrase
    pub api_key_salt: String,          // Base64 encoded salt for API key
    pub api_secret_salt: String,       // Base64 encoded salt for API secret
    pub passphrase_salt: Option<String>, // Base64 encoded salt for passphrase
    pub is_active: bool,
    pub last_sync: Option<ChronoDateTimeUtc>,
    pub connection_status: String,     // "connected", "error", "pending"
    pub last_error: Option<String>,
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
    #[sea_orm(has_many = "super::wallet_balance::Entity")]
    WalletBalances,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::wallet_balance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WalletBalances.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Request to create a new exchange connection
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateExchangeConnectionRequest {
    #[validate(length(min = 1, max = 50))]
    pub exchange_name: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
    #[validate(length(min = 1, max = 500))]
    pub api_key: String,
    #[validate(length(min = 1, max = 500))]
    pub api_secret: String,
    #[validate(length(min = 8))]
    pub password: String, // User's password for encryption
}

/// Request to update an exchange connection
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateExchangeConnectionRequest {
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub api_key: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub api_secret: Option<String>,
    #[validate(length(min = 8))]
    pub password: String, // User's password for encryption/decryption
}

/// Response for exchange connection
#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeConnectionResponse {
    pub id: Uuid,
    pub exchange_name: String,
    pub display_name: String,
    pub is_active: bool,
    pub last_sync: Option<ChronoDateTimeUtc>,
    pub connection_status: String,
    pub last_error: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    // Note: Never include encrypted credentials in responses
}

impl From<Model> for ExchangeConnectionResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            exchange_name: model.exchange_name,
            display_name: model.display_name,
            is_active: model.is_active,
            last_sync: model.last_sync,
            connection_status: model.connection_status,
            last_error: model.last_error,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// Re-export exchange connector types for convenience
#[allow(unused_imports)]
pub use crate::exchange_connectors::{Exchange as SupportedExchange, ExchangeCredentials};