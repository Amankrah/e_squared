use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::strategies::implementations::grid_trading::{GridTradingConfig, GridTradingStrategy as StrategyFrameworkGridTrading};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "grid_trading_strategies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String, // active, paused, completed, error
    pub config_json: String, // Store GridTradingConfig as JSON
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub current_inventory: Decimal, // Current inventory (positive = long, negative = short)
    pub grid_levels_count: i32, // Number of active grid levels
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub grid_center_price: Option<Decimal>,
    pub grid_upper_bound: Option<Decimal>,
    pub grid_lower_bound: Option<Decimal>,
    pub last_rebalance_at: Option<DateTime<Utc>>,
    pub total_grid_profit: Decimal, // Profit from grid trades
    pub active_buy_orders: i32,
    pub active_sell_orders: i32,
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

// Grid Trading Execution Records
pub mod execution {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "grid_trading_executions")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub strategy_id: Uuid,
        pub exchange_connection_id: Uuid,
        pub execution_type: String, // buy, sell
        pub trigger_reason: String, // grid_level_hit, rebalance, take_profit, stop_loss, manual
        pub amount_usd: Decimal,
        pub amount_asset: Decimal,
        pub price_at_execution: Decimal,
        pub grid_level_index: i32, // Which grid level was hit
        pub grid_level_price: Decimal, // Price of the grid level
        pub inventory_before: Decimal,
        pub inventory_after: Decimal,
        pub grid_profit: Option<Decimal>, // Profit from completing a grid cycle
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
pub struct CreateGridTradingStrategyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(length(min = 1, max = 20))]
    pub asset_symbol: String,

    // The GridTradingConfig contains all strategy configuration
    pub config: GridTradingConfig,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateGridTradingStrategyRequest {
    pub name: Option<String>,
    pub status: Option<GridTradingStatus>,
    pub config: Option<GridTradingConfig>,
}

#[derive(Debug, Serialize)]
pub struct GridTradingStrategyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub status: String,
    pub config: GridTradingConfig,
    pub total_invested: Decimal,
    pub total_purchased: Decimal,
    pub average_buy_price: Option<Decimal>,
    pub current_inventory: Decimal,
    pub grid_levels_count: i32,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Option<Decimal>,
    pub total_pnl: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub grid_center_price: Option<Decimal>,
    pub grid_upper_bound: Option<Decimal>,
    pub grid_lower_bound: Option<Decimal>,
    pub grid_spread: Option<Decimal>, // upper_bound - lower_bound
    pub last_rebalance_at: Option<DateTime<Utc>>,
    pub total_grid_profit: Decimal,
    pub active_buy_orders: i32,
    pub active_sell_orders: i32,
    pub grid_utilization: Decimal, // percentage of grid levels that have been hit
    pub inventory_utilization: Decimal, // percentage of max inventory used
    pub last_execution_at: Option<DateTime<Utc>>,
    pub recent_executions: Vec<GridTradingExecutionResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct GridTradingExecutionResponse {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub execution_type: String,
    pub trigger_reason: String,
    pub amount_usd: Decimal,
    pub amount_asset: Decimal,
    pub price_at_execution: Decimal,
    pub grid_level_index: i32,
    pub grid_level_price: Decimal,
    pub inventory_before: Decimal,
    pub inventory_after: Decimal,
    pub grid_profit: Option<Decimal>,
    pub order_status: String,
    pub execution_timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GridTradingStrategiesResponse {
    pub strategies: Vec<GridTradingStrategyResponse>,
    pub total_invested: Decimal,
    pub total_pnl: Decimal,
    pub active_strategies: usize,
    pub average_win_rate: Decimal,
    pub total_grid_profit: Decimal,
}

// Enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridTradingStatus {
    Active,
    Paused,
    Completed,
    Error,
    Rebalancing,
}

