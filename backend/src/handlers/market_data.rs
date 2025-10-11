use actix_web::{web, HttpResponse};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{info, error, debug};

use crate::utils::errors::AppError;

const BINANCE_API_BASE: &str = "https://api.binance.com";

#[derive(Debug, Serialize)]
pub struct CurrentPriceResponse {
    pub symbol: String,
    pub price: f64,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct BinanceTickerPrice {
    symbol: String,
    price: String,
}

/// Convert symbol to Binance trading pair (e.g., BTC -> BTCUSDT)
fn convert_to_trading_pair(symbol: &str) -> String {
    let symbol_upper = symbol.to_uppercase();

    // If already contains USDT, return as-is
    if symbol_upper.ends_with("USDT") {
        return symbol_upper;
    }

    // Otherwise append USDT
    format!("{}USDT", symbol_upper)
}

/// Get current price for a symbol from Binance
pub async fn get_current_price(
    symbol: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let symbol = symbol.into_inner();

    info!("Fetching current price for {}", symbol);

    let trading_pair = convert_to_trading_pair(&symbol);
    let url = format!("{}/api/v3/ticker/price", BINANCE_API_BASE);

    let client = Client::new();

    debug!("Requesting price for trading pair: {}", trading_pair);

    let response = client
        .get(&url)
        .query(&[("symbol", &trading_pair)])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Binance: {}", e);
            AppError::ExternalServiceError(format!("Failed to fetch price from Binance: {}", e))
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        error!("Binance API error: {} - {}", status, text);
        return Err(AppError::ExternalServiceError(format!(
            "Binance API returned error: {} - {}",
            status, text
        )));
    }

    let ticker: BinanceTickerPrice = response.json().await.map_err(|e| {
        error!("Failed to parse Binance response: {}", e);
        AppError::ExternalServiceError(format!("Failed to parse Binance response: {}", e))
    })?;

    let price_decimal = Decimal::from_str(&ticker.price)
        .map_err(|e| AppError::ParseError(format!("Invalid price format: {}", e)))?;

    let price_f64: f64 = price_decimal.to_string().parse()
        .map_err(|e| AppError::ParseError(format!("Failed to convert price: {}", e)))?;

    let response = CurrentPriceResponse {
        symbol: symbol.clone(),
        price: price_f64,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    info!("Current price for {}: ${}", symbol, price_f64);

    Ok(HttpResponse::Ok().json(response))
}
