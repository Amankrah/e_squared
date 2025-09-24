use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::strategies::implementations::rsi_strategy::{RSIStrategyConfig, RSIStrategy as StrategyFrameworkRSI};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rsi_strategies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String, // active, paused, completed, error
    pub config_json: String, // Store RSIStrategyConfig as JSON
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
    pub last_rsi_value: Option<Decimal>,
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

// RSI Execution Records
pub mod execution {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "rsi_executions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub strategy_id: Uuid,
        pub exchange_connection_id: Uuid,
        pub execution_type: String, // buy, sell
        pub trigger_reason: String, // oversold, overbought, risk_management
        pub amount_usd: Decimal,
        pub amount_asset: Option<Decimal>,
        pub price_at_execution: Decimal,
        pub rsi_value: Decimal,
        pub signal_strength: String, // weak, medium, strong, very_strong
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
pub struct CreateRSIStrategyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 20))]
    pub asset_symbol: String,

    // The RSIStrategyConfig contains all strategy configuration
    pub config: RSIStrategyConfig,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRSIStrategyRequest {
    pub name: Option<String>,
    pub status: Option<RSIStatus>,
    pub config: Option<RSIStrategyConfig>,
}

#[derive(Debug, Serialize)]
pub struct RSIStrategyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String,
    pub config: RSIStrategyConfig,
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
    pub last_rsi_value: Option<Decimal>,
    pub last_signal_time: Option<DateTime<Utc>>,
    pub last_execution_at: Option<DateTime<Utc>>,
    pub recent_executions: Vec<RSIExecutionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RSIExecutionResponse {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub execution_type: String,
    pub trigger_reason: String,
    pub amount_usd: Decimal,
    pub amount_asset: Option<Decimal>,
    pub price_at_execution: Decimal,
    pub rsi_value: Decimal,
    pub signal_strength: String,
    pub position_before: i32,
    pub position_after: i32,
    pub realized_pnl: Option<Decimal>,
    pub order_status: String,
    pub execution_timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RSIStrategiesResponse {
    pub strategies: Vec<RSIStrategyResponse>,
    pub total_invested: Decimal,
    pub total_pnl: Decimal,
    pub active_strategies: usize,
    pub average_win_rate: Decimal,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RSIStatus {
    Active,
    Paused,
    Completed,
    Error,
}

impl From<RSIStatus> for String {
    fn from(status: RSIStatus) -> Self {
        match status {
            RSIStatus::Active => "active".to_string(),
            RSIStatus::Paused => "paused".to_string(),
            RSIStatus::Completed => "completed".to_string(),
            RSIStatus::Error => "error".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RSIExecutionType {
    Buy,
    Sell,
}

impl From<RSIExecutionType> for String {
    fn from(execution_type: RSIExecutionType) -> Self {
        match execution_type {
            RSIExecutionType::Buy => "buy".to_string(),
            RSIExecutionType::Sell => "sell".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RSITriggerReason {
    Oversold,
    Overbought,
    RiskManagement,
    StopLoss,
    TakeProfit,
    Manual,
}

impl From<RSITriggerReason> for String {
    fn from(reason: RSITriggerReason) -> Self {
        match reason {
            RSITriggerReason::Oversold => "oversold".to_string(),
            RSITriggerReason::Overbought => "overbought".to_string(),
            RSITriggerReason::RiskManagement => "risk_management".to_string(),
            RSITriggerReason::StopLoss => "stop_loss".to_string(),
            RSITriggerReason::TakeProfit => "take_profit".to_string(),
            RSITriggerReason::Manual => "manual".to_string(),
        }
    }
}

// Implementation helpers
impl Model {
    /// Get the RSIStrategyConfig from stored JSON
    pub fn get_rsi_config(&self) -> Result<RSIStrategyConfig, String> {
        serde_json::from_str::<RSIStrategyConfig>(&self.config_json)
            .map_err(|e| format!("Failed to parse RSIStrategyConfig JSON: {}", e))
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

    /// Create a strategy framework instance from this model
    pub async fn to_strategy_framework(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<StrategyFrameworkRSI, String> {
        let config = self.get_rsi_config()?;
        let config_json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to convert config to JSON: {}", e))?;

        let mut strategy = StrategyFrameworkRSI::new();

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