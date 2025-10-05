use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "backtest_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub strategy_name: String,
    pub strategy_type: Option<String>,  // Sub-strategy type (e.g., "Simple", "RSIBased")
    pub symbol: String,
    pub interval: String,
    pub start_date: ChronoDateTimeUtc,
    pub end_date: ChronoDateTimeUtc,
    pub initial_balance: Decimal,
    pub final_balance: Decimal,
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_percentage: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub total_invested: Decimal,
    pub strategy_parameters: Json,
    pub trades_data: Json, // Store individual trade records
    pub equity_curve: Json, // Store equity curve data points
    pub drawdown_curve: Json, // Store drawdown curve data points
    pub status: String, // completed, failed, running
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i64>,
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

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestResultResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub strategy_name: String,
    pub strategy_type: Option<String>,
    pub symbol: String,
    pub interval: String,
    pub start_date: ChronoDateTimeUtc,
    pub end_date: ChronoDateTimeUtc,
    pub initial_balance: Decimal,
    pub final_balance: Decimal,
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_percentage: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub total_invested: Decimal,
    pub status: String,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacktestResultDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub strategy_name: String,
    pub strategy_type: Option<String>,
    pub symbol: String,
    pub interval: String,
    pub start_date: ChronoDateTimeUtc,
    pub end_date: ChronoDateTimeUtc,
    pub initial_balance: Decimal,
    pub final_balance: Decimal,
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_percentage: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub total_invested: Decimal,
    pub strategy_parameters: serde_json::Value,
    pub trades_data: serde_json::Value,
    pub equity_curve: serde_json::Value,
    pub drawdown_curve: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

impl From<Model> for BacktestResultResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            strategy_name: model.strategy_name,
            strategy_type: model.strategy_type,
            symbol: model.symbol,
            interval: model.interval,
            start_date: model.start_date,
            end_date: model.end_date,
            initial_balance: model.initial_balance,
            final_balance: model.final_balance,
            total_return: model.total_return,
            total_return_percentage: model.total_return_percentage,
            max_drawdown: model.max_drawdown,
            max_drawdown_percentage: model.max_drawdown_percentage,
            sharpe_ratio: model.sharpe_ratio,
            total_trades: model.total_trades,
            winning_trades: model.winning_trades,
            losing_trades: model.losing_trades,
            win_rate: model.win_rate,
            profit_factor: model.profit_factor,
            largest_win: model.largest_win,
            largest_loss: model.largest_loss,
            average_win: model.average_win,
            average_loss: model.average_loss,
            total_invested: model.total_invested,
            status: model.status,
            error_message: model.error_message,
            execution_time_ms: model.execution_time_ms,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl From<Model> for BacktestResultDetailResponse {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            strategy_name: model.strategy_name,
            strategy_type: model.strategy_type,
            symbol: model.symbol,
            interval: model.interval,
            start_date: model.start_date,
            end_date: model.end_date,
            initial_balance: model.initial_balance,
            final_balance: model.final_balance,
            total_return: model.total_return,
            total_return_percentage: model.total_return_percentage,
            max_drawdown: model.max_drawdown,
            max_drawdown_percentage: model.max_drawdown_percentage,
            sharpe_ratio: model.sharpe_ratio,
            total_trades: model.total_trades,
            winning_trades: model.winning_trades,
            losing_trades: model.losing_trades,
            win_rate: model.win_rate,
            profit_factor: model.profit_factor,
            largest_win: model.largest_win,
            largest_loss: model.largest_loss,
            average_win: model.average_win,
            average_loss: model.average_loss,
            total_invested: model.total_invested,
            strategy_parameters: model.strategy_parameters,
            trades_data: model.trades_data,
            equity_curve: model.equity_curve,
            drawdown_curve: model.drawdown_curve,
            status: model.status,
            error_message: model.error_message,
            execution_time_ms: model.execution_time_ms,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}