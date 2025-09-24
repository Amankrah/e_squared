pub mod traits;
pub mod binance;
pub mod factory;
pub mod errors;
pub mod shared_types;
pub mod common_types;

use serde::{Deserialize, Serialize};

pub use factory::ExchangeFactory;
pub use errors::ExchangeError;
pub use shared_types::*;
pub use common_types::*;

/// Simplified exchange credentials - only API key and secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCredentials {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Exchange {
    Binance,
    Bybit,
    Coinbase,
    Kraken,
    Kucoin,
    OKX,
}

impl Exchange {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "binance" => Some(Exchange::Binance),
            "bybit" => Some(Exchange::Bybit),
            "coinbase" => Some(Exchange::Coinbase),
            "kraken" => Some(Exchange::Kraken),
            "kucoin" => Some(Exchange::Kucoin),
            "okx" => Some(Exchange::OKX),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Exchange::Binance => "binance",
            Exchange::Bybit => "bybit",
            Exchange::Coinbase => "coinbase",
            Exchange::Kraken => "kraken",
            Exchange::Kucoin => "kucoin",
            Exchange::OKX => "okx",
        }
    }

    pub fn requires_passphrase(&self) -> bool {
        // Most exchanges only require API key and secret
        // Future exchanges that need passphrase can be added here
        false
    }
}