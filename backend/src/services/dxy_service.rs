use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{error, warn, debug, info};
use rust_decimal::Decimal;

use crate::utils::errors::AppError;

/// Twelve Data API error response
#[derive(Debug, Deserialize)]
pub struct TwelveDataError {
    pub code: Option<i32>,
    pub message: Option<String>,
    pub status: Option<String>,
}

/// Twelve Data API response for DXY quote
#[derive(Debug, Deserialize)]
pub struct TwelveDataQuote {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub datetime: Option<String>,
    pub timestamp: Option<i64>,
    pub open: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub close: Option<String>,
    pub previous_close: Option<String>,
    pub change: Option<String>,
    pub percent_change: Option<String>,
    pub average_volume: Option<String>,
    pub volume: Option<String>,
    // Error fields in case API returns error
    pub code: Option<i32>,
    pub message: Option<String>,
    pub status: Option<String>,
}

/// DXY (US Dollar Index) data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DxyData {
    pub value: Decimal,
    pub change: Option<Decimal>,
    pub percent_change: Option<Decimal>,
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
                    debug!("Rate limiting DXY API: waiting {:?}", wait_time);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }

        let mut last_call = self.last_call.write().await;
        *last_call = Some(Instant::now());
    }
}

/// DXY Service for fetching US Dollar Index data
#[derive(Clone)]
pub struct DxyService {
    client: Client,
    api_key: Option<String>,
    rate_limiter: RateLimiter,
    cached_data: Arc<RwLock<Option<(DxyData, Instant)>>>,
    cache_duration: Duration,
}

impl DxyService {
    /// Create a new DXY service
    /// api_key is optional - if not provided, will attempt to use free tier
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            rate_limiter: RateLimiter::new(8000), // 8 seconds between calls for free tier
            cached_data: Arc::new(RwLock::new(None)),
            cache_duration: Duration::from_secs(300), // Cache for 5 minutes
        }
    }

    /// Get current DXY value with caching
    pub async fn get_dxy(&self) -> Result<DxyData, AppError> {
        // Check cache first
        {
            let cache = self.cached_data.read().await;
            if let Some((data, cached_at)) = cache.as_ref() {
                if cached_at.elapsed() < self.cache_duration {
                    debug!("Returning cached DXY data");
                    return Ok(data.clone());
                }
            }
        }

        // Fetch fresh data
        let dxy_data = self.fetch_dxy_from_api().await?;

        // Update cache
        {
            let mut cache = self.cached_data.write().await;
            *cache = Some((dxy_data.clone(), Instant::now()));
        }

        Ok(dxy_data)
    }

    /// Fetch DXY data from Twelve Data API
    /// Using USD/JPY as proxy for dollar strength since DXY symbol may not be available
    async fn fetch_dxy_from_api(&self) -> Result<DxyData, AppError> {
        self.rate_limiter.wait_if_needed().await;

        // Try DXY first, fall back to USD/JPY if not available
        let symbols_to_try = vec!["DXY", "USD/JPY"];

        for symbol in symbols_to_try {
        let mut url = format!("https://api.twelvedata.com/quote?symbol={}&interval=1day", symbol);

        if let Some(ref api_key) = self.api_key {
            url.push_str(&format!("&apikey={}", api_key));
        }

        debug!("Fetching DXY data from Twelve Data API");

        let response = self.client
            .get(&url)
            .header("User-Agent", "E-Squared Trading Platform 1.0")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch DXY data: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("Twelve Data API returned status: {}", response.status());

            // Try to parse error response
            if let Ok(text) = response.text().await {
                error!("API error response: {}", text);
            }

            return Err(AppError::InternalServerError);
        }

        // Get the response text for debugging
        let response_text = response.text().await.map_err(|e| {
            error!("Failed to get response text: {}", e);
            AppError::InternalServerError
        })?;

        debug!("Twelve Data API response: {}", response_text);

        let quote: TwelveDataQuote = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("Failed to parse Twelve Data response: {}", e);
                error!("Response was: {}", response_text);
                AppError::InternalServerError
            })?;

        // Check if the response is an error
        if let Some(code) = quote.code {
            let message = quote.message.clone().unwrap_or_else(|| "Unknown API error".to_string());
            warn!("Twelve Data API error for symbol {} - code: {}, message: {}", symbol, code, message);
            continue; // Try next symbol
        }

        // Parse the data - if it fails, try next symbol
        let value = match quote.close
            .as_ref()
            .or(quote.open.as_ref())
            .and_then(|s| s.parse::<f64>().ok())
            .and_then(|v| Decimal::try_from(v).ok()) {
            Some(v) => v,
            None => {
                warn!("Failed to parse value for symbol {}", symbol);
                continue; // Try next symbol
            }
        };

        let change = quote.change
            .and_then(|s| s.parse::<f64>().ok())
            .and_then(|v| Decimal::try_from(v).ok());

        let percent_change = quote.percent_change
            .and_then(|s| s.parse::<f64>().ok())
            .and_then(|v| Decimal::try_from(v).ok());

        let high_24h = quote.high
            .and_then(|s| s.parse::<f64>().ok())
            .and_then(|v| Decimal::try_from(v).ok());

        let low_24h = quote.low
            .and_then(|s| s.parse::<f64>().ok())
            .and_then(|v| Decimal::try_from(v).ok());

        let timestamp = quote.timestamp
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        info!("Successfully fetched DXY data using symbol: {}", symbol);

        return Ok(DxyData {
            value,
            change,
            percent_change,
            high_24h,
            low_24h,
            timestamp,
        });
        } // End of for loop

        // If we get here, none of the symbols worked
        error!("Failed to fetch DXY data from all attempted symbols");
        Err(AppError::InternalServerError)
    }

    /// Clear the cache (useful for testing or forcing a refresh)
    pub async fn clear_cache(&self) {
        let mut cache = self.cached_data.write().await;
        *cache = None;
    }
}

impl Default for DxyService {
    fn default() -> Self {
        // Try to get API key from environment
        let api_key = std::env::var("TWELVE_DATA_API_KEY").ok();
        Self::new(api_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dxy_service_creation() {
        let service = DxyService::new(None);
        assert!(service.api_key.is_none());
    }

    #[tokio::test]
    async fn test_dxy_service_with_api_key() {
        let service = DxyService::new(Some("test_key".to_string()));
        assert!(service.api_key.is_some());
    }
}
