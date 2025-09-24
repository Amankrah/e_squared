use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::strategies::implementations::dca::{DCAConfig, DCAStrategy as StrategyFrameworkDCA};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dca_strategies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String, // active, paused, completed
    pub config_json: String, // Store DCAConfig as JSON - this is the source of truth
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub last_execution_at: Option<DateTime<Utc>>,
    pub next_execution_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::models::user::Entity",
        from = "Column::UserId",
        to = "crate::models::user::Column::Id"
    )]
    User,
}

impl Related<crate::models::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

// DCA Execution Records
pub mod execution {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "dca_executions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub strategy_id: Uuid,
        pub exchange_connection_id: Uuid,
        pub execution_type: String, // buy, sell, skip
        pub trigger_reason: String, // scheduled, fear_extreme, zone_hit, manual
        pub amount_usd: Decimal,
        pub amount_asset: Option<Decimal>,
        pub price_at_execution: Option<Decimal>,
        pub fear_greed_index: Option<i32>,
        pub market_volatility: Option<Decimal>,
        pub order_id: Option<String>,
        pub order_status: String, // pending, filled, cancelled, failed
        pub execution_timestamp: DateTime<Utc>,
        pub error_message: Option<String>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::Entity",
            from = "Column::StrategyId",
            to = "super::Column::Id"
        )]
        Strategy,
        #[sea_orm(
            belongs_to = "crate::models::exchange_connection::Entity",
            from = "Column::ExchangeConnectionId",
            to = "crate::models::exchange_connection::Column::Id"
        )]
        ExchangeConnection,
    }

    impl Related<super::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Strategy.def()
        }
    }

    impl Related<crate::models::exchange_connection::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::ExchangeConnection.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Type aliases for easier access
pub type ExecutionEntity = execution::Entity;

// Market Data for DCA decisions
pub mod market_data {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "market_data")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub asset_symbol: String,
        pub price: Decimal,
        pub volume_24h: Option<Decimal>,
        pub market_cap: Option<Decimal>,
        pub fear_greed_index: Option<i32>,
        pub volatility_7d: Option<Decimal>,
        pub volatility_30d: Option<Decimal>,
        pub rsi_14: Option<Decimal>,
        pub ema_20: Option<Decimal>,
        pub ema_50: Option<Decimal>,
        pub ema_200: Option<Decimal>,
        pub support_level: Option<Decimal>,
        pub resistance_level: Option<Decimal>,
        pub trend_direction: Option<String>, // bullish, bearish, sideways
        pub timestamp: DateTime<Utc>,
        pub created_at: DateTime<Utc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// Type aliases for easier access
pub type MarketDataModel = market_data::Model;

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDCAStrategyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 20))]
    pub asset_symbol: String,

    // The DCAConfig contains all strategy configuration
    pub config: DCAConfig,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDCAStrategyRequest {
    pub name: Option<String>,
    pub status: Option<DCAStatus>,
    pub config: Option<DCAConfig>,
}

#[derive(Debug, Serialize)]
pub struct DCAStrategyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String,
    pub config: DCAConfig,
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub current_profit_loss: Option<Decimal>,
    pub profit_loss_percentage: Option<Decimal>,
    pub last_execution_at: Option<DateTime<Utc>>,
    pub next_execution_at: Option<DateTime<Utc>>,
    pub recent_executions: Vec<DCAExecutionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DCAExecutionResponse {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub execution_type: String,
    pub trigger_reason: String,
    pub amount_usd: Decimal,
    pub amount_asset: Option<Decimal>,
    pub price_at_execution: Option<Decimal>,
    pub fear_greed_index: Option<i32>,
    pub market_volatility: Option<Decimal>,
    pub order_status: String,
    pub execution_timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DCAStrategiesResponse {
    pub strategies: Vec<DCAStrategyResponse>,
    pub total_allocation: Decimal,
    pub total_invested: Decimal,
    pub total_profit_loss: Decimal,
    pub active_strategies: usize,
}

#[derive(Debug, Serialize)]
pub struct ExecutionStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_amount_invested: Decimal,
    pub average_execution_amount: Decimal,
    pub last_execution_timestamp: Option<DateTime<Utc>>,
}

