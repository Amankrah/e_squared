use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{error, warn, debug, info};
use rust_decimal::Decimal;

use crate::utils::errors::AppError;

/// Yahoo Finance API response for DXY quote
#[derive(Debug, Deserialize)]
pub struct YahooFinanceResponse {
    pub chart: YahooChart,
}

#[derive(Debug, Deserialize)]
pub struct YahooChart {
    pub result: Vec<YahooResult>,
}

#[derive(Debug, Deserialize)]
pub struct YahooResult {
    pub meta: YahooMeta,
    pub timestamp: Option<Vec<i64>>,
    pub indicators: YahooIndicators,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YahooMeta {
    pub regular_market_price: Option<f64>,
    pub chart_previous_close: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct YahooIndicators {
    pub quote: Vec<YahooQuote>,
}

#[derive(Debug, Deserialize)]
pub struct YahooQuote {
    pub open: Option<Vec<Option<f64>>>,
    pub high: Option<Vec<Option<f64>>>,
    pub low: Option<Vec<Option<f64>>>,
    pub close: Option<Vec<Option<f64>>>,
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

/// DXY Service for fetching US Dollar Index data from Yahoo Finance
#[derive(Clone)]
pub struct DxyService {
    client: Client,
    rate_limiter: RateLimiter,
    cached_data: Arc<RwLock<Option<(DxyData, Instant)>>>,
    cache_duration: Duration,
}

impl DxyService {
    /// Create a new DXY service
    pub fn new(_api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            rate_limiter: RateLimiter::new(2000), // 2 seconds between calls
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

    /// Fetch DXY data from Yahoo Finance API
    /// DXY is available on Yahoo Finance as DX-Y.NYB
    async fn fetch_dxy_from_api(&self) -> Result<DxyData, AppError> {
        self.rate_limiter.wait_if_needed().await;

        // Yahoo Finance DXY symbol
        let symbol = "DX-Y.NYB";
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1d&range=5d", symbol);

        debug!("Fetching DXY data from Yahoo Finance API");

        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch DXY data from Yahoo Finance: {}", e);
                AppError::InternalServerError
            })?;

        if !response.status().is_success() {
            warn!("Yahoo Finance API returned status: {}", response.status());
            if let Ok(text) = response.text().await {
                error!("API error response: {}", text);
            }
            return Err(AppError::InternalServerError);
        }

        let yahoo_response: YahooFinanceResponse = response.json()
            .await
            .map_err(|e| {
                error!("Failed to parse Yahoo Finance response: {}", e);
                AppError::InternalServerError
            })?;

        // Extract data from response
        let result = yahoo_response.chart.result.first()
            .ok_or_else(|| {
                error!("No results in Yahoo Finance response");
                AppError::InternalServerError
            })?;

        let current_price = result.meta.regular_market_price
            .ok_or_else(|| {
                error!("No regular market price in Yahoo Finance response");
                AppError::InternalServerError
            })?;

        let previous_close = result.meta.chart_previous_close.unwrap_or(current_price);

        let value = Decimal::try_from(current_price)
            .map_err(|e| {
                error!("Failed to convert DXY value to Decimal: {}", e);
                AppError::InternalServerError
            })?;

        // Calculate change
        let change_val = current_price - previous_close;
        let change = Decimal::try_from(change_val).ok();

        // Calculate percent change
        let percent_change_val = if previous_close != 0.0 {
            (change_val / previous_close) * 100.0
        } else {
            0.0
        };
        let percent_change = Decimal::try_from(percent_change_val).ok();

        // Get high/low from quote data
        let quote = result.indicators.quote.first();
        let (high_24h, low_24h) = if let Some(q) = quote {
            let high = q.high.as_ref()
                .and_then(|h| h.iter().filter_map(|v| *v).max_by(|a, b| a.partial_cmp(b).unwrap()))
                .and_then(|v| Decimal::try_from(v).ok());

            let low = q.low.as_ref()
                .and_then(|l| l.iter().filter_map(|v| *v).min_by(|a, b| a.partial_cmp(b).unwrap()))
                .and_then(|v| Decimal::try_from(v).ok());

            (high, low)
        } else {
            (None, None)
        };

        let timestamp = chrono::Utc::now().timestamp();

        info!("Successfully fetched DXY data from Yahoo Finance: {}", value);

        Ok(DxyData {
            value,
            change,
            percent_change,
            high_24h,
            low_24h,
            timestamp,
        })
    }

    /// Clear the cache (useful for testing or forcing a refresh)
    pub async fn clear_cache(&self) {
        let mut cache = self.cached_data.write().await;
        *cache = None;
    }
}

impl Default for DxyService {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dxy_service_creation() {
        let service = DxyService::new(None);
        assert!(service.cached_data.read().await.is_none());
    }
}
