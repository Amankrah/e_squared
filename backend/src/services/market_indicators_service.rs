use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{error, warn, debug, info};
use rust_decimal::Decimal;

use crate::utils::errors::AppError;

/// CoinGecko API response for BTC dominance
#[derive(Debug, Deserialize)]
pub struct CoinGeckoGlobal {
    pub data: CoinGeckoData,
}

#[derive(Debug, Deserialize)]
pub struct CoinGeckoData {
    pub market_cap_percentage: std::collections::HashMap<String, f64>,
    pub market_cap_change_percentage_24h_usd: Option<f64>,
}

/// CoinGecko API response for Bitcoin price
#[derive(Debug, Deserialize)]
pub struct CoinGeckoBtcPrice {
    pub bitcoin: CoinGeckoBtcPriceDetail,
}

#[derive(Debug, Deserialize)]
pub struct CoinGeckoBtcPriceDetail {
    pub usd: f64,
    pub usd_24h_change: Option<f64>,
}

/// FRED API response for M2 Money Supply
#[derive(Debug, Deserialize)]
pub struct FredResponse {
    pub observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
pub struct FredObservation {
    pub date: String,
    pub value: String,
}

/// Bitcoin Dominance data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcDominanceData {
    pub value: Decimal,
    pub change_24h: Option<Decimal>,
    pub timestamp: i64,
}

/// M2 Money Supply data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M2Data {
    pub value: Decimal,
    pub change: Option<Decimal>,
    pub percent_change: Option<Decimal>,
    pub date: String,
    pub timestamp: i64,
}

/// Bitcoin Price data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcPriceData {
    pub price: Decimal,
    pub change_24h: Option<Decimal>,
    pub percent_change_24h: Option<Decimal>,
    pub high_24h: Option<Decimal>,
    pub low_24h: Option<Decimal>,
    pub timestamp: i64,
}

