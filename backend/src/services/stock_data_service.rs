use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error, debug};

use crate::utils::errors::AppError;

const ALPHA_VANTAGE_BASE: &str = "https://www.alphavantage.co/query";

/// Stock data service for fetching stock market data from Alpha Vantage
#[derive(Clone)]
pub struct StockDataService {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockPrice {
    pub symbol: String,
    pub price: f64,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

#[derive(Debug, Serialize)]
pub struct HistoricalStockData {
    pub symbol: String,
    pub data: Vec<HistoricalDataPoint>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageGlobalQuote {
    #[serde(rename = "Global Quote")]
    global_quote: GlobalQuoteData,
}

#[derive(Debug, Deserialize)]
struct GlobalQuoteData {
    #[serde(rename = "01. symbol")]
    symbol: String,
    #[serde(rename = "05. price")]
    price: String,
    #[serde(rename = "07. latest trading day")]
    trading_day: String,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageTimeSeries {
    #[serde(rename = "Meta Data")]
    _meta_data: HashMap<String, String>,
    #[serde(rename = "Time Series (Daily)")]
    time_series: HashMap<String, TimeSeriesEntry>,
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

impl StockDataService {
    /// Create a new stock data service
    ///
    /// # Arguments
    /// * `api_key` - Alpha Vantage API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get current stock price for a symbol
    ///
    /// # Arguments
    /// * `symbol` - Stock ticker symbol (e.g., "AAPL", "TSLA")
    pub async fn get_current_price(&self, symbol: &str) -> Result<StockPrice, AppError> {
        info!("Fetching current price for stock: {}", symbol);

        let response = self.client
            .get(ALPHA_VANTAGE_BASE)
            .query(&[
                ("function", "GLOBAL_QUOTE"),
                ("symbol", symbol),
                ("apikey", &self.api_key),
            ])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch stock price from Alpha Vantage: {}", e);
                AppError::ExternalServiceError(format!("Failed to fetch stock price: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Alpha Vantage API error: {} - {}", status, text);
            return Err(AppError::ExternalServiceError(format!(
                "Alpha Vantage API returned error: {} - {}",
                status, text
            )));
        }

        let quote: AlphaVantageGlobalQuote = response.json().await.map_err(|e| {
            error!("Failed to parse Alpha Vantage response: {}", e);
            AppError::ExternalServiceError(format!("Failed to parse response: {}", e))
        })?;

        let price: f64 = quote.global_quote.price.parse().map_err(|e| {
            AppError::ParseError(format!("Invalid price format: {}", e))
        })?;

        debug!("Stock {} current price: ${}", symbol, price);

        Ok(StockPrice {
            symbol: symbol.to_uppercase(),
            price,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Get historical daily stock data
    ///
    /// # Arguments
    /// * `symbol` - Stock ticker symbol (e.g., "AAPL", "TSLA")
    /// * `outputsize` - "compact" (100 data points) or "full" (20+ years)
    pub async fn get_historical_data(
        &self,
        symbol: &str,
        outputsize: Option<&str>,
    ) -> Result<HistoricalStockData, AppError> {
        info!("Fetching historical data for stock: {}", symbol);

        let output = outputsize.unwrap_or("compact");

        let response = self.client
            .get(ALPHA_VANTAGE_BASE)
            .query(&[
                ("function", "TIME_SERIES_DAILY"),
                ("symbol", symbol),
                ("outputsize", output),
                ("apikey", &self.api_key),
            ])
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch historical data from Alpha Vantage: {}", e);
                AppError::ExternalServiceError(format!("Failed to fetch historical data: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Alpha Vantage API error: {} - {}", status, text);
            return Err(AppError::ExternalServiceError(format!(
                "Alpha Vantage API returned error: {} - {}",
                status, text
            )));
        }

        let time_series: AlphaVantageTimeSeries = response.json().await.map_err(|e| {
            error!("Failed to parse Alpha Vantage historical data: {}", e);
            AppError::ExternalServiceError(format!("Failed to parse historical data: {}", e))
        })?;

        let mut data_points: Vec<HistoricalDataPoint> = time_series
            .time_series
            .into_iter()
            .map(|(date, entry)| {
                Ok(HistoricalDataPoint {
                    date,
                    open: entry.open.parse().map_err(|e| {
                        AppError::ParseError(format!("Invalid open price: {}", e))
                    })?,
                    high: entry.high.parse().map_err(|e| {
                        AppError::ParseError(format!("Invalid high price: {}", e))
                    })?,
                    low: entry.low.parse().map_err(|e| {
                        AppError::ParseError(format!("Invalid low price: {}", e))
                    })?,
                    close: entry.close.parse().map_err(|e| {
                        AppError::ParseError(format!("Invalid close price: {}", e))
                    })?,
                    volume: entry.volume.parse().map_err(|e| {
                        AppError::ParseError(format!("Invalid volume: {}", e))
                    })?,
                })
            })
            .collect::<Result<Vec<_>, AppError>>()?;

        // Sort by date descending (newest first)
        data_points.sort_by(|a, b| b.date.cmp(&a.date));

        debug!("Retrieved {} historical data points for {}", data_points.len(), symbol);

        Ok(HistoricalStockData {
            symbol: symbol.to_uppercase(),
            data: data_points,
        })
    }
}
