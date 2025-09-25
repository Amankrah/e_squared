use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::strategies::implementations::sma_crossover::{SMACrossoverConfig, SMACrossoverStrategy as StrategyFrameworkSMACrossover};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sma_crossover_strategies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String, // active, paused, completed, error
    pub config_json: String, // Store SMACrossoverConfig as JSON
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub current_position: i32, // -1 = short, 0 = none, 1 = long
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Option<Decimal>,
    pub current_streak: i32, // positive for wins, negative for losses
    pub max_drawdown: Option<Decimal>,
    pub last_fast_sma: Option<Decimal>,
    pub last_slow_sma: Option<Decimal>,
    pub last_signal_type: Option<String>, // bullish_crossover, bearish_crossover
    pub last_signal_time: Option<DateTime<Utc>>,
    pub last_execution_at: Option<DateTime<Utc>>,
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

// SMA Crossover Execution Records
pub mod execution {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "sma_crossover_executions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub strategy_id: Uuid,
        pub exchange_connection_id: Uuid,
        pub execution_type: String, // buy, sell
        pub trigger_reason: String, // bullish_crossover, bearish_crossover, risk_management
        pub amount_usd: Decimal,
        pub amount_asset: Option<Decimal>,
        pub price_at_execution: Decimal,
        pub fast_sma_value: Decimal,
        pub slow_sma_value: Decimal,
        pub sma_spread: Decimal, // fast_sma - slow_sma
        pub signal_strength: String, // weak, medium, strong, very_strong
        pub crossover_type: Option<String>, // bullish_crossover, bearish_crossover
        pub position_before: i32,
        pub position_after: i32,
        pub realized_pnl: Option<Decimal>,
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

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSMACrossoverStrategyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 20))]
    pub asset_symbol: String,

    // The SMACrossoverConfig contains all strategy configuration
    pub config: SMACrossoverConfig,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSMACrossoverStrategyRequest {
    pub name: Option<String>,
    pub status: Option<SMACrossoverStatus>,
    pub config: Option<SMACrossoverConfig>,
}