/// Rate limiter for API calls
#[derive(Debug, Clone)]
struct RateLimiter {
    last_call: Arc<RwLock<Option<Instant>>>,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(min_interval_ms: u64) -> Self {
        Self {
            last_call: Arc::new(RwLock::new(None)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    async fn wait_if_needed(&self) {
        let now = Instant::now();

        {
            let last_call = self.last_call.read().await;
            if let Some(last_time) = *last_call {
                let elapsed = now.duration_since(last_time);
                if elapsed < self.min_interval {
                    let wait_time = self.min_interval - elapsed;
                    debug!("Rate limiting market indicators API: waiting {:?}", wait_time);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }

        let mut last_call = self.last_call.write().await;
        *last_call = Some(Instant::now());
    }
}

/// Market Indicators Service for fetching M2 and BTC dominance
#[derive(Clone)]
pub struct MarketIndicatorsService {
    client: Client,
    rate_limiter: RateLimiter,
    btc_cached_data: Arc<RwLock<Option<(BtcDominanceData, Instant)>>>,
    btc_price_cached_data: Arc<RwLock<Option<(BtcPriceData, Instant)>>>,
    m2_cached_data: Arc<RwLock<Option<(M2Data, Instant)>>>,
    cache_duration: Duration,
    btc_price_cache_duration: Duration,
}

impl MarketIndicatorsService {
    /// Create a new market indicators service
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            rate_limiter: RateLimiter::new(2000), // 2 seconds between calls
            btc_cached_data: Arc::new(RwLock::new(None)),
            btc_price_cached_data: Arc::new(RwLock::new(None)),
            m2_cached_data: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_secs(3600), // Cache for 1 hour (these update slowly)
            btc_price_cache_duration: Duration::from_secs(60), // Cache BTC price for 1 minute
        }
    }

    /// Get Bitcoin Dominance with caching
    pub async fn get_btc_dominance(&self) -> Result<BtcDominanceData, AppError> {
        // Check cache first
        {
            let cache = self.btc_cached_data.read().await;
            if let Some((data, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.cache_duration {
                    debug!("Returning cached BTC dominance data");
                    return Ok(data.clone());
                }
            }
        }

        // Fetch fresh data
        let btc_data = self.fetch_btc_dominance_from_api().await?;

        // Update cache
        {
            let mut cache = self.btc_cached_data.write().await;
            *cache = Some((btc_data.clone(), Instant::now()));
        }

        Ok(btc_data)
    }

    /// Get M2 Money Supply with caching
    pub async fn get_m2(&self) -> Result<M2Data, AppError> {
        // Check cache first
        {
            let cache = self.m2_cached_data.read().await;
            if let Some((data, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.cache_duration {
                    debug!("Returning cached M2 data");
                    return Ok(data.clone());
                }
            }
        }

        // Fetch fresh data
        let m2_data = self.fetch_m2_from_api().await?;

        // Update cache
        {
            let mut cache = self.m2_cached_data.write().await;
            *cache = Some((m2_data.clone(), Instant::now()));
        }

        Ok(m2_data)
    }

    /// Get Bitcoin Price with caching
    pub async fn get_btc_price(&self) -> Result<BtcPriceData, AppError> {
        // Check cache first
        {
            let cache = self.btc_price_cached_data.read().await;
            if let Some((data, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.btc_price_cache_duration {
                    debug!("Returning cached BTC price data");
                    return Ok(data.clone());
                }
            }
        }

        // Fetch fresh data
        let btc_price_data = self.fetch_btc_price_from_api().await?;

        // Update cache
        {
            let mut cache = self.btc_price_cached_data.write().await;
            *cache = Some((btc_price_data.clone(), Instant::now()));
        }

        Ok(btc_price_data)
    }

    /// Fetch BTC Dominance from CoinGecko API (free, no API key needed)
    async fn fetch_btc_dominance_from_api(&self) -> Result<BtcDominanceData, AppError> {
        self.rate_limiter.wait_if_needed().await;

        let url = "https://api.coingecko.com/api/v3/global";

        debug!("Fetching BTC dominance from CoinGecko API");

        let response = self.client
            .get(url)
            .header("User-Agent", "E-Squared Trading Platform 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch BTC dominance from CoinGecko: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("CoinGecko API returned status: {}", response.status());
            if let Ok(text) = response.text().await {
                error!("API error response: {}", text);
            }
            return Err(AppError::InternalServerError);
        }

        let coingecko_response: CoinGeckoGlobal = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse CoinGecko response: {}", e);
                AppError::InternalServerError
            })?;

        // Extract BTC dominance percentage
        let btc_dominance = coingecko_response.data.market_cap_percentage
            .get("btc")
            .ok_or_else(|| {
                error!("No BTC dominance in CoinGecko response");
                AppError::InternalServerError
            })?;

        let value = Decimal::try_from(*btc_dominance)
            .map_err(|e| {
                error!("Failed to convert BTC dominance to Decimal: {}", e);
                AppError::InternalServerError
            })?;

        // Get 24h change if available
        let change_24h = coingecko_response.data.market_cap_change_percentage_24h_usd
            .and_then(|change| Decimal::try_from(change).ok());

        let timestamp = chrono::Utc::now().timestamp();

        info!("Successfully fetched BTC dominance from CoinGecko: {}%", value);

        Ok(BtcDominanceData {
            value,
            change_24h,
            timestamp,
        })
    }

    /// Fetch M2 Money Supply from FRED API
    /// Note: This returns US M2. For global M2, you'd need to aggregate multiple sources
    async fn fetch_m2_from_api(&self) -> Result<M2Data, AppError> {
        self.rate_limiter.wait_if_needed().await;

        // Get FRED API key from environment, or return static fallback data
        let api_key = match std::env::var("FRED_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                warn!("FRED_API_KEY not set, returning static M2 data");
                // Return recent M2 value as fallback (as of Oct 2024)
                let timestamp = chrono::Utc::now().timestamp();
                return Ok(M2Data {
                    value: Decimal::try_from(21080.0).unwrap(), // ~$21.08 trillion
                    change: None,
                    percent_change: None,
                    date: "2024-10-14".to_string(),
                    timestamp,
                });
            }
        };

        // FRED API endpoint for M2 Money Stock (US) - get last 2 observations for change calculation
        let url = format!("https://api.stlouisfed.org/fred/series/observations?series_id=WM2NS&api_key={}&file_type=json&limit=2&sort_order=desc", api_key);

        debug!("Fetching M2 data from FRED API");

        let response = self.client
            .get(url)
            .header("User-Agent", "E-Squared Trading Platform 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch M2 from FRED: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("FRED API returned status: {}", response.status());
            if let Ok(text) = response.text().await {
                error!("API error response: {}", text);
            }
            return Err(AppError::InternalServerError);
        }

        let fred_response: FredResponse = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse FRED response: {}", e);
                AppError::InternalServerError
            })?;

        // Get the latest observation
        let observation = fred_response.observations.first()
            .ok_or_else(|| {
                error!("No observations in FRED response");
                AppError::InternalServerError
            })?;

        // Parse the value (in billions of dollars)
        let m2_value = observation.value.parse::<f64>()
            .map_err(|e| {
                error!("Failed to parse M2 value: {}", e);
                AppError::InternalServerError
            })?;

        let value = Decimal::try_from(m2_value)
            .map_err(|e| {
                error!("Failed to convert M2 value to Decimal: {}", e);
                AppError::InternalServerError
            })?;

        // Calculate change if previous observation is available
        let (change, percent_change) = if fred_response.observations.len() >= 2 {
            let previous_observation = &fred_response.observations[1];
            if let Ok(previous_value) = previous_observation.value.parse::<f64>() {
                let change_val = m2_value - previous_value;
                let percent_change_val = if previous_value != 0.0 {
                    (change_val / previous_value) * 100.0
                } else {
                    0.0
                };

                let change_decimal = Decimal::try_from(change_val).ok();
                let percent_decimal = Decimal::try_from(percent_change_val).ok();
                (change_decimal, percent_decimal)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let timestamp = chrono::Utc::now().timestamp();

        info!("Successfully fetched M2 from FRED: ${} billion (as of {})", value, observation.date);

        Ok(M2Data {
            value,
            change,
            percent_change,
            date: observation.date.clone(),
            timestamp,
        })
    }

    /// Fetch Bitcoin Price from CoinGecko API (free, no API key needed)
    async fn fetch_btc_price_from_api(&self) -> Result<BtcPriceData, AppError> {
        self.rate_limiter.wait_if_needed().await;

        // CoinGecko simple price endpoint with 24h data
        let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true&include_24hr_vol=true";

        debug!("Fetching BTC price from CoinGecko API");

        let response = self.client
            .get(url)
            .header("User-Agent", "E-Squared Trading Platform 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch BTC price from CoinGecko: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("CoinGecko API returned status: {}", response.status());
            if let Ok(text) = response.text().await {
                error!("API error response: {}", text);
            }
            return Err(AppError::InternalServerError);
        }

        let price_response: CoinGeckoBtcPrice = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse CoinGecko price response: {}", e);
                AppError::InternalServerError
            })?;

        let btc_data = &price_response.bitcoin;

        let price = Decimal::try_from(btc_data.usd)
            .map_err(|e| {
                error!("Failed to convert BTC price to Decimal: {}", e);
                AppError::InternalServerError
            })?;

        // Calculate 24h change
        let (change_24h, percent_change_24h) = if let Some(percent_change) = btc_data.usd_24h_change {
            let percent_decimal = Decimal::try_from(percent_change).ok();
            let change_decimal = if let Some(pct) = percent_decimal {
                // Calculate absolute change from percentage
                Some(price * pct / Decimal::from(100))
            } else {
                None
            };
            (change_decimal, percent_decimal)
        } else {
            (None, None)
        };

        let timestamp = chrono::Utc::now().timestamp();

        info!("Successfully fetched BTC price from CoinGecko: ${}", price);

        Ok(BtcPriceData {
            price,
            change_24h,
            percent_change_24h,
            high_24h: None, // Simple API doesn't provide this
            low_24h: None,  // Simple API doesn't provide this
            timestamp,
        })
    }

    /// Clear all caches
    pub async fn clear_cache(&self) {
        let mut btc_cache = self.btc_cached_data.write().await;
        *btc_cache = None;
        let mut btc_price_cache = self.btc_price_cached_data.write().await;
        *btc_price_cache = None;
        let mut m2_cache = self.m2_cached_data.write().await;
        *m2_cache = None;
    }
}

impl Default for MarketIndicatorsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = MarketIndicatorsService::new();
        assert!(service.btc_cached_data.read().await.is_none());
        assert!(service.m2_cached_data.read().await.is_none());
    }
}
