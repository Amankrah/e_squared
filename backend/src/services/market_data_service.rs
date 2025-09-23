use reqwest::Client;
use serde::Deserialize;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, interval, sleep, Instant};
use tracing::{info, warn, error, debug};

use crate::utils::errors::AppError;
use crate::models::dca_strategy::MarketDataModel;

/// Fear & Greed Index data structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FearGreedResponse {
    pub name: String,
    pub data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FearGreedData {
    pub value: String,
    pub value_classification: String,
    pub timestamp: String,
    pub time_until_update: Option<String>,
}

/// CoinGecko price data
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CoinGeckoPriceData {
    #[serde(flatten)]
    pub prices: HashMap<String, f64>,
}

/// Binance 24hr ticker statistics
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Binance24hrTicker {
    pub symbol: String,
    #[serde(rename = "priceChange")]
    pub price_change: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    #[serde(rename = "weightedAvgPrice")]
    pub weighted_avg_price: String,
    #[serde(rename = "prevClosePrice")]
    pub prev_close_price: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "bidPrice")]
    pub bid_price: String,
    #[serde(rename = "askPrice")]
    pub ask_price: String,
    #[serde(rename = "openPrice")]
    pub open_price: String,
    #[serde(rename = "highPrice")]
    pub high_price: String,
    #[serde(rename = "lowPrice")]
    pub low_price: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "openTime")]
    pub open_time: i64,
    #[serde(rename = "closeTime")]
    pub close_time: i64,
    #[serde(rename = "firstId")]
    pub first_id: i64,
    #[serde(rename = "lastId")]
    pub last_id: i64,
    pub count: i64,
}

/// Technical indicators from TradingView-like calculation
#[derive(Debug, Clone)]
pub struct TechnicalIndicators {
    pub rsi_14: Option<Decimal>,
    pub ema_20: Option<Decimal>,
    pub ema_50: Option<Decimal>,
    pub ema_200: Option<Decimal>,
    pub volatility_7d: Option<Decimal>,
    pub volatility_30d: Option<Decimal>,
    pub support_level: Option<Decimal>,
    pub resistance_level: Option<Decimal>,
    pub trend_direction: Option<String>,
}

/// Rate limiter for API calls
#[derive(Debug, Clone)]
pub struct RateLimiter {
    last_call: Arc<RwLock<HashMap<String, Instant>>>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_call: Arc::new(RwLock::new(HashMap::new())),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    pub async fn wait_if_needed(&self, key: &str) {
        let now = Instant::now();

        {
            let last_call = self.last_call.read().await;
            if let Some(&last_time) = last_call.get(key) {
                let elapsed = now.duration_since(last_time);
                if elapsed < self.min_interval {
                    let wait_time = self.min_interval - elapsed;
                    debug!("Rate limiting: waiting {:?} for {}", wait_time, key);
                    sleep(wait_time).await;
                }
            }
        }

        let mut last_call = self.last_call.write().await;
        last_call.insert(key.to_string(), Instant::now());
    }
}

/// Market data service for real-time crypto market analysis
#[derive(Clone)]
pub struct MarketDataService {
    client: Client,
    fear_greed_url: String,
    coingecko_url: String,
    binance_url: String,
    rate_limiter: RateLimiter,
}

