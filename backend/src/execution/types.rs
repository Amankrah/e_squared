use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::strategies::core::{StrategySignal, StrategyMode, Position, OrderStatus};

/// Execution engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent strategy instances
    pub max_concurrent_strategies: usize,
    /// Default order timeout in seconds
    pub order_timeout_seconds: u64,
    /// Risk management settings
    pub risk_config: RiskConfig,
    /// Execution mode (paper trading, live trading)
    pub execution_mode: ExecutionMode,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum portfolio value at risk (as percentage)
    pub max_portfolio_risk_percentage: Decimal,
    /// Maximum position size per symbol (as percentage of portfolio)
    pub max_position_size_percentage: Decimal,
    /// Maximum daily loss limit
    pub daily_loss_limit: Option<Decimal>,
    /// Maximum number of trades per day
    pub max_trades_per_day: Option<u32>,
    /// Minimum time between trades for same symbol (minutes)
    pub min_trade_interval_minutes: u32,
    /// Enable stop loss on all positions
    pub enable_global_stop_loss: bool,
    /// Global stop loss percentage
    pub global_stop_loss_percentage: Option<Decimal>,
}

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {
    /// Paper trading (simulation)
    Paper,
    /// Live trading with real money
    Live,
    /// Dry run (analyze but don't execute)
    DryRun,
}

/// Retry configuration for failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
}

/// Strategy execution instance
#[derive(Debug, Clone)]
pub struct StrategyInstance {
    /// Instance ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Strategy ID
    pub strategy_id: String,
    /// Trading symbol
    pub symbol: String,
    /// Time interval
    pub interval: String,
    /// Strategy mode
    pub mode: StrategyMode,
    /// Instance configuration
    pub config: serde_json::Value,
    /// Current status
    pub status: InstanceStatus,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Last execution time
    pub last_execution: Option<DateTime<Utc>>,
    /// Next scheduled execution
    pub next_execution: Option<DateTime<Utc>>,
    /// Performance metrics
    pub metrics: InstanceMetrics,
}

/// Strategy instance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    /// Instance is starting up
    Starting,
    /// Instance is running normally
    Running,
    /// Instance is paused
    Paused,
    /// Instance is stopping
    Stopping,
    /// Instance has stopped
    Stopped,
    /// Instance encountered an error
    Error(String),
}

/// Performance metrics for a strategy instance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceMetrics {
    /// Total signals generated
    pub signals_generated: u32,
    /// Total signals executed
    pub signals_executed: u32,
    /// Total execution errors
    pub execution_errors: u32,
    /// Total profit/loss
    pub total_pnl: Decimal,
    /// Total return percentage
    pub total_return_percentage: Decimal,
    /// Number of winning trades
    pub winning_trades: u32,
    /// Number of losing trades
    pub losing_trades: u32,
    /// Largest winning trade
    pub largest_win: Decimal,
    /// Largest losing trade
    pub largest_loss: Decimal,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: u64,
    /// Last error message
    pub last_error: Option<String>,
}

/// Execution result for a strategy signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Signal that was executed
    pub signal: StrategySignal,
    /// Execution status
    pub status: ExecutionStatus,
    /// Order ID (if order was placed)
    pub order_id: Option<String>,
    /// Execution price
    pub execution_price: Option<Decimal>,
    /// Executed quantity
    pub executed_quantity: Option<Decimal>,
    /// Execution fees
    pub fees: Option<Decimal>,
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution was successful
    Success,
    /// Execution failed
    Failed,
    /// Execution was skipped due to risk management
    Skipped,
    /// Execution was cancelled
    Cancelled,
    /// Partial execution
    Partial,
}

/// Live execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Strategy instance created
    InstanceCreated {
        instance_id: Uuid,
        user_id: Uuid,
        strategy_id: String,
    },
    /// Strategy instance started
    InstanceStarted {
        instance_id: Uuid,
    },
    /// Strategy instance paused
    InstancePaused {
        instance_id: Uuid,
        reason: String,
    },
    /// Strategy instance resumed
    InstanceResumed {
        instance_id: Uuid,
    },
    /// Strategy instance stopped
    InstanceStopped {
        instance_id: Uuid,
        reason: String,
    },
    /// Signal generated by strategy
    SignalGenerated {
        instance_id: Uuid,
        signal: StrategySignal,
    },
    /// Signal execution started
    ExecutionStarted {
        instance_id: Uuid,
        signal: StrategySignal,
    },
    /// Signal execution completed
    ExecutionCompleted {
        instance_id: Uuid,
        result: ExecutionResult,
    },
    /// Position updated
    PositionUpdated {
        instance_id: Uuid,
        position: Position,
    },
    /// Order filled notification
    OrderFilled {
        instance_id: Uuid,
        order_id: String,
        symbol: String,
        filled_quantity: Decimal,
        execution_price: Decimal,
        timestamp: DateTime<Utc>,
    },
    /// Risk limit exceeded
    RiskLimitExceeded {
        instance_id: Uuid,
        limit_type: String,
        current_value: Decimal,
        limit_value: Decimal,
    },
    /// Error occurred
    ErrorOccurred {
        instance_id: Uuid,
        error: String,
        context: serde_json::Value,
    },
}

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// Is the execution allowed
    pub allowed: bool,
    /// Risk score (0.0 to 1.0)
    pub risk_score: Decimal,
    /// Warnings
    pub warnings: Vec<String>,
    /// Blocking reasons (if not allowed)
    pub blocking_reasons: Vec<String>,
    /// Recommended adjustments
    pub recommendations: Vec<String>,
}

/// Portfolio snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    /// User ID
    pub user_id: Uuid,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Total portfolio value
    pub total_value: Decimal,
    /// Available cash balance
    pub cash_balance: Decimal,
    /// All positions
    pub positions: Vec<Position>,
    /// Total unrealized PnL
    pub unrealized_pnl: Decimal,
    /// Daily PnL
    pub daily_pnl: Decimal,
    /// Portfolio allocation by symbol
    pub allocation: std::collections::HashMap<String, Decimal>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionStats {
    /// Total number of active strategies
    pub active_strategies: usize,
    /// Total signals processed today
    pub signals_today: u32,
    /// Total executions today
    pub executions_today: u32,
    /// Success rate
    pub success_rate: Decimal,
    /// Average execution time
    pub avg_execution_time_ms: u64,
    /// Total volume traded today
    pub volume_today: Decimal,
    /// Top performing strategy
    pub top_strategy: Option<String>,
    /// System uptime
    pub uptime_seconds: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_strategies: 50,
            order_timeout_seconds: 30,
            risk_config: RiskConfig::default(),
            execution_mode: ExecutionMode::Paper,
            retry_config: RetryConfig::default(),
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_portfolio_risk_percentage: Decimal::from(20), // 20%
            max_position_size_percentage: Decimal::from(10),  // 10%
            daily_loss_limit: Some(Decimal::from(1000)),      // $1000
            max_trades_per_day: Some(100),
            min_trade_interval_minutes: 1,
            enable_global_stop_loss: false,
            global_stop_loss_percentage: Some(Decimal::from(5)),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,    // 1 second
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,       // 30 seconds
        }
    }
}