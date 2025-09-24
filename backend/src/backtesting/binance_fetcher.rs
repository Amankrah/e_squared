use std::sync::Arc;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::exchange_connectors::{Kline, KlineInterval};
use crate::utils::errors::AppError;
use super::data_cache::{get_cache, DataCache};

/// Binance public API base URL
const BINANCE_API_BASE: &str = "https://api.binance.com";

/// Maximum klines per request (Binance limit)
const MAX_KLINES_PER_REQUEST: usize = 1000;

/// Request weight for klines endpoint
const KLINES_REQUEST_WEIGHT: u32 = 1;

/// Binance kline response structure
#[derive(Debug, Deserialize)]
struct BinanceKline(
    i64,    // Open time
    String, // Open
    String, // High
    String, // Low
    String, // Close
    String, // Volume
    i64,    // Close time
    String, // Quote asset volume
    i64,    // Number of trades
    String, // Taker buy base asset volume
    String, // Taker buy quote asset volume
    String, // Ignore
);

/// Binance data fetcher with caching
pub struct BinanceFetcher {
    client: reqwest::Client,
    cache: Arc<DataCache>,
}

impl BinanceFetcher {
    /// Create a new Binance fetcher
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache: get_cache(),
        }
    }

    /// Fetch historical klines data with intelligent caching
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Kline>, AppError> {
        // Validate time range
        if start_time >= end_time {
            return Err(AppError::BadRequest("Invalid time range".to_string()));
        }

        // Check cache first
        if let Some(cached_data) = self.cache.get(symbol, interval, start_time, end_time).await {
            info!("Using cached data for {}:{}", symbol, interval);
            return Ok((*cached_data).clone());
        }

        // Fetch from Binance API
        info!(
            "Fetching {} klines from {} to {}",
            symbol, start_time, end_time
        );

        let all_klines = self
            .fetch_klines_chunked(symbol, interval, start_time, end_time)
            .await?;

        // Store in cache for future requests
        self.cache
            .store(symbol, interval, start_time, end_time, all_klines.clone())
            .await;

        Ok(all_klines)
    }

    /// Fetch klines in chunks (due to Binance 1000 limit)
    async fn fetch_klines_chunked(
        &self,
        symbol: &str,
        interval: &KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Kline>, AppError> {
        let mut all_klines = Vec::new();
        let mut current_start = start_time;

        // Calculate interval duration
        let interval_duration = self.get_interval_duration(interval);

        while current_start < end_time {
            // Calculate chunk end time
            let chunk_end = std::cmp::min(
                current_start + interval_duration * (MAX_KLINES_PER_REQUEST as i32),
                end_time,
            );

            // Wait if rate limited
            self.cache.wait_if_needed().await;

            // Check if we can make request
            if !self.cache.can_make_request(KLINES_REQUEST_WEIGHT).await {
                // Wait for rate limit reset
                self.cache.wait_if_needed().await;
            }

            // Fetch chunk
            let chunk = self
                .fetch_single_chunk(symbol, interval, current_start, chunk_end)
                .await?;

            // Record the request
            self.cache.record_request(KLINES_REQUEST_WEIGHT).await;

            if chunk.is_empty() {
                break;
            }

            // Get the last kline's close time for next iteration
            if let Some(last) = chunk.last() {
                current_start = last.close_time + ChronoDuration::milliseconds(1);
                all_klines.extend(chunk);
            } else {
                break;
            }

            // Small delay to be nice to the API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(all_klines)
    }

    /// Fetch a single chunk of klines
    async fn fetch_single_chunk(
        &self,
        symbol: &str,
        interval: &KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Kline>, AppError> {
        let url = format!("{}/api/v3/klines", BINANCE_API_BASE);

        let params = [
            ("symbol", symbol.to_uppercase()),
            ("interval", interval.to_string()),
            ("startTime", start_time.timestamp_millis().to_string()),
            ("endTime", end_time.timestamp_millis().to_string()),
            ("limit", MAX_KLINES_PER_REQUEST.to_string()),
        ];

        debug!(
            "Fetching klines: {} {} from {} to {}",
            symbol, interval, start_time, end_time
        );

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Binance: {}", e);
                AppError::ExternalServiceError(format!("Failed to fetch data: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Binance API error: {} - {}", status, text);

            // Handle specific error codes
            if status == 429 {
                return Err(AppError::RateLimitError(
                    "Binance rate limit exceeded. Please try again later.".to_string(),
                ));
            } else if status == 418 {
                return Err(AppError::Banned(
                    "IP has been banned from Binance API".to_string(),
                ));
            }

            return Err(AppError::ExternalServiceError(format!(
                "Binance API error: {} - {}",
                status, text
            )));
        }

        let binance_klines: Vec<BinanceKline> = response.json().await.map_err(|e| {
            error!("Failed to parse Binance response: {}", e);
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        // Convert to our Kline format
        let klines: Vec<Kline> = binance_klines
            .into_iter()
            .map(|k| self.convert_kline(k))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(klines)
    }

    /// Convert Binance kline to our format
    fn convert_kline(&self, binance_kline: BinanceKline) -> Result<Kline, AppError> {
        Ok(Kline {
            open_time: DateTime::from_timestamp_millis(binance_kline.0)
                .ok_or_else(|| AppError::ParseError("Invalid open time".to_string()))?,
            open: Decimal::from_str_exact(&binance_kline.1)
                .map_err(|e| AppError::ParseError(format!("Invalid open price: {}", e)))?,
            high: Decimal::from_str_exact(&binance_kline.2)
                .map_err(|e| AppError::ParseError(format!("Invalid high price: {}", e)))?,
            low: Decimal::from_str_exact(&binance_kline.3)
                .map_err(|e| AppError::ParseError(format!("Invalid low price: {}", e)))?,
            close: Decimal::from_str_exact(&binance_kline.4)
                .map_err(|e| AppError::ParseError(format!("Invalid close price: {}", e)))?,
            volume: Decimal::from_str_exact(&binance_kline.5)
                .map_err(|e| AppError::ParseError(format!("Invalid volume: {}", e)))?,
            close_time: DateTime::from_timestamp_millis(binance_kline.6)
                .ok_or_else(|| AppError::ParseError("Invalid close time".to_string()))?,
            quote_asset_volume: Decimal::from_str_exact(&binance_kline.7)
                .map_err(|e| AppError::ParseError(format!("Invalid quote volume: {}", e)))?,
            number_of_trades: binance_kline.8,
            taker_buy_base_asset_volume: Decimal::from_str_exact(&binance_kline.9)
                .map_err(|e| AppError::ParseError(format!("Invalid taker buy volume: {}", e)))?,
            taker_buy_quote_asset_volume: Decimal::from_str_exact(&binance_kline.10)
                .map_err(|e| {
                    AppError::ParseError(format!("Invalid taker buy quote volume: {}", e))
                })?,
        })
    }

    /// Get interval duration
    fn get_interval_duration(&self, interval: &KlineInterval) -> ChronoDuration {
        match interval.as_str() {
            "1m" => ChronoDuration::minutes(1),
            "3m" => ChronoDuration::minutes(3),
            "5m" => ChronoDuration::minutes(5),
            "15m" => ChronoDuration::minutes(15),
            "30m" => ChronoDuration::minutes(30),
            "1h" => ChronoDuration::hours(1),
            "2h" => ChronoDuration::hours(2),
            "4h" => ChronoDuration::hours(4),
            "6h" => ChronoDuration::hours(6),
            "8h" => ChronoDuration::hours(8),
            "12h" => ChronoDuration::hours(12),
            "1d" => ChronoDuration::days(1),
            "3d" => ChronoDuration::days(3),
            "1w" => ChronoDuration::weeks(1),
            "1M" => ChronoDuration::days(30),
            _ => ChronoDuration::hours(1), // Default
        }
    }

    /// Validate symbol format (e.g., BTCUSDT)
    pub fn validate_symbol(symbol: &str) -> Result<(), AppError> {
        if symbol.is_empty() {
            return Err(AppError::BadRequest("Symbol cannot be empty".to_string()));
        }

        // Basic validation - symbol should be uppercase alphanumeric
        if !symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return Err(AppError::BadRequest(
                "Symbol should be uppercase (e.g., BTCUSDT)".to_string(),
            ));
        }

        Ok(())
    }

    /// Get available symbols from Binance (cached)
    pub async fn get_symbols(&self) -> Result<Vec<String>, AppError> {
        // This would typically fetch from Binance exchange info endpoint
        // For now, return common symbols
        Ok(vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
            "SOLUSDT".to_string(),
            "ADAUSDT".to_string(),
            "DOGEUSDT".to_string(),
            "XRPUSDT".to_string(),
            "DOTUSDT".to_string(),
            "UNIUSDT".to_string(),
            "LINKUSDT".to_string(),
        ])
    }
}