// Enums - keeping only what's needed

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DCAStatus {
    Active,
    Paused,
    Completed,
    Error,
}

impl From<DCAStatus> for String {
    fn from(status: DCAStatus) -> Self {
        match status {
            DCAStatus::Active => "active".to_string(),
            DCAStatus::Paused => "paused".to_string(),
            DCAStatus::Completed => "completed".to_string(),
            DCAStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionType {
    Buy,
    Sell,
    Skip,
}

impl From<ExecutionType> for String {
    fn from(execution_type: ExecutionType) -> Self {
        match execution_type {
            ExecutionType::Buy => "buy".to_string(),
            ExecutionType::Sell => "sell".to_string(),
            ExecutionType::Skip => "skip".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerReason {
    Scheduled,
    FearExtreme,
    GreedExtreme,
    ZoneHit,
    VolatilitySpike,
    Manual,
}

impl From<TriggerReason> for String {
    fn from(reason: TriggerReason) -> Self {
        match reason {
            TriggerReason::Scheduled => "scheduled".to_string(),
            TriggerReason::FearExtreme => "fear_extreme".to_string(),
            TriggerReason::GreedExtreme => "greed_extreme".to_string(),
            TriggerReason::ZoneHit => "zone_hit".to_string(),
            TriggerReason::VolatilitySpike => "volatility_spike".to_string(),
            TriggerReason::Manual => "manual".to_string(),
        }
    }
}

// Implementation helpers
impl Model {
    /// Get the DCAConfig from stored JSON
    pub fn get_dca_config(&self) -> Result<DCAConfig, String> {
        serde_json::from_str::<DCAConfig>(&self.config_json)
            .map_err(|e| format!("Failed to parse DCAConfig JSON: {}", e))
    }

    /// Create a strategy framework instance from this model
    pub async fn to_strategy_framework(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<StrategyFrameworkDCA, String> {
        let config = self.get_dca_config()?;
        let config_json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to convert config to JSON: {}", e))?;

        let mut strategy = StrategyFrameworkDCA::new();

        use crate::strategies::core::{StrategyMode, StrategyContextBuilder};
        use rust_decimal::prelude::FromPrimitive;

        // Create context for strategy initialization
        let context = StrategyContextBuilder::new()
            .strategy_id(self.id)
            .user_id(self.user_id)
            .symbol(self.asset_symbol.clone())
            .interval("1h".to_string())
            .mode(StrategyMode::Live)
            .historical_data(historical_data)
            .current_price(Decimal::from_f64(50000.0).unwrap_or(Decimal::ZERO))
            .available_balance(config.base_amount)
            .build()
            .map_err(|e| format!("Failed to build context: {:?}", e))?;

        // Initialize the strategy
        strategy.initialize(&config_json, StrategyMode::Live, &context).await
            .map_err(|e| format!("Failed to initialize strategy: {:?}", e))?;

        Ok(strategy)
    }

    /// Calculate current tranche size using the strategy framework
    pub async fn calculate_current_tranche_size(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<Decimal, String> {
        let strategy = self.to_strategy_framework(historical_data).await?;

        // Use the strategy framework to determine amount
        let config = self.get_dca_config()?;
        Ok(config.base_amount) // Simple implementation - strategy framework handles complexity
    }

    /// Check if strategy should execute using strategy framework
    pub async fn should_execute(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<bool, String> {
        let strategy = self.to_strategy_framework(historical_data).await?;

        use crate::strategies::core::{StrategyContextBuilder, StrategyMode, Strategy};
        use rust_decimal::prelude::FromPrimitive;

        let context = StrategyContextBuilder::new()
            .strategy_id(self.id)
            .user_id(self.user_id)
            .symbol(self.asset_symbol.clone())
            .interval("1h".to_string())
            .mode(StrategyMode::Live)
            .historical_data(historical_data)
            .current_price(Decimal::from_f64(50000.0).unwrap_or(Decimal::ZERO))
            .available_balance(self.get_dca_config()?.base_amount)
            .build()
            .map_err(|e| format!("Failed to build context: {:?}", e))?;

        // Use strategy framework to analyze if we should execute
        match strategy.analyze(&context).await {
            Ok(signal) => Ok(signal.is_some()),
            Err(e) => Err(format!("Strategy analysis failed: {:?}", e))
        }
    }
}