use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::signals::{StrategySignal, StrategySignalType};

/// Strategy execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StrategyMode {
    Backtest,    // Historical simulation
    Paper,       // Live simulation (no real trades)
    Live,        // Live trading with real money
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskLevel {
    Conservative,
    Moderate,
    Aggressive,
    Expert,
}

/// Strategy category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StrategyCategory {
    DCA,              // Dollar Cost Averaging
    TechnicalAnalysis, // Technical indicators
    Arbitrage,        // Price differences
    MeanReversion,    // Buy low, sell high
    Momentum,         // Trend following
    Scalping,         // High-frequency small profits
    Swing,            // Medium-term positions
    GridTrading,      // Grid-based strategies
    Custom,           // User-defined strategies
}

/// Base trait for all trading strategies
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Strategy metadata
    fn metadata(&self) -> StrategyMetadata;

    /// Initialize strategy with parameters and mode
    async fn initialize(
        &mut self,
        parameters: &Value,
        mode: StrategyMode,
        context: &StrategyContext,
    ) -> Result<(), AppError>;

    /// Analyze market data and generate signals
    async fn analyze(
        &mut self,
        context: &StrategyContext,
    ) -> Result<Option<StrategySignal>, AppError>;

    /// Validate strategy parameters
    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError>;

    /// Get parameter schema for UI generation
    fn parameter_schema(&self) -> Value;

    /// Handle position updates (for live trading)
    async fn on_position_update(&mut self, position: &PositionUpdate) -> Result<(), AppError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Handle order updates (for live trading)
    async fn on_order_update(&mut self, order: &OrderUpdate) -> Result<(), AppError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Called when strategy is stopped
    async fn on_stop(&mut self) -> Result<(), AppError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get current strategy state for persistence
    fn get_state(&self) -> Result<Value, AppError>;

    /// Restore strategy state from persistence
    fn restore_state(&mut self, state: &Value) -> Result<(), AppError>;

    /// Check if strategy can handle the given symbol/market
    fn supports_symbol(&self, symbol: &str) -> bool {
        // Default implementation supports all symbols
        true
    }

    /// Get minimum required data points for analysis
    fn min_data_points(&self) -> usize {
        1
    }

    /// Get strategy performance metrics (for live strategies)
    fn get_metrics(&self) -> StrategyMetrics {
        StrategyMetrics::default()
    }
}

/// Strategy metadata for discovery and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: StrategyCategory,
    pub risk_level: RiskLevel,
    pub supported_modes: Vec<StrategyMode>,
    pub min_balance: Option<Decimal>,
    pub max_positions: Option<u32>,
    pub supported_intervals: Vec<String>, // e.g., ["1m", "5m", "1h", "1d"]
    pub tags: Vec<String>,
}

/// Context provided to strategies during execution
#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub strategy_id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub interval: String,
    pub mode: StrategyMode,
    pub current_time: DateTime<Utc>,
    pub historical_data: Vec<Kline>,
    pub current_price: Decimal,
    pub available_balance: Decimal,
    pub current_positions: Vec<Position>,
    pub market_data: MarketData,
}

/// Market data context
#[derive(Debug, Clone, Default)]
pub struct MarketData {
    pub volume_24h: Option<Decimal>,
    pub price_change_24h: Option<Decimal>,
    pub bid_price: Option<Decimal>,
    pub ask_price: Option<Decimal>,
    pub spread: Option<Decimal>,
}

/// Current position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub current_price: Decimal,
    pub pnl: Decimal,
    pub pnl_percentage: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Position update event
#[derive(Debug, Clone)]
pub struct PositionUpdate {
    pub symbol: String,
    pub old_quantity: Decimal,
    pub new_quantity: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

/// Order update event
#[derive(Debug, Clone)]
pub struct OrderUpdate {
    pub order_id: String,
    pub symbol: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyMetrics {
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub max_drawdown: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Strategy configuration for registration
pub struct StrategyConfig {
    pub metadata: StrategyMetadata,
    pub factory: Box<dyn StrategyFactory>,
}

/// Factory trait for creating strategy instances
pub trait StrategyFactory: Send + Sync {
    fn create(&self) -> Box<dyn Strategy>;
    fn metadata(&self) -> &StrategyMetadata;
}

/// Trait for strategies that support backtesting
#[async_trait]
pub trait BacktestableStrategy: Strategy {
    /// Run backtest simulation on historical data
    async fn backtest(
        &mut self,
        historical_data: &[Kline],
        initial_balance: Decimal,
        config: &Value,
    ) -> Result<BacktestResult, AppError>;
}

/// Backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub trades: Vec<BacktestTrade>,
    pub metrics: StrategyMetrics,
    pub final_balance: Decimal,
    pub max_drawdown: Decimal,
    pub total_return: Decimal,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub sharpe_ratio: Option<Decimal>,
}

/// Individual trade in backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub timestamp: DateTime<Utc>,
    pub signal_type: StrategySignalType,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub total_value: Decimal,
    pub pnl: Option<Decimal>,
    pub pnl_percentage: Option<Decimal>,
    pub reason: String,
}

/// Trait for strategies that support live execution
#[async_trait]
pub trait LiveExecutableStrategy: Strategy {
    /// Start live execution
    async fn start_live_execution(&mut self, context: &StrategyContext) -> Result<(), AppError>;

    /// Stop live execution
    async fn stop_live_execution(&mut self) -> Result<(), AppError>;

    /// Check if strategy is currently running
    fn is_running(&self) -> bool;

    /// Get next execution time (for scheduled strategies)
    fn next_execution_time(&self) -> Option<DateTime<Utc>>;
}

/// Trait for strategies that can be paused and resumed
#[async_trait]
pub trait ControllableStrategy: Strategy {
    /// Pause strategy execution
    async fn pause(&mut self) -> Result<(), AppError>;

    /// Resume strategy execution
    async fn resume(&mut self) -> Result<(), AppError>;

    /// Check if strategy is paused
    fn is_paused(&self) -> bool;
}