#[derive(Debug, Serialize)]
pub struct SMACrossoverStrategyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String,
    pub config: SMACrossoverConfig,
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub current_position: i32,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Option<Decimal>,
    pub total_pnl: Option<Decimal>,
    pub current_streak: i32,
    pub max_drawdown: Option<Decimal>,
    pub last_fast_sma: Option<Decimal>,
    pub last_slow_sma: Option<Decimal>,
    pub sma_spread: Option<Decimal>, // calculated from fast - slow
    pub last_signal_type: Option<String>,
    pub last_signal_time: Option<DateTime<Utc>>,
    pub last_execution_at: Option<DateTime<Utc>>,
    pub recent_executions: Vec<SMACrossoverExecutionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SMACrossoverExecutionResponse {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub execution_type: String,
    pub trigger_reason: String,
    pub amount_usd: Decimal,
    pub amount_asset: Option<Decimal>,
    pub price_at_execution: Decimal,
    pub fast_sma_value: Decimal,
    pub slow_sma_value: Decimal,
    pub sma_spread: Decimal,
    pub signal_strength: String,
    pub crossover_type: Option<String>,
    pub position_before: i32,
    pub position_after: i32,
    pub realized_pnl: Option<Decimal>,
    pub order_status: String,
    pub execution_timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SMACrossoverStrategiesResponse {
    pub strategies: Vec<SMACrossoverStrategyResponse>,
    pub total_invested: Decimal,
    pub total_pnl: Decimal,
    pub active_strategies: usize,
    pub average_win_rate: Decimal,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SMACrossoverStatus {
    Active,
    Paused,
    Completed,
    Error,
}

impl From<SMACrossoverStatus> for String {
    fn from(status: SMACrossoverStatus) -> Self {
        match status {
            SMACrossoverStatus::Active => "active".to_string(),
            SMACrossoverStatus::Paused => "paused".to_string(),
            SMACrossoverStatus::Completed => "completed".to_string(),
            SMACrossoverStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SMACrossoverExecutionType {
    Buy,
    Sell,
}

impl From<SMACrossoverExecutionType> for String {
    fn from(execution_type: SMACrossoverExecutionType) -> Self {
        match execution_type {
            SMACrossoverExecutionType::Buy => "buy".to_string(),
            SMACrossoverExecutionType::Sell => "sell".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SMACrossoverTriggerReason {
    BullishCrossover,
    BearishCrossover,
    RiskManagement,
    StopLoss,
    TakeProfit,
    Manual,
}

impl From<SMACrossoverTriggerReason> for String {
    fn from(reason: SMACrossoverTriggerReason) -> Self {
        match reason {
            SMACrossoverTriggerReason::BullishCrossover => "bullish_crossover".to_string(),
            SMACrossoverTriggerReason::BearishCrossover => "bearish_crossover".to_string(),
            SMACrossoverTriggerReason::RiskManagement => "risk_management".to_string(),
            SMACrossoverTriggerReason::StopLoss => "stop_loss".to_string(),
            SMACrossoverTriggerReason::TakeProfit => "take_profit".to_string(),
            SMACrossoverTriggerReason::Manual => "manual".to_string(),
        }
    }
}

// Implementation helpers
impl Model {
    /// Get the SMACrossoverConfig from stored JSON
    pub fn get_sma_crossover_config(&self) -> Result<SMACrossoverConfig, String> {
        serde_json::from_str::<SMACrossoverConfig>(&self.config_json)
            .map_err(|e| format!("Failed to parse SMACrossoverConfig JSON: {}", e))
    }

    /// Calculate win rate
    pub fn calculate_win_rate(&self) -> Decimal {
        if self.total_trades > 0 {
            Decimal::from(self.winning_trades) / Decimal::from(self.total_trades) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// Calculate total P&L (realized + unrealized)
    pub fn calculate_total_pnl(&self) -> Option<Decimal> {
        match self.unrealized_pnl {
            Some(unrealized) => Some(self.realized_pnl + unrealized),
            None => Some(self.realized_pnl),
        }
    }

    /// Calculate SMA spread (fast - slow)
    pub fn calculate_sma_spread(&self) -> Option<Decimal> {
        match (self.last_fast_sma, self.last_slow_sma) {
            (Some(fast), Some(slow)) => Some(fast - slow),
            _ => None,
        }
    }

    /// Create a strategy framework instance from this model
    pub async fn to_strategy_framework(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<StrategyFrameworkSMACrossover, String> {
        let config = self.get_sma_crossover_config()?;
        let config_json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to convert config to JSON: {}", e))?;

        let mut strategy = StrategyFrameworkSMACrossover::new();

        use crate::strategies::core::{Strategy, StrategyMode, StrategyContextBuilder};
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
            .available_balance(Decimal::from(1000)) // Default balance
            .build()
            .map_err(|e| format!("Failed to build context: {:?}", e))?;

        // Initialize the strategy
        strategy.initialize(&config_json, StrategyMode::Live, &context).await
            .map_err(|e| format!("Failed to initialize strategy: {:?}", e))?;

        Ok(strategy)
    }

    /// Check if strategy should execute using strategy framework
    pub async fn should_execute(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<bool, String> {
        let historical_data_clone = historical_data.clone();
        let mut strategy = self.to_strategy_framework(historical_data_clone).await?;

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
            .available_balance(Decimal::from(1000))
            .build()
            .map_err(|e| format!("Failed to build context: {:?}", e))?;

        // Use strategy framework to analyze if we should execute
        match strategy.analyze(&context).await {
            Ok(signal) => Ok(signal.is_some()),
            Err(e) => Err(format!("Strategy analysis failed: {:?}", e))
        }
    }
}