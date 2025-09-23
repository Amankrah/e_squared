use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::{info, error, debug, warn};
use std::collections::HashMap;
use serde_json::Value;

use crate::exchange_connectors::{
    traits::{ExchangeConnector, AccountAPI, OrderAPI, TradeExecutionAPI, MarketDataAPI},
    types::*,
    ExchangeCredentials,
    ExchangeError,
};

use super::api_client::BinanceApiClient;
use super::converters::*;

pub struct BinanceConnector {
    client: BinanceApiClient,
}

impl BinanceConnector {
    pub fn new(credentials: ExchangeCredentials) -> Result<Self, ExchangeError> {
        let client = BinanceApiClient::new(credentials)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ExchangeConnector for BinanceConnector {
    fn name(&self) -> &'static str {
        "Binance"
    }

    async fn test_connection(&self) -> Result<bool, ExchangeError> {
        // First test basic connectivity
        match self.client.test_connectivity().await {
            Ok(false) => return Ok(false),
            Err(e) => {
                error!("Basic connectivity test failed: {}", e);
                return Ok(false);
            }
            _ => {}
        }

        // Then test API credentials by calling an authenticated endpoint
        // Use account endpoint which requires valid API key and permissions
        let params = HashMap::new();
        match self.client.signed_request("account", &params).await {
            Ok(_) => {
                info!("API credentials validated successfully");
                Ok(true)
            }
            Err(e) => {
                error!("API credential validation failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_server_time(&self) -> Result<DateTime<Utc>, ExchangeError> {
        self.client.get_server_time().await
    }
}

#[async_trait]
impl AccountAPI for BinanceConnector {
    async fn get_spot_account(&self) -> Result<SpotAccount, ExchangeError> {
        let params = HashMap::new();
        let response = self.client.signed_request("account", &params).await?;
        parse_spot_account_from_json_with_prices(response, &self.client).await
    }

    async fn get_margin_account(&self) -> Result<MarginAccount, ExchangeError> {
        tracing::debug!("Attempting to fetch margin account...");
        let params = HashMap::new();

        match self.client.signed_request("margin/account", &params).await {
            Ok(response) => {
                tracing::debug!("Margin account response received, parsing...");
                parse_margin_account_from_json_with_prices(response, &self.client).await
            }
            Err(e) => {
                tracing::info!("Margin account request failed: {:?}", e);
                match &e {
                    ExchangeError::ApiError(msg) if msg.contains("404") => {
                        tracing::info!("Margin trading not enabled on this account (404 error)");
                    }
                    ExchangeError::ApiError(msg) if msg.contains("401") => {
                        tracing::warn!("API key lacks margin trading permissions (401 error)");
                    }
                    _ => {
                        tracing::warn!("Unexpected margin account error: {:?}", e);
                    }
                }
                Err(e)
            }
        }
    }

    async fn get_futures_account(&self, account_type: FuturesType) -> Result<FuturesAccount, ExchangeError> {
        match account_type {
            FuturesType::USDM => {
                // USD-M Futures account
                let params = HashMap::new();
                let response = self.client.signed_request("fapi/v2/account", &params).await?;
                parse_futures_account_from_json_with_prices(response, &self.client, account_type).await
            }
            FuturesType::COINM => {
                // COIN-M Futures account  
                let params = HashMap::new();
                let response = self.client.signed_request("dapi/v1/account", &params).await?;
                parse_futures_account_from_json_with_prices(response, &self.client, account_type).await
            }
        }
    }

    async fn get_all_balances(&self) -> Result<AccountBalances, ExchangeError> {
        // Fetch all account types in parallel for better performance
        let spot_future = self.get_spot_account();
        let margin_future = self.get_margin_account();
        let isolated_margin_future = self.get_isolated_margin_accounts();
        let futures_usdm_future = self.get_futures_account(FuturesType::USDM);
        let futures_coinm_future = self.get_futures_account(FuturesType::COINM);
        let earn_future = self.get_savings_balances();

        // Fetch spot account (required)
        let spot = match spot_future.await {
            Ok(account) => Some(account),
            Err(e) => {
                error!("Failed to fetch spot account: {:?}", e);
                return Err(e);
            }
        };

        // Fetch margin account (optional - may fail if not enabled)
        let margin = match margin_future.await {
            Ok(account) => Some(account),
            Err(e) => {
                info!("Margin account not available or disabled: {:?}", e);
                None
            }
        };

        // Fetch isolated margin accounts (optional)
        let isolated_margin = match isolated_margin_future.await {
            Ok(accounts) if !accounts.is_empty() => Some(accounts),
            Ok(_) => None,
            Err(e) => {
                info!("Isolated margin accounts not available: {:?}", e);
                None
            }
        };

        // Fetch futures accounts (optional)
        let futures_usdm = match futures_usdm_future.await {
            Ok(account) => Some(account),
            Err(e) => {
                info!("USD-M futures account not available: {:?}", e);
                None
            }
        };

        let futures_coinm = match futures_coinm_future.await {
            Ok(account) => Some(account),
            Err(e) => {
                info!("COIN-M futures account not available: {:?}", e);
                None
            }
        };

        // Fetch Savings/Earn balances (optional)
        let earn = match earn_future.await {
            Ok(balances) if !balances.is_empty() => Some(balances),
            Ok(_) => None,
            Err(e) => {
                info!("Earn/Savings balances not available: {:?}", e);
                None
            }
        };

        // Calculate total USD and BTC values
        let mut total_usd_value = Decimal::ZERO;
        let mut total_btc_value = Decimal::ZERO;

        if let Some(ref account) = spot {
            total_usd_value += account.total_usd_value.unwrap_or(Decimal::ZERO);
            total_btc_value += account.total_btc_value.unwrap_or(Decimal::ZERO);
        }

        if let Some(ref account) = margin {
            total_usd_value += account.total_net_value;
        }

        if let Some(ref accounts) = isolated_margin {
            for account in accounts {
                total_usd_value += account.total_net_value;
            }
        }

        if let Some(ref account) = futures_usdm {
            total_usd_value += account.total_margin_balance;
        }

        if let Some(ref account) = futures_coinm {
            total_usd_value += account.total_margin_balance;
        }

        if let Some(ref balances) = earn {
            for balance in balances {
                if let Some(usd_value) = balance.usd_value {
                    total_usd_value += usd_value;
                }
            }
        }

        Ok(AccountBalances {
            spot,
            margin,
            isolated_margin,
            futures_usdm,
            futures_coinm,
            earn,
            total_usd_value,
            total_btc_value,
        })
    }

    async fn get_asset_balance(&self, asset: &str, wallet_type: WalletType) -> Result<AssetBalance, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                let account = self.get_spot_account().await?;
                account.balances.into_iter()
                    .find(|b| b.asset == asset)
                    .ok_or_else(|| ExchangeError::SymbolNotFound(format!("Asset {} not found", asset)))
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }
}

impl BinanceConnector {
    // Helper method to get isolated margin accounts
    async fn get_isolated_margin_accounts(&self) -> Result<Vec<IsolatedMarginAccount>, ExchangeError> {
        let params = HashMap::new();
        let response = self.client.signed_request("sapi/v1/margin/isolated/account", &params).await?;
        super::converters::parse_isolated_margin_accounts_from_json(response, &self.client).await
    }

    // Helper method to get Savings/Earn balances
    async fn get_savings_balances(&self) -> Result<Vec<AssetBalance>, ExchangeError> {
        let mut all_balances = Vec::new();

        // Get Flexible Savings (Simple Earn)
        let flexible_params = HashMap::new();
        match self.client.signed_request("sapi/v1/simple-earn/flexible/position", &flexible_params).await {
            Ok(response) => {
                if let Some(rows) = response.get("rows").and_then(|v| v.as_array()) {
                    for position in rows {
                        let asset = position.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
                        let total_amount_str = position.get("totalAmount").and_then(|v| v.as_str()).unwrap_or("0");
                        
                        let total_amount = parse_decimal(total_amount_str)?;
                        
                        if total_amount > Decimal::ZERO {
                            // Get USD value
                            let mut usd_value = None;
                            if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                                usd_value = Some(total_amount);
                            } else {
                                match self.client.get_symbol_price(asset).await {
                                    Ok(price) => {
                                        usd_value = Some(total_amount * price);
                                        debug!("Savings - Got Binance price for {}: ${}, value: ${}", asset, price, total_amount * price);
                                    }
                                    Err(e) => {
                                        warn!("Failed to get Binance price for savings asset {}: {:?}", asset, e);
                                    }
                                }
                            }

                            all_balances.push(AssetBalance {
                                asset: asset.to_string(),
                                free: total_amount,
                                locked: Decimal::ZERO,
                                total: total_amount,
                                usd_value,
                                btc_value: None,
                                wallet_type: WalletType::Earn,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                info!("Flexible savings not available: {:?}", e);
            }
        }

        // Get Locked Savings (Simple Earn)
        let locked_params = HashMap::new();
        match self.client.signed_request("sapi/v1/simple-earn/locked/position", &locked_params).await {
            Ok(response) => {
                if let Some(rows) = response.get("rows").and_then(|v| v.as_array()) {
                    for position in rows {
                        let asset = position.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
                        let amount_str = position.get("amount").and_then(|v| v.as_str()).unwrap_or("0");
                        
                        let amount = parse_decimal(amount_str)?;
                        
                        if amount > Decimal::ZERO {
                            // Get USD value
                            let mut usd_value = None;
                            if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                                usd_value = Some(amount);
                            } else {
                                match self.client.get_symbol_price(asset).await {
                                    Ok(price) => {
                                        usd_value = Some(amount * price);
                                        debug!("Locked Savings - Got Binance price for {}: ${}, value: ${}", asset, price, amount * price);
                                    }
                                    Err(e) => {
                                        warn!("Failed to get Binance price for locked savings asset {}: {:?}", asset, e);
                                    }
                                }
                            }

                            all_balances.push(AssetBalance {
                                asset: asset.to_string(),
                                free: Decimal::ZERO,
                                locked: amount,
                                total: amount,
                                usd_value,
                                btc_value: None,
                                wallet_type: WalletType::Earn,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                info!("Locked savings not available: {:?}", e);
            }
        }

        Ok(all_balances)
    }

    async fn get_asset_balance(&self, asset: &str, wallet_type: WalletType) -> Result<AssetBalance, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                let account = self.get_spot_account().await?;
                account.balances.into_iter()
                    .find(|b| b.asset == asset)
                    .ok_or_else(|| ExchangeError::SymbolNotFound(format!("Asset {} not found", asset)))
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }
}

#[async_trait]
impl OrderAPI for BinanceConnector {
    async fn get_open_orders(&self, symbol: Option<&str>, wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                let mut params = HashMap::new();
                if let Some(sym) = symbol {
                    params.insert("symbol".to_string(), sym.to_string());
                }

                let response = self.client.signed_request("openOrders", &params).await?;
                parse_orders_from_json(response, wallet_type)
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }

    async fn get_order(&self, order_id: &str, symbol: &str, wallet_type: WalletType) -> Result<Order, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.client.signed_request("order", &params).await?;
                parse_single_order_from_json(response, wallet_type)
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }

    async fn get_order_history(
        &self,
        symbol: Option<&str>,
        wallet_type: WalletType,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>
    ) -> Result<Vec<Order>, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                let mut params = HashMap::new();
                if let Some(sym) = symbol {
                    params.insert("symbol".to_string(), sym.to_string());
                } else {
                    return Err(ExchangeError::InvalidParameter("Symbol required for order history".to_string()));
                }

                if let Some(start) = start_time {
                    params.insert("startTime".to_string(), start.timestamp_millis().to_string());
                }
                if let Some(end) = end_time {
                    params.insert("endTime".to_string(), end.timestamp_millis().to_string());
                }
                if let Some(lim) = limit {
                    params.insert("limit".to_string(), lim.to_string());
                }

                let response = self.client.signed_request("allOrders", &params).await?;
                parse_orders_from_json(response, wallet_type)
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }

    async fn cancel_order(&self, order_id: &str, symbol: &str, wallet_type: WalletType) -> Result<Order, ExchangeError> {
        match wallet_type {
            WalletType::Spot => {
                Err(ExchangeError::NotSupported("Order cancellation not yet implemented".to_string()))
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported", wallet_type)))
        }
    }

    async fn cancel_all_orders(&self, symbol: Option<&str>, wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError> {
        Err(ExchangeError::NotSupported("Bulk order cancellation not yet implemented".to_string()))
    }
}

#[async_trait]
impl TradeExecutionAPI for BinanceConnector {
    async fn place_market_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _quantity: Option<Decimal>,
        _quote_quantity: Option<Decimal>,
        _wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        Err(ExchangeError::NotSupported("Market orders not yet implemented".to_string()))
    }

    async fn place_limit_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _price: Decimal,
        _quantity: Decimal,
        _time_in_force: TimeInForce,
        _wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        Err(ExchangeError::NotSupported("Limit orders not yet implemented".to_string()))
    }

    async fn place_stop_loss_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _stop_price: Decimal,
        _quantity: Decimal,
        _limit_price: Option<Decimal>,
        _wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        Err(ExchangeError::NotSupported("Stop loss orders not yet implemented".to_string()))
    }

    async fn place_take_profit_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _stop_price: Decimal,
        _quantity: Decimal,
        _limit_price: Option<Decimal>,
        _wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        Err(ExchangeError::NotSupported("Take profit orders not yet implemented".to_string()))
    }

    async fn place_oco_order(
        &self,
        _symbol: &str,
        _side: OrderSide,
        _quantity: Decimal,
        _price: Decimal,
        _stop_price: Decimal,
        _stop_limit_price: Option<Decimal>,
        _wallet_type: WalletType,
    ) -> Result<OcoOrder, ExchangeError> {
        Err(ExchangeError::NotSupported("OCO orders not yet implemented".to_string()))
    }
}

#[async_trait]
impl MarketDataAPI for BinanceConnector {
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError> {
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.client.spot_base_url, symbol);
        let response = self.client.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        parse_ticker_from_json(json, symbol)
    }

    async fn get_order_book(&self, symbol: &str, limit: Option<u32>) -> Result<OrderBook, ExchangeError> {
        let limit_param = limit.unwrap_or(100);
        let url = format!("{}/api/v3/depth?symbol={}&limit={}", self.client.spot_base_url, symbol, limit_param);
        let response = self.client.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        parse_order_book_from_json(json, symbol)
    }

    async fn get_recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>, ExchangeError> {
        let limit_param = limit.unwrap_or(500);
        let url = format!("{}/api/v3/trades?symbol={}&limit={}", self.client.spot_base_url, symbol, limit_param);
        let response = self.client.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        parse_trades_from_json(json, symbol)
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = convert_kline_interval_to_string(&interval);
        let mut url = format!("{}/api/v3/klines?symbol={}&interval={}", self.client.spot_base_url, symbol, interval_str);

        if let Some(start) = start_time {
            url.push_str(&format!("&startTime={}", start.timestamp_millis()));
        }
        if let Some(end) = end_time {
            url.push_str(&format!("&endTime={}", end.timestamp_millis()));
        }
        if let Some(lim) = limit {
            url.push_str(&format!("&limit={}", lim));
        }

        let response = self.client.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        parse_klines_from_json(json)
    }

    async fn get_exchange_info(&self) -> Result<ExchangeInfo, ExchangeError> {
        let url = format!("{}/api/v3/exchangeInfo", self.client.spot_base_url);
        let response = self.client.client.get(&url).send().await?;
        let json: Value = response.json().await?;
        parse_exchange_info_from_json(json)
    }

    async fn get_symbol_info(&self, symbol: &str) -> Result<SymbolInfo, ExchangeError> {
        let info = self.get_exchange_info().await?;
        info.symbols.into_iter()
            .find(|s| s.symbol == symbol)
            .ok_or_else(|| ExchangeError::SymbolNotFound(format!("Symbol {} not found", symbol)))
    }
}