impl From<GridTradingStatus> for String {
    fn from(status: GridTradingStatus) -> Self {
        match status {
            GridTradingStatus::Active => "active".to_string(),
            GridTradingStatus::Paused => "paused".to_string(),
            GridTradingStatus::Completed => "completed".to_string(),
            GridTradingStatus::Error => "error".to_string(),
            GridTradingStatus::Rebalancing => "rebalancing".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GridTradingExecutionType {
    Buy,
    Sell,
}

impl From<GridTradingExecutionType> for String {
    fn from(execution_type: GridTradingExecutionType) -> Self {
        match execution_type {
            GridTradingExecutionType::Buy => "buy".to_string(),
            GridTradingExecutionType::Sell => "sell".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridTradingTriggerReason {
    GridLevelHit,
    Rebalance,
    TakeProfit,
    StopLoss,
    RiskManagement,
    Manual,
    InventoryAdjustment,
}

impl From<GridTradingTriggerReason> for String {
    fn from(reason: GridTradingTriggerReason) -> Self {
        match reason {
            GridTradingTriggerReason::GridLevelHit => "grid_level_hit".to_string(),
            GridTradingTriggerReason::Rebalance => "rebalance".to_string(),
            GridTradingTriggerReason::TakeProfit => "take_profit".to_string(),
            GridTradingTriggerReason::StopLoss => "stop_loss".to_string(),
            GridTradingTriggerReason::RiskManagement => "risk_management".to_string(),
            GridTradingTriggerReason::Manual => "manual".to_string(),
            GridTradingTriggerReason::InventoryAdjustment => "inventory_adjustment".to_string(),
        }
    }
}

// Implementation helpers
impl Model {
    /// Get the GridTradingConfig from stored JSON
    pub fn get_grid_trading_config(&self) -> Result<GridTradingConfig, String> {
        serde_json::from_str::<GridTradingConfig>(&self.config_json)
            .map_err(|e| format!("Failed to parse GridTradingConfig JSON: {}", e))
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

    /// Calculate grid spread (upper_bound - lower_bound)
    pub fn calculate_grid_spread(&self) -> Option<Decimal> {
        match (self.grid_upper_bound, self.grid_lower_bound) {
            (Some(upper), Some(lower)) => Some(upper - lower),
            _ => None,
        }
    }

    /// Calculate grid utilization (percentage of levels hit)
    pub fn calculate_grid_utilization(&self) -> Decimal {
        if self.grid_levels_count > 0 && self.total_trades > 0 {
            // Estimate grid utilization based on trades vs levels
            let utilization = Decimal::from(self.total_trades) / Decimal::from(self.grid_levels_count * 2); // 2 for buy/sell pairs
            utilization.min(Decimal::from(100)) // Cap at 100%
        } else {
            Decimal::ZERO
        }
    }

    /// Calculate inventory utilization
    pub fn calculate_inventory_utilization(&self) -> Decimal {
        match self.get_grid_trading_config() {
            Ok(config) => {
                if config.risk_settings.max_inventory > Decimal::ZERO {
                    (self.current_inventory.abs() / config.risk_settings.max_inventory * Decimal::from(100))
                        .min(Decimal::from(100))
                } else {
                    Decimal::ZERO
                }
            }
            Err(_) => Decimal::ZERO,
        }
    }

    /// Create a strategy framework instance from this model
    pub async fn to_strategy_framework(&self, historical_data: Vec<crate::exchange_connectors::Kline>) -> Result<StrategyFrameworkGridTrading, String> {
        let config = self.get_grid_trading_config()?;
        let config_json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to convert config to JSON: {}", e))?;

        let mut strategy = StrategyFrameworkGridTrading::new();

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
            .available_balance(config.total_investment)
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

        let config = self.get_grid_trading_config()?;

        let context = StrategyContextBuilder::new()
            .strategy_id(self.id)
            .user_id(self.user_id)
            .symbol(self.asset_symbol.clone())
            .interval("1h".to_string())
            .mode(StrategyMode::Live)
            .historical_data(historical_data)
            .current_price(Decimal::from_f64(50000.0).unwrap_or(Decimal::ZERO))
            .available_balance(config.total_investment)
            .build()
            .map_err(|e| format!("Failed to build context: {:?}", e))?;

        // Use strategy framework to analyze if we should execute
        match strategy.analyze(&context).await {
            Ok(signal) => Ok(signal.is_some()),
            Err(e) => Err(format!("Strategy analysis failed: {:?}", e))
        }
    }
}