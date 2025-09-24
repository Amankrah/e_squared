use async_trait::async_trait;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use crate::exchange_connectors::{
    shared_types::*,
    common_types::*,
    ExchangeError,
};

#[async_trait]
pub trait ExchangeConnector: Send + Sync {
    fn name(&self) -> &'static str;

    async fn test_connection(&self) -> Result<bool, ExchangeError>;

    async fn get_server_time(&self) -> Result<DateTime<Utc>, ExchangeError>;
}

#[async_trait]
pub trait AccountAPI: ExchangeConnector {
    async fn get_spot_account(&self) -> Result<SpotAccount, ExchangeError>;

    async fn get_margin_account(&self) -> Result<MarginAccount, ExchangeError>;

    async fn get_futures_account(&self, account_type: FuturesType) -> Result<FuturesAccount, ExchangeError>;

    async fn get_all_balances(&self) -> Result<AccountBalances, ExchangeError>;

    async fn get_asset_balance(&self, asset: &str, wallet_type: WalletType) -> Result<AssetBalance, ExchangeError>;
}

#[async_trait]
pub trait OrderAPI: ExchangeConnector {
    async fn get_open_orders(&self, symbol: Option<&str>, wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError>;

    async fn get_order(&self, order_id: &str, symbol: &str, wallet_type: WalletType) -> Result<Order, ExchangeError>;

    async fn get_order_history(
        &self,
        symbol: Option<&str>,
        wallet_type: WalletType,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>
    ) -> Result<Vec<Order>, ExchangeError>;

    async fn cancel_order(&self, order_id: &str, symbol: &str, wallet_type: WalletType) -> Result<Order, ExchangeError>;

    async fn cancel_all_orders(&self, symbol: Option<&str>, wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError>;
}

#[async_trait]
pub trait TradeExecutionAPI: ExchangeConnector {
    async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: Option<Decimal>,
        quote_quantity: Option<Decimal>,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError>;

    async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        time_in_force: TimeInForce,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError>;

    async fn place_stop_loss_order(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: Decimal,
        quantity: Decimal,
        limit_price: Option<Decimal>,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError>;

    async fn place_take_profit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        stop_price: Decimal,
        quantity: Decimal,
        limit_price: Option<Decimal>,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError>;

    async fn place_oco_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
        stop_price: Decimal,
        stop_limit_price: Option<Decimal>,
        wallet_type: WalletType,
    ) -> Result<OcoOrder, ExchangeError>;
}

#[async_trait]
pub trait MarketDataAPI: ExchangeConnector {
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError>;

    async fn get_order_book(&self, symbol: &str, limit: Option<u32>) -> Result<OrderBook, ExchangeError>;

    async fn get_recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>, ExchangeError>;

    async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>
    ) -> Result<Vec<Kline>, ExchangeError>;

    async fn get_exchange_info(&self) -> Result<ExchangeInfo, ExchangeError>;

    async fn get_symbol_info(&self, symbol: &str) -> Result<SymbolInfo, ExchangeError>;
}