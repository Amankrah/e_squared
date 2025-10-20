use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Supported blockchain networks for wallet connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockchainNetwork {
    Ethereum,
    BNBChain,
    Solana,
}

impl BlockchainNetwork {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" => Some(Self::Ethereum),
            "bnbchain" | "bnb" | "bsc" => Some(Self::BNBChain),
            "solana" | "sol" => Some(Self::Solana),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::BNBChain => "bnbchain",
            Self::Solana => "solana",
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wallet_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub blockchain_network: String, // "ethereum", "bnbchain", "solana"
    pub wallet_address: String,     // Public wallet address
    pub display_name: String,
    pub encrypted_private_key: String,    // Base64 encoded encrypted private key
    pub private_key_nonce: String,        // Base64 encoded nonce
    pub private_key_salt: String,         // Base64 encoded salt
    pub is_active: bool,
    pub last_used: Option<ChronoDateTimeUtc>,
    pub connection_status: String,        // "connected", "error", "pending"
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
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Request to create a new wallet connection
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateWalletConnectionRequest {
    #[validate(length(min = 1, max = 50))]
    pub blockchain_network: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: String,
    #[validate(length(min = 1, max = 500))]
    pub private_key: String, // User's wallet private key (will be encrypted)
    #[validate(length(min = 8))]
    pub password: String, // User's password for encryption
}

/// Request to update a wallet connection
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateWalletConnectionRequest {
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
    #[validate(length(min = 8))]
    pub password: String, // User's password for verification
}

/// Response for wallet connection
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletConnectionResponse {
    pub id: Uuid,
    pub blockchain_network: String,
    pub wallet_address: String,
    pub display_name: String,
    pub is_active: bool,
    pub last_used: Option<ChronoDateTimeUtc>,
    pub connection_status: String,
    pub last_error: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    // Note: Never include encrypted private key in responses
}

impl From<Model> for WalletConnectionResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            blockchain_network: model.blockchain_network,
            wallet_address: model.wallet_address,
            display_name: model.display_name,
            is_active: model.is_active,
            last_used: model.last_used,
            connection_status: model.connection_status,
            last_error: model.last_error,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