impl MarketDataService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            fear_greed_url: "https://api.alternative.me/fng/".to_string(),
            coingecko_url: "https://api.coingecko.com/api/v3".to_string(),
            binance_url: "https://api.binance.com".to_string(),
            rate_limiter: RateLimiter::new(1000), // 1 second between calls
        }
    }

    /// Get current Fear & Greed Index
    pub async fn get_fear_greed_index(&self) -> Result<i32, AppError> {
        self.rate_limiter.wait_if_needed("fear_greed").await;

        let url = format!("{}?limit=1", self.fear_greed_url);

        let response = self.client
            .get(&url)
            .header("User-Agent", "E-Squared DCA Bot 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch Fear & Greed Index: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("Fear & Greed API returned status: {}", response.status());
            return Err(AppError::InternalServerError);
        }

        let fear_greed: FearGreedResponse = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse Fear & Greed response: {}", e);
                AppError::InternalServerError
            })?;

        if let Some(data) = fear_greed.data.first() {
            data.value.parse::<i32>()
                .map_err(|e| {
                    error!("Failed to parse Fear & Greed value: {}", e);
                    AppError::InternalServerError
                })
        } else {
            Err(AppError::InternalServerError)
        }
    }

    /// Get current price from CoinGecko
    pub async fn get_current_price(&self, symbol: &str) -> Result<Decimal, AppError> {
        self.rate_limiter.wait_if_needed(&format!("coingecko_price_{}", symbol)).await;

        let coin_id = self.symbol_to_coingecko_id(symbol);
        let url = format!("{}/simple/price?ids={}&vs_currencies=usd",
                         self.coingecko_url, coin_id);

        let response = self.client
            .get(&url)
            .header("User-Agent", "E-Squared DCA Bot 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch price for {}: {}", symbol, e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("CoinGecko API returned status: {} for {}", response.status(), symbol);
            return Err(AppError::InternalServerError);
        }

        let price_data: HashMap<String, HashMap<String, f64>> = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse price response for {}: {}", symbol, e);
                AppError::InternalServerError
            })?;

        if let Some(coin_data) = price_data.get(&coin_id) {
            if let Some(usd_price) = coin_data.get("usd") {
                Decimal::try_from(*usd_price)
                    .map_err(|e| {
                        error!("Failed to convert price to Decimal: {}", e);
                        AppError::InternalServerError
                    })
            } else {
                Err(AppError::InternalServerError)
            }
        } else {
            Err(AppError::InternalServerError)
        }
    }

    /// Get 24hr ticker statistics from Binance
    #[allow(dead_code)]
    pub async fn get_binance_ticker(&self, symbol: &str) -> Result<Binance24hrTicker, AppError> {
        let binance_symbol = format!("{}USDT", symbol.to_uppercase());
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.binance_url, binance_symbol);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch Binance ticker for {}: {}", symbol, e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("Binance API returned status: {} for {}", response.status(), symbol);
            return Err(AppError::InternalServerError);
        }

        response.json::<Binance24hrTicker>()
            .await
            .map_err(|e| {
                error!("Failed to parse Binance ticker response for {}: {}", symbol, e);
                AppError::InternalServerError
            })
    }

    /// Calculate volatility from price data
    #[allow(dead_code)]
    pub async fn calculate_volatility(&self, symbol: &str, days: u32) -> Result<Decimal, AppError> {
        let coin_id = self.symbol_to_coingecko_id(symbol);
        let url = format!("{}/coins/{}/market_chart?vs_currency=usd&days={}&interval=daily",
                         self.coingecko_url, coin_id, days);

        let response = self.client
            .get(&url)
            .header("User-Agent", "E-Squared DCA Bot 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch market chart for {}: {}", symbol, e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            return Err(AppError::InternalServerError);
        }

        #[derive(Deserialize)]
        struct MarketChart {
            prices: Vec<[f64; 2]>,
        }

        let chart: MarketChart = response.json()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if chart.prices.len() < 2 {
            return Err(AppError::InternalServerError);
        }

        // Calculate daily returns
        let mut returns = Vec::new();
        for i in 1..chart.prices.len() {
            let current_price = chart.prices[i][1];
            let previous_price = chart.prices[i-1][1];
            let daily_return = (current_price - previous_price) / previous_price;
            returns.push(daily_return);
        }

        // Calculate standard deviation of returns (volatility)
        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|return_val| {
                let diff = return_val - mean_return;
                diff * diff
            })
            .sum::<f64>() / returns.len() as f64;

        let volatility = variance.sqrt() * 100.0; // Convert to percentage

        Decimal::try_from(volatility)
            .map_err(|_| AppError::InternalServerError)
    }

    /// Calculate simple technical indicators
    pub async fn calculate_technical_indicators(&self, symbol: &str) -> Result<TechnicalIndicators, AppError> {
        // Get 200 days of data for EMA calculations
        let coin_id = self.symbol_to_coingecko_id(symbol);
        let url = format!("{}/coins/{}/market_chart?vs_currency=usd&days=200&interval=daily",
                         self.coingecko_url, coin_id);

        let response = self.client
            .get(&url)
            .header("User-Agent", "E-Squared DCA Bot 1.0")
            .send()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if !response.status().is_success() {
            return Ok(TechnicalIndicators {
                rsi_14: None,
                ema_20: None,
                ema_50: None,
                ema_200: None,
                volatility_7d: None,
                volatility_30d: None,
                support_level: None,
                resistance_level: None,
                trend_direction: None,
            });
        }

        #[derive(Deserialize)]
        struct MarketChart {
            prices: Vec<[f64; 2]>,
        }

        let chart: MarketChart = response.json()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if chart.prices.len() < 200 {
            return Ok(TechnicalIndicators {
                rsi_14: None,
                ema_20: None,
                ema_50: None,
                ema_200: None,
                volatility_7d: None,
                volatility_30d: None,
                support_level: None,
                resistance_level: None,
                trend_direction: None,
            });
        }

        let prices: Vec<f64> = chart.prices.iter().map(|p| p[1]).collect();

        // Calculate EMAs
        let ema_20 = self.calculate_ema(&prices, 20);
        let ema_50 = self.calculate_ema(&prices, 50);
        let ema_200 = self.calculate_ema(&prices, 200);

        // Calculate RSI
        let rsi_14 = self.calculate_rsi(&prices, 14);

        // Calculate volatilities
        let volatility_7d = self.calculate_volatility_from_prices(&prices[prices.len()-7..]);
        let volatility_30d = self.calculate_volatility_from_prices(&prices[prices.len()-30..]);

        // Calculate support and resistance levels
        let recent_prices = &prices[prices.len()-50..]; // Last 50 days
        let support_level = recent_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let resistance_level = recent_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Determine trend direction
        let trend_direction = if let (Some(ema_20), Some(ema_50), Some(ema_200)) = (ema_20, ema_50, ema_200) {
            if ema_20 > ema_50 && ema_50 > ema_200 {
                Some("bullish".to_string())
            } else if ema_20 < ema_50 && ema_50 < ema_200 {
                Some("bearish".to_string())
            } else {
                Some("sideways".to_string())
            }
        } else {
            None
        };

        Ok(TechnicalIndicators {
            rsi_14: rsi_14.and_then(|v| Decimal::try_from(v).ok()),
            ema_20: ema_20.and_then(|v| Decimal::try_from(v).ok()),
            ema_50: ema_50.and_then(|v| Decimal::try_from(v).ok()),
            ema_200: ema_200.and_then(|v| Decimal::try_from(v).ok()),
            volatility_7d: volatility_7d.and_then(|v| Decimal::try_from(v).ok()),
            volatility_30d: volatility_30d.and_then(|v| Decimal::try_from(v).ok()),
            support_level: Decimal::try_from(support_level).ok(),
            resistance_level: Decimal::try_from(resistance_level).ok(),
            trend_direction,
        })
    }

    /// Get comprehensive market data for DCA decision making
    pub async fn get_market_data(&self, symbol: &str) -> Result<MarketDataModel, AppError> {
        // Add delay between calls to respect rate limits
        let price = self.get_current_price(symbol).await?;

        // Small delay before next API call
        sleep(Duration::from_millis(200)).await;
        let fear_greed = self.get_fear_greed_index().await.ok();

        // Small delay before next API call
        sleep(Duration::from_millis(200)).await;
        let technical_indicators = self.calculate_technical_indicators(symbol).await?;

        Ok(MarketDataModel {
            id: uuid::Uuid::new_v4(),
            asset_symbol: symbol.to_uppercase(),
            price,
            volume_24h: None, // Could be fetched from Binance ticker
            market_cap: None, // Could be fetched from CoinGecko
            fear_greed_index: fear_greed,
            volatility_7d: technical_indicators.volatility_7d,
            volatility_30d: technical_indicators.volatility_30d,
            rsi_14: technical_indicators.rsi_14,
            ema_20: technical_indicators.ema_20,
            ema_50: technical_indicators.ema_50,
            ema_200: technical_indicators.ema_200,
            support_level: technical_indicators.support_level,
            resistance_level: technical_indicators.resistance_level,
            trend_direction: technical_indicators.trend_direction,
            timestamp: Utc::now(),
            created_at: Utc::now(),
        })
    }

    /// Helper function to map symbols to CoinGecko IDs
    fn symbol_to_coingecko_id(&self, symbol: &str) -> String {
        match symbol.to_lowercase().as_str() {
            "btc" | "bitcoin" => "bitcoin".to_string(),
            "eth" | "ethereum" => "ethereum".to_string(),
            "sol" | "solana" => "solana".to_string(),
            "ada" | "cardano" => "cardano".to_string(),
            "dot" | "polkadot" => "polkadot".to_string(),
            "matic" | "polygon" => "matic-network".to_string(),
            "avax" | "avalanche" => "avalanche-2".to_string(),
            "link" | "chainlink" => "chainlink".to_string(),
            "uni" | "uniswap" => "uniswap".to_string(),
            "ltc" | "litecoin" => "litecoin".to_string(),
            _ => symbol.to_lowercase(),
        }
    }

    /// Calculate Exponential Moving Average
    fn calculate_ema(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period {
            return None;
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];

        for &price in prices.iter().skip(1) {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }

        Some(ema)
    }

    /// Calculate Relative Strength Index
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> Option<f64> {
        if prices.len() < period + 1 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        if gains.len() < period {
            return None;
        }

        let avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
        let avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }

    /// Calculate volatility from price array
    fn calculate_volatility_from_prices(&self, prices: &[f64]) -> Option<f64> {
        if prices.len() < 2 {
            return None;
        }

        let mut returns = Vec::new();
        for i in 1..prices.len() {
            let daily_return = (prices[i] - prices[i-1]) / prices[i-1];
            returns.push(daily_return);
        }

        let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|return_val| {
                let diff = return_val - mean_return;
                diff * diff
            })
            .sum::<f64>() / returns.len() as f64;

        Some(variance.sqrt() * 100.0)
    }
}

impl Default for MarketDataService {
    fn default() -> Self {
        Self::new()
    }
}

/// Background service to periodically update market data
#[allow(dead_code)]
pub struct MarketDataUpdater {
    market_service: MarketDataService,
    update_interval: Duration,
}

#[allow(dead_code)]
impl MarketDataUpdater {
    pub fn new(update_interval_minutes: u64) -> Self {
        Self {
            market_service: MarketDataService::new(),
            update_interval: Duration::from_secs(update_interval_minutes * 60),
        }
    }

    /// Start the background market data update loop
    pub async fn start_background_updates(&self, symbols: Vec<String>) {
        let mut interval = interval(self.update_interval);

        info!("Starting market data updates for symbols: {:?}", symbols);

        loop {
            interval.tick().await;

            for symbol in &symbols {
                match self.market_service.get_market_data(symbol).await {
                    Ok(market_data) => {
                        info!("Updated market data for {}: price=${}, fear_greed={:?}",
                              symbol, market_data.price, market_data.fear_greed_index);
                        // Here you would save to database
                        // db.save_market_data(market_data).await;
                    }
                    Err(e) => {
                        error!("Failed to update market data for {}: {:?}", symbol, e);
                    }
                }
            }
        }
    }
}