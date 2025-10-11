use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Trade side for grid operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Grid trading order types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridOrderType {
    /// Buy order at grid level
    Buy,
    /// Sell order at grid level
    Sell,
    /// Take profit order
    TakeProfit,
    /// Stop loss order
    StopLoss,
}

/// Grid level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLevel {
    /// Price level for this grid
    pub price: Decimal,
    /// Order type at this level
    pub order_type: GridOrderType,
    /// Quantity to trade at this level
    pub quantity: Decimal,
    /// Whether this level is active
    pub is_active: bool,
    /// Number of times this level has been filled
    pub fill_count: u32,
    /// Last fill timestamp
    pub last_fill_time: Option<DateTime<Utc>>,
    /// Total quantity filled at this level
    pub total_filled: Decimal,
}

/// Grid trading strategy state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridTradingState {
    /// Current grid levels
    pub grid_levels: Vec<GridLevel>,
    /// Current inventory (positive = long, negative = short)
    pub inventory: Decimal,
    /// Average entry price
    pub average_entry_price: Option<Decimal>,
    /// Total realized profit/loss
    pub realized_pnl: Decimal,
    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,
    /// Total number of grid trades executed
    pub total_trades: u32,
    /// Current grid center price
    pub grid_center: Decimal,
    /// Grid upper bound
    pub grid_upper_bound: Decimal,
    /// Grid lower bound
    pub grid_lower_bound: Decimal,
    /// Is grid currently active
    pub is_active: bool,
    /// Last rebalance timestamp
    pub last_rebalance_time: Option<DateTime<Utc>>,
    /// Grid statistics
    pub stats: GridStats,
}

impl Default for GridTradingState {
    fn default() -> Self {
        Self {
            grid_levels: Vec::new(),
            inventory: Decimal::ZERO,
            average_entry_price: None,
            realized_pnl: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            total_trades: 0,
            grid_center: Decimal::ZERO,
            grid_upper_bound: Decimal::ZERO,
            grid_lower_bound: Decimal::ZERO,
            is_active: false,
            last_rebalance_time: None,
            stats: GridStats::default(),
        }
    }
}

/// Grid trading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStats {
    /// Number of buy orders filled
    pub buy_fills: u32,
    /// Number of sell orders filled
    pub sell_fills: u32,
    /// Average buy price
    pub avg_buy_price: Option<Decimal>,
    /// Average sell price
    pub avg_sell_price: Option<Decimal>,
    /// Total volume traded (sum of all buys and sells)
    pub total_volume: Decimal,
    /// Total capital deployed (sum of buy orders only)
    pub total_deployed: Decimal,
    /// Grid efficiency (filled orders / total orders)
    pub grid_efficiency: Decimal,
    /// Maximum inventory reached
    pub max_inventory: Decimal,
    /// Minimum inventory reached (most negative)
    pub min_inventory: Decimal,
    /// Time in position (hours)
    pub time_in_position: Decimal,
}

impl Default for GridStats {
    fn default() -> Self {
        Self {
            buy_fills: 0,
            sell_fills: 0,
            avg_buy_price: None,
            avg_sell_price: None,
            total_volume: Decimal::ZERO,
            total_deployed: Decimal::ZERO,
            grid_efficiency: Decimal::ZERO,
            max_inventory: Decimal::ZERO,
            min_inventory: Decimal::ZERO,
            time_in_position: Decimal::ZERO,
        }
    }
}

/// Grid execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridExecution {
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Grid level index
    pub grid_level_index: usize,
    /// Execution price
    pub price: Decimal,
    /// Executed quantity
    pub quantity: Decimal,
    /// Order type executed
    pub order_type: GridOrderType,
    /// Grid level before execution
    pub grid_level_before: Decimal,
    /// Grid level after execution
    pub grid_level_after: Decimal,
    /// Inventory before execution
    pub inventory_before: Decimal,
    /// Inventory after execution
    pub inventory_after: Decimal,
    /// Realized PnL from this execution
    pub realized_pnl: Decimal,
    /// Market conditions at execution
    pub market_conditions: GridMarketConditions,
    /// Execution reason
    pub reason: String,
}

/// Market conditions for grid trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridMarketConditions {
    /// Current price
    pub price: Decimal,
    /// Bid-ask spread
    pub spread: Option<Decimal>,
    /// Market volatility (ATR or similar)
    pub volatility: Option<Decimal>,
    /// Volume
    pub volume: Option<Decimal>,
    /// Price trend (RSI-based or moving average)
    pub trend: Option<TrendDirection>,
    /// Support and resistance levels
    pub support_level: Option<Decimal>,
    pub resistance_level: Option<Decimal>,
}

impl Default for GridMarketConditions {
    fn default() -> Self {
        Self {
            price: Decimal::ZERO,
            spread: None,
            volatility: None,
            volume: None,
            trend: None,
            support_level: None,
            resistance_level: None,
        }
    }
}

/// Trend direction for market analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Sideways,
}

/// Grid rebalancing reasons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RebalanceReason {
    /// Price moved outside grid bounds
    PriceOutOfBounds,
    /// Volatility change requires grid adjustment
    VolatilityChange,
    /// Time-based rebalancing
    TimeInterval,
    /// Manual rebalancing triggered
    Manual,
    /// Risk management triggered rebalancing
    RiskManagement,
    /// Market condition change
    MarketCondition,
}

