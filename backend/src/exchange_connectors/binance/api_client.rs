use reqwest::Client;
use serde_json::Value;
use chrono::{ Utc};
use std::collections::HashMap;
use std::str::FromStr;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use rust_decimal::Decimal;
use crate::exchange_connectors::{ExchangeCredentials, ExchangeError};

type HmacSha256 = Hmac<Sha256>;

pub struct BinanceApiClient {
    pub client: Client,
    pub spot_base_url: String,
    pub futures_base_url: String,
    credentials: ExchangeCredentials,
}

impl BinanceApiClient {
    pub fn new(credentials: ExchangeCredentials) -> Result<Self, ExchangeError> {
        let client = Client::new();

        Ok(Self {
            client,
            spot_base_url: "https://api.binance.com".to_string(),
            futures_base_url: "https://fapi.binance.com".to_string(),
            credentials,
        })
    }


    pub async fn test_connectivity(&self) -> Result<bool, ExchangeError> {
        let url = format!("{}/api/v3/ping", self.spot_base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    fn create_signature(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.credentials.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub async fn get_symbol_price(&self, symbol: &str) -> Result<Decimal, ExchangeError> {
        // Handle stablecoins
        if matches!(symbol.to_uppercase().as_str(), "USDT" | "USDC" | "BUSD" | "DAI" | "FDUSD") {
            return Ok(Decimal::from(1));
        }

        // Try different trading pairs
        let pairs = vec![
            format!("{}USDT", symbol.to_uppercase()),
            format!("{}USDC", symbol.to_uppercase()),
            format!("{}BUSD", symbol.to_uppercase()),
        ];

        for pair in pairs {
            let url = format!("{}/api/v3/ticker/price?symbol={}", self.spot_base_url, pair);

            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(json) = response.json::<Value>().await {
                        if let Some(price_str) = json.get("price").and_then(|v| v.as_str()) {
                            if let Ok(price) = Decimal::from_str(price_str) {
                                return Ok(price);
                            }
                        }
                    }
                }
            }
        }

        // If we can't find a direct pair, return zero (no price data)
        Ok(Decimal::ZERO)
    }

    pub async fn signed_request(&self, endpoint: &str, params: &HashMap<String, String>) -> Result<Value, ExchangeError> {
        let mut query_params = params.clone();
        query_params.insert("timestamp".to_string(), Utc::now().timestamp_millis().to_string());

        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let signature = self.create_signature(&query_string);
        let final_query = format!("{}&signature={}", query_string, signature);

        // Determine the base URL based on the endpoint
        let base_url = if endpoint.starts_with("fapi/") {
            self.futures_base_url.clone()
        } else if endpoint.starts_with("dapi/") {
            "https://dapi.binance.com".to_string() // COIN-M futures
        } else if endpoint.starts_with("sapi/") {
            self.spot_base_url.clone() // SAPI endpoints use spot base URL
        } else {
            self.spot_base_url.clone()
        };

        // Build URL based on endpoint type
        let url = if endpoint.starts_with("fapi/") || endpoint.starts_with("dapi/") {
            format!("{}/{}", base_url, endpoint)
        } else if endpoint.starts_with("sapi/") {
            format!("{}/{}", base_url, endpoint)
        } else {
            format!("{}/api/v3/{}", base_url, endpoint)
        };

        let response = self.client
            .get(&format!("{}?{}", url, final_query))
            .header("X-MBX-APIKEY", &self.credentials.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(self.parse_binance_error(status.as_u16(), &error_text));
        }

        let json: Value = response.json().await?;
        Ok(json)
    }

    fn parse_binance_error(&self, status_code: u16, error_text: &str) -> ExchangeError {
        // Try to parse error_text as JSON to get Binance error codes
        if let Ok(error_json) = serde_json::from_str::<Value>(error_text) {
            if let Some(error_code) = error_json.get("code").and_then(|c| c.as_i64()) {
                let msg = error_json.get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                return match error_code {
                    // Authentication errors
                    -1022 | -2014 => ExchangeError::InvalidApiKey,
                    -1021 => ExchangeError::AuthenticationError(format!("Timestamp outside of recvWindow: {}", msg)),
                    -2015 => ExchangeError::AuthenticationError(format!("Invalid API key format: {}", msg)),
                    
                    // Order related errors
                    -1013 => ExchangeError::InvalidOrder(format!("Invalid quantity: {}", msg)),
                    -1102 => ExchangeError::InvalidParameter(format!("Mandatory parameter missing: {}", msg)),
                    -2010 => ExchangeError::InvalidOrder(format!("New order rejected: {}", msg)),
                    -2011 => ExchangeError::OrderNotFound(format!("Order not found: {}", msg)),
                    
                    // Balance related errors
                    -2019 => ExchangeError::InsufficientBalance(format!("Margin is insufficient: {}", msg)),
                    
                    // Symbol/market errors
                    -1121 => ExchangeError::SymbolNotFound(format!("Invalid symbol: {}", msg)),
                    -1100 => ExchangeError::InvalidParameter(format!("Illegal characters in parameter: {}", msg)),
                    
                    // Rate limiting or balance (error code -1003 can mean both)
                    -1003 => {
                        if msg.to_lowercase().contains("rate") || msg.to_lowercase().contains("limit") {
                            ExchangeError::RateLimitExceeded(format!("Too many requests: {}", msg))
                        } else {
                            ExchangeError::InsufficientBalance(format!("Balance insufficient: {}", msg))
                        }
                    },
                    
                    // Default to ApiError for other codes
                    _ => ExchangeError::ApiError(format!("Binance error {}: {}", error_code, msg))
                };
            }
        }

        // Handle by HTTP status code when JSON parsing fails
        match status_code {
            401 | 403 => ExchangeError::AuthenticationError(format!("Authentication failed: {}", error_text)),
            429 => ExchangeError::RateLimitExceeded(format!("Rate limit exceeded: {}", error_text)),
            503 => ExchangeError::Maintenance,
            404 => ExchangeError::Unknown(format!("Endpoint not found: {}", error_text)),
            400 => ExchangeError::InvalidParameter(format!("Bad request: {}", error_text)),
            _ => ExchangeError::Unknown(format!("HTTP {}: {}", status_code, error_text))
        }
    }
}