use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{entity::prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dca_strategies")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub total_allocation: Decimal,
    pub base_tranche_size: Decimal,
    pub status: String, // active, paused, completed
    pub strategy_type: String, // adaptive_zone, classic, aggressive
    pub sentiment_multiplier: bool,
    pub volatility_adjustment: bool,
    pub fear_greed_threshold_buy: i32,
    pub fear_greed_threshold_sell: i32,
    pub max_tranche_percentage: Decimal,
    pub min_tranche_percentage: Decimal,
    pub dca_interval_hours: i32,
    pub target_zones: Option<String>, // JSON array of price zones
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
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

    pub total_allocation: Decimal,

    pub base_tranche_percentage: Decimal,

    pub strategy_type: DCAStrategyType,
    pub sentiment_multiplier: bool,
    pub volatility_adjustment: bool,

    pub fear_greed_threshold_buy: i32,

    pub fear_greed_threshold_sell: i32,

    pub dca_interval_hours: i32,

    pub target_zones: Option<Vec<Decimal>>,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDCAStrategyRequest {
    pub name: Option<String>,
    pub total_allocation: Option<Decimal>,
    pub base_tranche_percentage: Option<Decimal>,
    pub status: Option<DCAStatus>,
    pub sentiment_multiplier: Option<bool>,
    pub volatility_adjustment: Option<bool>,
    pub fear_greed_threshold_buy: Option<i32>,
    pub fear_greed_threshold_sell: Option<i32>,
    pub dca_interval_hours: Option<i32>,
    pub target_zones: Option<Vec<Decimal>>,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct DCAStrategyResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub asset_symbol: String,
    pub total_allocation: Decimal,
    pub base_tranche_size: Decimal,
    pub status: String,
    pub strategy_type: String,
    pub sentiment_multiplier: bool,
    pub volatility_adjustment: bool,
    pub fear_greed_threshold_buy: i32,
    pub fear_greed_threshold_sell: i32,
    pub max_tranche_percentage: Decimal,
    pub min_tranche_percentage: Decimal,
    pub dca_interval_hours: i32,
    pub target_zones: Option<Vec<Decimal>>,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
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

// Enums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DCAStrategyType {
    AdaptiveZone,  // Main strategy from the description
    Classic,       // Traditional fixed DCA
    Aggressive,    // High frequency, high risk
}

impl From<DCAStrategyType> for String {
    fn from(strategy_type: DCAStrategyType) -> Self {
        match strategy_type {
            DCAStrategyType::AdaptiveZone => "adaptive_zone".to_string(),
            DCAStrategyType::Classic => "classic".to_string(),
            DCAStrategyType::Aggressive => "aggressive".to_string(),
        }
    }
}

impl std::str::FromStr for DCAStrategyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "adaptive_zone" => Ok(DCAStrategyType::AdaptiveZone),
            "classic" => Ok(DCAStrategyType::Classic),
            "aggressive" => Ok(DCAStrategyType::Aggressive),
            _ => Err(format!("Invalid strategy type: {}", s)),
        }
    }
}

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
    pub fn calculate_current_tranche_size(&self, market_data: &market_data::Model) -> Result<Decimal, String> {
        // Validate percentage bounds
        if self.max_tranche_percentage <= Decimal::ZERO || self.min_tranche_percentage <= Decimal::ZERO {
            return Err("Invalid tranche percentage: must be greater than zero".to_string());
        }

        if self.max_tranche_percentage > Decimal::from(100) || self.min_tranche_percentage > Decimal::from(100) {
            return Err("Invalid tranche percentage: cannot exceed 100%".to_string());
        }

        if self.min_tranche_percentage > self.max_tranche_percentage {
            return Err("Invalid tranche percentage: min cannot exceed max".to_string());
        }

        let mut tranche_size = self.base_tranche_size;

        // Apply sentiment multiplier
        if self.sentiment_multiplier {
            if let Some(fear_greed) = market_data.fear_greed_index {
                let multiplier = if fear_greed < 25 {
                    Decimal::from(2) // Double the size in extreme fear
                } else if fear_greed > 75 {
                    Decimal::try_from(0.5).map_err(|_| "Failed to convert multiplier".to_string())? // Half the size in extreme greed
                } else {
                    Decimal::from(1)
                };
                tranche_size *= multiplier;
            }
        }

        // Apply volatility adjustment
        if self.volatility_adjustment {
            if let Some(volatility) = market_data.volatility_7d {
                if volatility > Decimal::from(30) {
                    // Reduce size during high volatility
                    let volatility_multiplier = Decimal::try_from(0.7)
                        .map_err(|_| "Failed to convert volatility multiplier".to_string())?;
                    tranche_size *= volatility_multiplier;
                }
            }
        }

        // Ensure within bounds
        let max_tranche = self.total_allocation * self.max_tranche_percentage / Decimal::from(100);
        let min_tranche = self.total_allocation * self.min_tranche_percentage / Decimal::from(100);

        Ok(tranche_size.max(min_tranche).min(max_tranche))
    }

    pub fn should_execute_buy(&self, market_data: &market_data::Model, current_price: Decimal) -> bool {
        // Check if we have allocation left
        if self.total_invested >= self.total_allocation {
            return false;
        }

        // Sentiment-based execution
        if let Some(fear_greed) = market_data.fear_greed_index {
            if fear_greed <= self.fear_greed_threshold_buy {
                return true; // Execute in fear
            }
            if fear_greed >= self.fear_greed_threshold_sell {
                return false; // Don't buy in greed
            }
        }

        // Zone-based execution
        if let Some(zones_str) = &self.target_zones {
            if let Ok(zones) = serde_json::from_str::<Vec<Decimal>>(zones_str) {
                for zone in zones {
                    if let Ok(tolerance_multiplier) = Decimal::try_from(1.02) {
                        if current_price <= zone * tolerance_multiplier { // 2% tolerance
                            return true;
                        }
                    }
                }
            }
        }

        // Default scheduled execution
        true
    }

    pub fn should_execute_sell(&self, market_data: &market_data::Model, current_price: Decimal) -> bool {
        // Only sell if we have positions
        if self.total_purchased <= Decimal::ZERO {
            return false;
        }

        // Take profit check
        if let (Some(avg_price), Some(take_profit)) = (self.average_buy_price, self.take_profit_percentage) {
            let target_price = avg_price * (Decimal::from(100) + take_profit) / Decimal::from(100);
            if current_price >= target_price {
                return true;
            }
        }

        // Greed-based partial selling
        if let Some(fear_greed) = market_data.fear_greed_index {
            if fear_greed >= self.fear_greed_threshold_sell {
                return true;
            }
        }

        false
    }
}