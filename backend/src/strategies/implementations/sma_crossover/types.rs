use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Types of SMA crossover signals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrossoverSignal {
    /// Fast SMA crossed above slow SMA (bullish)
    BullishCrossover,
    /// Fast SMA crossed below slow SMA (bearish)
    BearishCrossover,
    /// No crossover detected
    None,
}

/// SMA crossover strategy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMACrossoverState {
    /// Last fast SMA value
    pub last_fast_sma: Option<Decimal>,
    /// Last slow SMA value
    pub last_slow_sma: Option<Decimal>,
    /// Previous fast SMA value (for crossover detection)
    pub prev_fast_sma: Option<Decimal>,
    /// Previous slow SMA value (for crossover detection)
    pub prev_slow_sma: Option<Decimal>,
    /// Current position (1 for long, -1 for short, 0 for none)
    pub position: i8,
    /// Entry price of current position
    pub entry_price: Option<Decimal>,
    /// Total profit/loss
    pub total_pnl: Decimal,
    /// Number of trades executed
    pub trade_count: u32,
    /// Number of winning trades
    pub winning_trades: u32,
    /// Last signal generated
    pub last_signal: Option<CrossoverSignal>,
    /// Last signal timestamp
    pub last_signal_time: Option<DateTime<Utc>>,
}

impl Default for SMACrossoverState {
    fn default() -> Self {
        Self {
            last_fast_sma: None,
            last_slow_sma: None,
            prev_fast_sma: None,
            prev_slow_sma: None,
            position: 0,
            entry_price: None,
            total_pnl: Decimal::ZERO,
            trade_count: 0,
            winning_trades: 0,
            last_signal: None,
            last_signal_time: None,
        }
    }
}

/// Execution record for SMA crossover trades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMACrossoverExecution {
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Signal that triggered the execution
    pub signal: CrossoverSignal,
    /// Price at execution
    pub price: Decimal,
    /// Position size
    pub quantity: Decimal,
    /// Trade side (Buy/Sell)
    pub side: TradeSide,
    /// Fast SMA value at execution
    pub fast_sma: Decimal,
    /// Slow SMA value at execution
    pub slow_sma: Decimal,
    /// Market conditions at execution
    pub market_conditions: MarketConditions,
    /// Reason for execution
    pub reason: String,
}

/// Trade side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Market conditions snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Current price
    pub price: Decimal,
    /// 24h price change percentage
    pub price_change_24h: Option<Decimal>,
    /// Current volume
    pub volume: Option<Decimal>,
    /// RSI value (if available)
    pub rsi: Option<Decimal>,
    /// MACD histogram (if available)
    pub macd_histogram: Option<Decimal>,
    /// Bollinger Band position (0-1, where 0.5 is middle)
    pub bb_position: Option<Decimal>,
}

impl Default for MarketConditions {
    fn default() -> Self {
        Self {
            price: Decimal::ZERO,
            price_change_24h: None,
            volume: None,
            rsi: None,
            macd_histogram: None,
            bb_position: None,
        }
    }
}

/// Signal strength enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalStrength {
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

/// Crossover analysis result
#[derive(Debug, Clone)]
pub struct CrossoverAnalysis {
    /// Detected signal type
    pub signal: CrossoverSignal,
    /// Signal strength
    pub strength: SignalStrength,
    /// Confidence level (0.0 to 1.0)
    pub confidence: Decimal,
    /// Current fast SMA value
    pub fast_sma: Decimal,
    /// Current slow SMA value
    pub slow_sma: Decimal,
    /// SMA spread (fast - slow)
    pub sma_spread: Decimal,
    /// Additional market indicators
    pub market_conditions: MarketConditions,
}

/// Risk management parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    /// Stop loss percentage (0.02 = 2%)
    pub stop_loss_pct: Decimal,
    /// Take profit percentage (0.04 = 4%)
    pub take_profit_pct: Decimal,
    /// Maximum position size as percentage of balance
    pub max_position_pct: Decimal,
    /// Minimum time between signals (in minutes)
    pub min_signal_interval: u32,
    /// Enable trailing stop loss
    pub trailing_stop: bool,
    /// Trailing stop distance percentage
    pub trailing_stop_pct: Option<Decimal>,
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            stop_loss_pct: Decimal::new(2, 2), // 2%
            take_profit_pct: Decimal::new(4, 2), // 4%
            max_position_pct: Decimal::new(10, 2), // 10%
            min_signal_interval: 30, // 30 minutes
            trailing_stop: false,
            trailing_stop_pct: None,
        }
    }
}

/// Performance metrics for the strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total return percentage
    pub total_return_pct: Decimal,
    /// Win rate (0.0 to 1.0)
    pub win_rate: Decimal,
    /// Average win amount
    pub avg_win: Decimal,
    /// Average loss amount
    pub avg_loss: Decimal,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: Decimal,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: Decimal,
    /// Sharpe ratio (if applicable)
    pub sharpe_ratio: Option<Decimal>,
    /// Total number of trades
    pub total_trades: u32,
    /// Average trade duration (in hours)
    pub avg_trade_duration: Option<Decimal>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return_pct: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            max_drawdown_pct: Decimal::ZERO,
            sharpe_ratio: None,
            total_trades: 0,
            avg_trade_duration: None,
        }
    }
}

/// Filters for signal validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalFilters {
    /// Minimum volume requirement
    pub min_volume: Option<Decimal>,
    /// Maximum spread percentage
    pub max_spread_pct: Option<Decimal>,
    /// RSI overbought level (skip buy signals)
    pub rsi_overbought: Option<Decimal>,
    /// RSI oversold level (skip sell signals)
    pub rsi_oversold: Option<Decimal>,
    /// Require MACD confirmation
    pub macd_confirmation: bool,
    /// Minimum SMA spread for signal validity
    pub min_sma_spread_pct: Option<Decimal>,
}

impl Default for SignalFilters {
    fn default() -> Self {
        Self {
            min_volume: None,
            max_spread_pct: Some(Decimal::new(5, 3)), // 0.5%
            rsi_overbought: Some(Decimal::from(70)),
            rsi_oversold: Some(Decimal::from(30)),
            macd_confirmation: false,
            min_sma_spread_pct: Some(Decimal::new(1, 3)), // 0.1%
        }
    }
}