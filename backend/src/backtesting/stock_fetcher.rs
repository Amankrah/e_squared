use std::sync::Arc;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use rust_decimal::Decimal;
use serde::Deserialize;
use tracing::{debug, error, info};

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::data_cache::{get_cache, DataCache};

/// Alpha Vantage API base URL
const ALPHA_VANTAGE_BASE: &str = "https://www.alphavantage.co/query";

/// Request weight for Alpha Vantage (they use request counts)
const REQUEST_WEIGHT: u32 = 1;

/// Alpha Vantage time series response
#[derive(Debug, Deserialize)]
struct AlphaVantageTimeSeries {
    #[serde(rename = "Meta Data")]
    _meta_data: std::collections::HashMap<String, String>,
    #[serde(rename = "Time Series (Daily)")]
    time_series: Option<std::collections::HashMap<String, TimeSeriesEntry>>,
}

#[derive(Debug, Deserialize)]
struct TimeSeriesEntry {
    #[serde(rename = "1. open")]
    open: String,
    #[serde(rename = "2. high")]
    high: String,
    #[serde(rename = "3. low")]
    low: String,
    #[serde(rename = "4. close")]
    close: String,
    #[serde(rename = "5. volume")]
    volume: String,
}

/// Stock data fetcher for Alpha Vantage with caching
pub struct StockFetcher {
    client: reqwest::Client,
    cache: Arc<DataCache>,
    api_key: String,
}

