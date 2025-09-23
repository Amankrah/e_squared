use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "wallet_balances")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub exchange_connection_id: Uuid,
    pub wallet_type: String,        // "spot", "margin", "isolated_margin", "futures_usdm", "futures_coinm", "earn"
    pub asset_symbol: String,       // "BTC", "ETH", "USDT", etc.
    pub free_balance: String,      // Available balance (stored as TEXT)
    pub locked_balance: String,    // Locked balance (stored as TEXT)
    pub total_balance: String,     // free + locked (stored as TEXT)
    pub usd_value: Option<String>, // USD equivalent value (stored as TEXT)
    pub last_updated: ChronoDateTimeUtc,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::exchange_connection::Entity",
        from = "Column::ExchangeConnectionId",
        to = "super::exchange_connection::Column::Id"
    )]
    ExchangeConnection,
}

impl Related<super::exchange_connection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExchangeConnection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Response for wallet balance
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBalanceResponse {
    pub id: Uuid,
    pub wallet_type: String,
    pub asset_symbol: String,
    pub free_balance: String,      // String to avoid precision loss in JSON
    pub locked_balance: String,    // String to avoid precision loss in JSON
    pub total_balance: String,     // String to avoid precision loss in JSON
    pub usd_value: Option<String>, // String to avoid precision loss in JSON
    pub last_updated: ChronoDateTimeUtc,
}

impl From<Model> for WalletBalanceResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            wallet_type: model.wallet_type,
            asset_symbol: model.asset_symbol,
            free_balance: model.free_balance,
            locked_balance: model.locked_balance,
            total_balance: model.total_balance,
            usd_value: model.usd_value,
            last_updated: model.last_updated,
        }
    }
}

/// Grouped wallet balances by wallet type
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletSummaryResponse {
    pub exchange_connection_id: Uuid,
    pub exchange_name: String,
    pub display_name: String,
    pub wallets: Vec<WalletTypeBalance>,
    pub total_usd_value: Option<String>,
    pub last_updated: ChronoDateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletTypeBalance {
    pub wallet_type: String,
    pub balances: Vec<WalletBalanceResponse>,
    pub wallet_usd_value: Option<String>,
}

/// Supported wallet types for Binance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalletType {
    Spot,
    Margin,
    IsolatedMargin,
    FuturesUsdm,
    FuturesCoinm,
    Earn,
    Options,
}

impl WalletType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WalletType::Spot => "spot",
            WalletType::Margin => "margin",
            WalletType::IsolatedMargin => "isolated_margin",
            WalletType::FuturesUsdm => "futures_usdm",
            WalletType::FuturesCoinm => "futures_coinm",
            WalletType::Earn => "earn",
            WalletType::Options => "options",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "spot" => Some(WalletType::Spot),
            "margin" => Some(WalletType::Margin),
            "isolated_margin" => Some(WalletType::IsolatedMargin),
            "futures_usdm" => Some(WalletType::FuturesUsdm),
            "futures_coinm" => Some(WalletType::FuturesCoinm),
            "earn" => Some(WalletType::Earn),
            "options" => Some(WalletType::Options),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            WalletType::Spot,
            WalletType::Margin,
            WalletType::IsolatedMargin,
            WalletType::FuturesUsdm,
            WalletType::FuturesCoinm,
            WalletType::Earn,
            WalletType::Options,
        ]
    }
}

/// Balance data from exchange API (before storing in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeBalanceData {
    pub wallet_type: WalletType,
    pub asset_symbol: String,
    pub free_balance: Decimal,
    pub locked_balance: Decimal,
}

impl ExchangeBalanceData {
    pub fn total_balance(&self) -> Decimal {
        self.free_balance + self.locked_balance
    }
}