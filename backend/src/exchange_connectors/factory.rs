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

}