impl StockFetcher {
    /// Create a new stock fetcher
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache: get_cache(),
            api_key,
        }
    }

    /// Fetch historical stock data
    /// Note: Alpha Vantage only supports daily intervals for historical data
    pub async fn fetch_klines(
        &self,
        symbol: &str,
        interval: &crate::exchange_connectors::KlineInterval,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Kline>, AppError> {
        // Validate time range
        if start_time >= end_time {
            return Err(AppError::BadRequest("Invalid time range".to_string()));
        }

        // Alpha Vantage only supports daily intervals for historical data
        // For backtesting purposes, we'll use daily data
        if !matches!(interval.as_str(), "1d" | "1w" | "1M") {
            return Err(AppError::BadRequest(
                "Stock backtesting only supports daily (1d), weekly (1w), or monthly (1M) intervals".to_string()
            ));
        }

        let symbol_upper = symbol.to_uppercase();

        // Check cache first
        if let Some(cached_data) = self.cache.get(&symbol_upper, interval, start_time, end_time).await {
            info!("Using cached data for stock {}:{}", symbol_upper, interval);
            return Ok((*cached_data).clone());
        }

        // Fetch from Alpha Vantage API
        info!(
            "Fetching stock {} daily data from {} to {}",
            symbol_upper, start_time, end_time
        );

        // Wait if rate limited
        self.cache.wait_if_needed().await;

        // Check if we can make request
        if !self.cache.can_make_request(REQUEST_WEIGHT).await {
            self.cache.wait_if_needed().await;
        }

        let klines = self.fetch_daily_data(&symbol_upper).await?;

        // Record the request
        self.cache.record_request(REQUEST_WEIGHT).await;

        // Filter klines to the requested time range
        let filtered_klines: Vec<Kline> = klines
            .into_iter()
            .filter(|k| k.open_time >= start_time && k.open_time <= end_time)
            .collect();

        if filtered_klines.is_empty() {
            return Err(AppError::NotFound(format!(
                "No data found for {} between {} and {}",
                symbol_upper, start_time, end_time
            )));
        }

        // Store in cache for future requests
        self.cache
            .store(&symbol_upper, interval, start_time, end_time, filtered_klines.clone())
            .await;

        Ok(filtered_klines)
    }

    /// Fetch daily data from Alpha Vantage
    /// Returns full dataset (outputsize=full gives 20+ years of data)
    async fn fetch_daily_data(&self, symbol: &str) -> Result<Vec<Kline>, AppError> {
        let response = self.client
            .get(ALPHA_VANTAGE_BASE)
            .query(&[
                ("function", "TIME_SERIES_DAILY"),
                ("symbol", symbol),
                ("outputsize", "full"), // Get full historical data
                ("apikey", &self.api_key),
            ])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch stock data from Alpha Vantage: {}", e);
                AppError::ExternalServiceError(format!("Failed to fetch stock data: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Alpha Vantage API error: {} - {}", status, text);

            // Handle rate limiting
            if status == 429 || text.contains("rate limit") {
                return Err(AppError::RateLimitError(
                    "Alpha Vantage rate limit exceeded. Free tier allows 25 requests per day.".to_string(),
                ));
            }

            return Err(AppError::ExternalServiceError(format!(
                "Alpha Vantage API error: {} - {}",
                status, text
            )));
        }

        let time_series: AlphaVantageTimeSeries = response.json().await.map_err(|e| {
            error!("Failed to parse Alpha Vantage response: {}", e);
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        let time_series_data = time_series.time_series.ok_or_else(|| {
            AppError::ExternalServiceError("No time series data in response".to_string())
        })?;

        // Convert to Kline format
        let mut klines: Vec<Kline> = Vec::new();

        for (date_str, entry) in time_series_data {
            // Parse date (format: YYYY-MM-DD)
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| {
                    AppError::ParseError(format!("Invalid date format {}: {}", date_str, e))
                })?;

            // Create DateTime at market open (9:30 AM EST = 14:30 UTC typically)
            let open_time = date.and_hms_opt(14, 30, 0)
                .ok_or_else(|| AppError::ParseError("Invalid time".to_string()))?
                .and_utc();

            // Close time is same day at market close (4:00 PM EST = 21:00 UTC)
            let close_time = date.and_hms_opt(21, 0, 0)
                .ok_or_else(|| AppError::ParseError("Invalid time".to_string()))?
                .and_utc();

            let kline = Kline {
                open_time,
                open: Decimal::from_str_exact(&entry.open)
                    .map_err(|e| AppError::ParseError(format!("Invalid open price: {}", e)))?,
                high: Decimal::from_str_exact(&entry.high)
                    .map_err(|e| AppError::ParseError(format!("Invalid high price: {}", e)))?,
                low: Decimal::from_str_exact(&entry.low)
                    .map_err(|e| AppError::ParseError(format!("Invalid low price: {}", e)))?,
                close: Decimal::from_str_exact(&entry.close)
                    .map_err(|e| AppError::ParseError(format!("Invalid close price: {}", e)))?,
                volume: Decimal::from_str_exact(&entry.volume)
                    .map_err(|e| AppError::ParseError(format!("Invalid volume: {}", e)))?,
                close_time,
                quote_asset_volume: Decimal::ZERO, // Not provided by Alpha Vantage for daily data
                number_of_trades: 0, // Not provided
                taker_buy_base_asset_volume: Decimal::ZERO, // Not provided
                taker_buy_quote_asset_volume: Decimal::ZERO, // Not provided
            };

            klines.push(kline);
        }

        // Sort by date ascending (oldest first)
        klines.sort_by(|a, b| a.open_time.cmp(&b.open_time));

        debug!("Fetched {} daily klines for {}", klines.len(), symbol);

        Ok(klines)
    }

    /// Validate stock symbol format
    pub fn validate_symbol(symbol: &str) -> Result<(), AppError> {
        if symbol.is_empty() {
            return Err(AppError::BadRequest("Symbol cannot be empty".to_string()));
        }

        // Stock symbols should be uppercase letters (1-5 characters typically)
        if !symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            return Err(AppError::BadRequest(
                "Stock symbol should be uppercase (e.g., AAPL, TSLA)".to_string(),
            ));
        }

        if symbol.len() > 5 {
            return Err(AppError::BadRequest(
                "Stock symbol too long (max 5 characters)".to_string(),
            ));
        }

        Ok(())
    }

    /// Get common stock symbols
    pub async fn get_symbols(&self) -> Result<Vec<String>, AppError> {
        // Return common stock symbols
        Ok(vec![
            "AAPL".to_string(),   // Apple
            "MSFT".to_string(),   // Microsoft
            "GOOGL".to_string(),  // Alphabet
            "AMZN".to_string(),   // Amazon
            "TSLA".to_string(),   // Tesla
            "NVDA".to_string(),   // NVIDIA
            "META".to_string(),   // Meta
            "SPY".to_string(),    // S&P 500 ETF
            "QQQ".to_string(),    // NASDAQ ETF
            "VOO".to_string(),    // Vanguard S&P 500 ETF
        ])
    }
}