/// Grid order status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridOrderStatus {
    /// Order is pending/active
    Pending,
    /// Order has been filled
    Filled,
    /// Order was cancelled
    Cancelled,
    /// Order failed
    Failed,
    /// Order is being replaced
    Replacing,
}

/// Risk management settings for grid trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRiskSettings {
    /// Maximum inventory size (absolute value)
    pub max_inventory: Decimal,
    /// Stop loss percentage from entry
    pub stop_loss_pct: Option<Decimal>,
    /// Take profit percentage from entry
    pub take_profit_pct: Option<Decimal>,
    /// Maximum drawdown before stopping grid
    pub max_drawdown_pct: Decimal,
    /// Maximum time in position (hours)
    pub max_time_in_position: Option<u32>,
    /// Enable dynamic grid adjustment
    pub dynamic_adjustment: bool,
    /// Volatility threshold for grid pause
    pub volatility_pause_threshold: Option<Decimal>,
}

impl Default for GridRiskSettings {
    fn default() -> Self {
        Self {
            max_inventory: Decimal::from(1000), // Base currency units
            stop_loss_pct: Some(Decimal::new(5, 2)), // 5%
            take_profit_pct: Some(Decimal::new(10, 2)), // 10%
            max_drawdown_pct: Decimal::new(10, 2), // 10%
            max_time_in_position: None,
            dynamic_adjustment: true,
            volatility_pause_threshold: Some(Decimal::from(50)), // 50% ATR
        }
    }
}

/// Grid performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPerformance {
    /// Total return percentage
    pub total_return_pct: Decimal,
    /// Annualized return percentage
    pub annualized_return_pct: Option<Decimal>,
    /// Maximum drawdown
    pub max_drawdown_pct: Decimal,
    /// Sharpe ratio
    pub sharpe_ratio: Option<Decimal>,
    /// Win rate (profitable vs unprofitable periods)
    pub win_rate: Decimal,
    /// Average profit per trade
    pub avg_profit_per_trade: Decimal,
    /// Grid utilization (how often levels are hit)
    pub grid_utilization: Decimal,
    /// Market capture ratio (profit vs market movement)
    pub market_capture_ratio: Option<Decimal>,
}

impl Default for GridPerformance {
    fn default() -> Self {
        Self {
            total_return_pct: Decimal::ZERO,
            annualized_return_pct: None,
            max_drawdown_pct: Decimal::ZERO,
            sharpe_ratio: None,
            win_rate: Decimal::ZERO,
            avg_profit_per_trade: Decimal::ZERO,
            grid_utilization: Decimal::ZERO,
            market_capture_ratio: None,
        }
    }
}

/// Grid trading modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GridTradingMode {
    /// Standard grid with fixed levels
    Standard,
    /// Arithmetic progression grid
    Arithmetic,
    /// Geometric progression grid
    Geometric,
    /// Dynamic grid that adjusts to volatility
    Dynamic,
    /// Zone-based grid trading
    ZoneBased,
}

/// Grid spacing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSpacing {
    /// Grid trading mode
    pub mode: GridTradingMode,
    /// For standard/arithmetic: fixed spacing between levels
    pub fixed_spacing: Option<Decimal>,
    /// For arithmetic: spacing increment per level
    pub arithmetic_increment: Option<Decimal>,
    /// For geometric: multiplier between levels
    pub geometric_multiplier: Option<Decimal>,
    /// For dynamic: base spacing as percentage of price
    pub dynamic_base_pct: Option<Decimal>,
    /// For dynamic: volatility adjustment factor
    pub volatility_factor: Option<Decimal>,
}

impl Default for GridSpacing {
    fn default() -> Self {
        Self {
            mode: GridTradingMode::Standard,
            fixed_spacing: Some(Decimal::new(1, 2)), // 1% default spacing
            arithmetic_increment: None,
            geometric_multiplier: None,
            dynamic_base_pct: Some(Decimal::new(5, 3)), // 0.5%
            volatility_factor: Some(Decimal::new(2, 0)), // 2x volatility adjustment
        }
    }
}

/// Grid bounds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridBounds {
    /// Upper bound (price or percentage above center)
    pub upper_bound: Decimal,
    /// Lower bound (price or percentage below center)
    pub lower_bound: Decimal,
    /// Whether bounds are absolute prices or percentages
    pub bounds_type: BoundsType,
    /// Auto-adjust bounds based on market conditions
    pub auto_adjust: bool,
    /// Support/resistance level integration
    pub use_support_resistance: bool,
}

/// Types of grid bounds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BoundsType {
    /// Absolute price values
    AbsolutePrice,
    /// Percentage from center price
    PercentageFromCenter,
    /// Multiple of ATR (Average True Range)
    VolatilityBased,
    /// Based on support/resistance levels
    TechnicalLevels,
}

impl Default for GridBounds {
    fn default() -> Self {
        Self {
            upper_bound: Decimal::new(10, 2), // 10% above center
            lower_bound: Decimal::new(10, 2), // 10% below center
            bounds_type: BoundsType::PercentageFromCenter,
            auto_adjust: true,
            use_support_resistance: false,
        }
    }
}