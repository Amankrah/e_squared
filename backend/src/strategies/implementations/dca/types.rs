use serde::{Deserialize, Serialize};
use rust_decimal::{Decimal, prelude::FromPrimitive};

/// DCA strategy variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DCAType {
    /// Fixed amount at regular intervals
    Simple,
    /// Adjust amount based on RSI
    RSIBased,
    /// Adjust amount based on volatility
    VolatilityBased,
    /// Combine multiple factors
    Dynamic,
    /// Buy more when price drops
    DipBuying,
    /// Adjust based on market sentiment
    SentimentBased,
}

/// DCA execution frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DCAFrequency {
    Hourly(u32),     // Every N hours
    Daily(u32),      // Every N days
    Weekly(u32),     // Every N weeks
    Monthly(u32),    // Every N months
    Custom(u64),     // Custom interval in minutes
}

impl DCAFrequency {
    /// Get interval in minutes
    pub fn to_minutes(&self) -> u64 {
        match self {
            DCAFrequency::Hourly(hours) => (*hours as u64) * 60,
            DCAFrequency::Daily(days) => (*days as u64) * 24 * 60,
            DCAFrequency::Weekly(weeks) => (*weeks as u64) * 7 * 24 * 60,
            DCAFrequency::Monthly(months) => (*months as u64) * 30 * 24 * 60, // Approximate
            DCAFrequency::Custom(minutes) => *minutes,
        }
    }

    /// Get interval in hours
    pub fn to_hours(&self) -> u64 {
        self.to_minutes() / 60
    }
}

/// Price level configuration for dip buying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DipBuyingLevel {
    /// Price drop percentage to trigger this level
    pub price_drop_percentage: Decimal,
    /// Multiplier for the base amount
    pub amount_multiplier: Decimal,
    /// Maximum times this level can be triggered
    pub max_triggers: Option<u32>,
}

/// Market sentiment indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    /// Fear & Greed index threshold (0-100)
    pub fear_greed_threshold: Option<u32>,
    /// Social sentiment score threshold (-1.0 to 1.0)
    pub social_sentiment_threshold: Option<Decimal>,
    /// News sentiment threshold (-1.0 to 1.0)
    pub news_sentiment_threshold: Option<Decimal>,
    /// Multiplier when sentiment is bearish
    pub bearish_multiplier: Decimal,
    /// Multiplier when sentiment is bullish
    pub bullish_multiplier: Decimal,
}

/// RSI-based adjustment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSIConfig {
    /// RSI calculation period
    pub period: usize,
    /// RSI oversold threshold (typically 30)
    pub oversold_threshold: Decimal,
    /// RSI overbought threshold (typically 70)
    pub overbought_threshold: Decimal,
    /// Amount multiplier when oversold
    pub oversold_multiplier: Decimal,
    /// Amount multiplier when overbought
    pub overbought_multiplier: Decimal,
    /// Amount multiplier in normal range
    pub normal_multiplier: Decimal,
}

impl Default for RSIConfig {
    fn default() -> Self {
        Self {
            period: 14,
            oversold_threshold: Decimal::from(30),
            overbought_threshold: Decimal::from(70),
            oversold_multiplier: Decimal::new(2, 0), // 2.0
            overbought_multiplier: Decimal::new(5, 1), // 0.5
            normal_multiplier: Decimal::from(1),
        }
    }
}

/// Volatility-based adjustment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityConfig {
    /// Volatility calculation period
    pub period: usize,
    /// Low volatility threshold (percentage)
    pub low_threshold: Decimal,
    /// High volatility threshold (percentage)
    pub high_threshold: Decimal,
    /// Amount multiplier during low volatility
    pub low_volatility_multiplier: Decimal,
    /// Amount multiplier during high volatility
    pub high_volatility_multiplier: Decimal,
    /// Amount multiplier during normal volatility
    pub normal_multiplier: Decimal,
}

impl Default for VolatilityConfig {
    fn default() -> Self {
        Self {
            period: 20,
            low_threshold: Decimal::from(10),
            high_threshold: Decimal::from(30),
            low_volatility_multiplier: Decimal::new(8, 1), // 0.8
            high_volatility_multiplier: Decimal::new(15, 1), // 1.5
            normal_multiplier: Decimal::from(1),
        }
    }
}

/// Dynamic DCA factors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicFactors {
    /// Weight for RSI factor (0.0 to 1.0)
    pub rsi_weight: Decimal,
    /// Weight for volatility factor (0.0 to 1.0)
    pub volatility_weight: Decimal,
    /// Weight for sentiment factor (0.0 to 1.0)
    pub sentiment_weight: Decimal,
    /// Weight for price trend factor (0.0 to 1.0)
    pub trend_weight: Decimal,
    /// Maximum total multiplier
    pub max_multiplier: Decimal,
    /// Minimum total multiplier
    pub min_multiplier: Decimal,
}

/// DCA execution state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DCAState {
    /// Last execution timestamp
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    /// Total amount invested
    pub total_invested: Decimal,
    /// Total quantity purchased
    pub total_quantity: Decimal,
    /// Average purchase price
    pub average_price: Decimal,
    /// Number of purchases made
    pub purchase_count: u32,
    /// Execution count for each dip level
    pub dip_level_executions: std::collections::HashMap<String, u32>,
}

/// DCA execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCAExecution {
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Amount invested
    pub amount: Decimal,
    /// Quantity purchased
    pub quantity: Decimal,
    /// Purchase price
    pub price: Decimal,
    /// Strategy type used
    pub strategy_type: DCAType,
    /// Reason for this execution
    pub reason: String,
    /// Multiplier applied
    pub multiplier: Decimal,
    /// Market conditions at execution
    pub market_conditions: MarketConditions,
}

/// Market conditions snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketConditions {
    /// Current RSI value
    pub rsi: Option<Decimal>,
    /// Current volatility
    pub volatility: Option<Decimal>,
    /// Price change from reference point
    pub price_change_percentage: Option<Decimal>,
    /// Volume compared to average
    pub volume_ratio: Option<Decimal>,
    /// Market sentiment score
    pub sentiment_score: Option<Decimal>,
}