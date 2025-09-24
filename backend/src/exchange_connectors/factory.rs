use std::sync::Arc;
use crate::exchange_connectors::{
    Exchange,
    ExchangeCredentials,
    ExchangeError,
    traits::{ExchangeConnector, AccountAPI, OrderAPI, TradeExecutionAPI, MarketDataAPI},
    binance::BinanceConnector,
};

pub trait FullExchangeAPI: ExchangeConnector + AccountAPI + OrderAPI + TradeExecutionAPI + MarketDataAPI {}

impl<T> FullExchangeAPI for T where T: ExchangeConnector + AccountAPI + OrderAPI + TradeExecutionAPI + MarketDataAPI {}

pub struct ExchangeFactory;

impl ExchangeFactory {
    pub fn create(
        exchange: Exchange,
        credentials: ExchangeCredentials,
    ) -> Result<Arc<dyn FullExchangeAPI>, ExchangeError> {
        match exchange {
            Exchange::Binance => {
                let connector = BinanceConnector::new(credentials)?;
                Ok(Arc::new(connector))
            }
            Exchange::Bybit => {
                Err(ExchangeError::NotSupported("Bybit connector not yet implemented".to_string()))
            }
            Exchange::Coinbase => {
                Err(ExchangeError::NotSupported("Coinbase connector not yet implemented".to_string()))
            }
            Exchange::Kraken => {
                Err(ExchangeError::NotSupported("Kraken connector not yet implemented".to_string()))
            }
            Exchange::Kucoin => {
                Err(ExchangeError::NotSupported("Kucoin connector not yet implemented".to_string()))
            }
            Exchange::OKX => {
                Err(ExchangeError::NotSupported("OKX connector not yet implemented".to_string()))
            }
        }
    }

    pub fn create_with_optional_credentials(
        exchange: Exchange,
        api_key: Option<String>,
        api_secret: Option<String>,
    ) -> Result<Arc<dyn FullExchangeAPI>, ExchangeError> {
        let credentials = if let (Some(key), Some(secret)) = (api_key, api_secret) {
            ExchangeCredentials {
                api_key: key,
                api_secret: secret,
            }
        } else {
            return Err(ExchangeError::InvalidApiKey);
        };

        Self::create(exchange, credentials)
    }

    pub fn supported_exchanges() -> Vec<Exchange> {
        vec![Exchange::Binance]
    }

    pub fn is_supported(exchange: &Exchange) -> bool {
        matches!(exchange, Exchange::Binance)
    }
}