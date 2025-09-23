pub mod traits;
pub mod types;
pub mod binance;
pub mod factory;
pub mod errors;

pub use traits::*;
pub use types::*;
pub use factory::ExchangeFactory;
pub use errors::ExchangeError;

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
        matches!(self, Exchange::Kucoin | Exchange::OKX)
    }
}