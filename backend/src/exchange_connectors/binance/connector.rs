use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::{info, error, debug, warn};
use std::collections::HashMap;
use serde_json::Value;

use crate::exchange_connectors::{
    traits::{ExchangeConnector, AccountAPI, OrderAPI, TradeExecutionAPI, MarketDataAPI},
    ExchangeCredentials,
    ExchangeError,
    common_types::{SpotAccount, MarginAccount, FuturesAccount, AccountBalances, AssetBalance, WalletType, FuturesType, OrderSide, TimeInForce, Order, OcoOrder},
    shared_types::{Ticker, OrderBook, Trade, Kline, KlineInterval, ExchangeInfo, SymbolInfo},
};
use super::types::*;

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
    async fn test_connection(&self) -> Result<bool, ExchangeError> {
        // First check exchange status (maintenance mode, etc.)
        let url = format!("{}/api/v3/exchangeInfo", self.client.spot_base_url);
        match self.client.client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    let status_code = response.status().as_u16();
                    match status_code {
                        503 => return Err(ExchangeError::Maintenance),
                        _ => return Err(ExchangeError::Unknown(format!("Exchange unavailable: {}", status_code)))
                    }
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    return Err(ExchangeError::Timeout);
                } else {
                    return Err(ExchangeError::NetworkError(format!("Exchange unreachable: {}", e)));
                }
            }
        }

        // Test basic connectivity
        match self.client.test_connectivity().await {
            Ok(false) => return Ok(false),
            Err(e) => {
                error!("Basic connectivity test failed: {}", e);
                return Ok(false);
            }
            _ => {}
        }

        // Then test API credentials by calling an authenticated endpoint
        let params = HashMap::new();
        match self.client.signed_request("account", &params).await {
            Ok(_) => {
                info!("API credentials validated successfully");
                Ok(true)
            }
            Err(ExchangeError::AuthenticationError(_)) | 
            Err(ExchangeError::InvalidApiKey) => {
                error!("API credential validation failed: Invalid credentials");
                Ok(false)
            }
            Err(e) => {
                error!("API credential validation failed: {}", e);
                Ok(false)
            }
        }
    }

}

#[async_trait]
impl AccountAPI for BinanceConnector {
    async fn get_spot_account(&self) -> Result<SpotAccount, ExchangeError> {
        let params = HashMap::new();
        let response = self.client.signed_request("account", &params).await?;
        let binance_account = parse_spot_account_from_json_with_prices(response, &self.client).await?;
        Ok(binance_account.into())
    }

    async fn get_margin_account(&self) -> Result<MarginAccount, ExchangeError> {
        tracing::debug!("Attempting to fetch margin account...");
        let params = HashMap::new();

        match self.client.signed_request("margin/account", &params).await {
            Ok(response) => {
                tracing::debug!("Margin account response received, parsing...");
                let binance_account = parse_margin_account_from_json_with_prices(response, &self.client).await?;
                Ok(binance_account.into())
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
        let binance_type: BinanceFuturesType = account_type.into();
        match binance_type {
            BinanceFuturesType::USDM => {
                // USD-M Futures account
                let params = HashMap::new();
                let response = self.client.signed_request("fapi/v2/account", &params).await?;
                let binance_account = parse_futures_account_from_json_with_prices(response, &self.client, binance_type).await?;
                Ok(binance_account.into())
            }
            BinanceFuturesType::COINM => {
                // COIN-M Futures account  
                let params = HashMap::new();
                let response = self.client.signed_request("dapi/v1/account", &params).await?;
                let binance_account = parse_futures_account_from_json_with_prices(response, &self.client, binance_type).await?;
                Ok(binance_account.into())
            }
        }
    }

    async fn get_all_balances(&self) -> Result<AccountBalances, ExchangeError> {
        // Fetch all account types in parallel for better performance
        let spot_future = self.get_spot_account();
        let margin_future = self.get_margin_account();
        // Note: isolated margin not included in common AccountBalances type
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

        // Skip isolated margin as it's not in common AccountBalances

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

        // Skip isolated margin totals

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
            futures_usdm,
            futures_coinm,
        total_usd_value,
        total_btc_value,
    })
}
}

impl BinanceConnector {
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
                                wallet_type: WalletType::Spot, // Map Earn to Spot for compatibility
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
                                wallet_type: WalletType::Spot, // Map Earn to Spot for compatibility
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
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());
                params.insert("orderId".to_string(), order_id.to_string());

                match self.client.signed_request("order", &params).await {
                    Ok(response) => parse_single_order_from_json(response, wallet_type),
                    Err(ExchangeError::ApiError(msg)) if msg.contains("-2011") => {
                        Err(ExchangeError::OrderNotFound(format!("Order {} not found for symbol {}", order_id, symbol)))
                    }
                    Err(e) => Err(e)
                }
            }
            _ => Err(ExchangeError::NotSupported(format!("{:?} wallet not supported for order cancellation", wallet_type)))
        }
    }

    async fn cancel_all_orders(&self, _symbol: Option<&str>, _wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError> {
        Err(ExchangeError::NotSupported("Bulk order cancellation not yet implemented".to_string()))
    }
}

#[async_trait]
impl TradeExecutionAPI for BinanceConnector {
    async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: Option<Decimal>,
        quote_quantity: Option<Decimal>,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        if wallet_type != WalletType::Spot {
            return Err(ExchangeError::NotSupported(format!("{:?} wallet not supported for market orders", wallet_type)));
        }

        // Validate trading symbol exists (financial safety check)
        self.get_symbol_info(symbol).await?;

        // Validate order parameters
        if quantity.is_none() && quote_quantity.is_none() {
            return Err(ExchangeError::InvalidOrder("Either quantity or quote_quantity must be specified".to_string()));
        }

        if quantity.is_some() && quote_quantity.is_some() {
            return Err(ExchangeError::InvalidOrder("Cannot specify both quantity and quote_quantity".to_string()));
        }

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        });
        params.insert("type".to_string(), "MARKET".to_string());

        if let Some(qty) = quantity {
            if qty <= Decimal::ZERO {
                return Err(ExchangeError::InvalidOrder("Quantity must be greater than zero".to_string()));
            }
            params.insert("quantity".to_string(), qty.to_string());
        }

        if let Some(quote_qty) = quote_quantity {
            if quote_qty <= Decimal::ZERO {
                return Err(ExchangeError::InvalidOrder("Quote quantity must be greater than zero".to_string()));
            }
            params.insert("quoteOrderQty".to_string(), quote_qty.to_string());
        }

        // Check balance before placing order
        if side == OrderSide::Sell {
            if let Some(qty) = quantity {
                let base_asset = symbol.trim_end_matches("USDT").trim_end_matches("USDC").trim_end_matches("BUSD");
                match self.get_asset_balance(base_asset, WalletType::Spot).await {
                    Ok(balance) => {
                        if balance.free < qty {
                            return Err(ExchangeError::InsufficientBalance(
                                format!("Insufficient {} balance. Required: {}, Available: {}", base_asset, qty, balance.free)
                            ));
                        }
                    }
                    Err(_) => {} // Continue if we can't check balance
                }
            }
        }

        match self.client.signed_request("order", &params).await {
            Ok(response) => parse_single_order_from_json(response, wallet_type),
            Err(e) => Err(e)
        }
    }

    async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        price: Decimal,
        quantity: Decimal,
        time_in_force: TimeInForce,
        wallet_type: WalletType,
    ) -> Result<Order, ExchangeError> {
        if wallet_type != WalletType::Spot {
            return Err(ExchangeError::NotSupported(format!("{:?} wallet not supported for limit orders", wallet_type)));
        }

        // Validate trading symbol exists (financial safety check)
        self.get_symbol_info(symbol).await?;

        // Validate order parameters
        if price <= Decimal::ZERO {
            return Err(ExchangeError::InvalidOrder("Price must be greater than zero".to_string()));
        }
        if quantity <= Decimal::ZERO {
            return Err(ExchangeError::InvalidOrder("Quantity must be greater than zero".to_string()));
        }

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_uppercase());
        params.insert("side".to_string(), match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        });
        params.insert("type".to_string(), "LIMIT".to_string());
        params.insert("price".to_string(), price.to_string());
        params.insert("quantity".to_string(), quantity.to_string());
        params.insert("timeInForce".to_string(), match time_in_force {
            TimeInForce::GTC => "GTC".to_string(),
            TimeInForce::IOC => "IOC".to_string(),
            TimeInForce::FOK => "FOK".to_string(),
            TimeInForce::GTX => "GTX".to_string(),
        });

        // Check balance before placing order
        let asset_to_check = if side == OrderSide::Buy {
            // For buy orders, check quote asset (usually USDT)
            if symbol.ends_with("USDT") {
                "USDT"
            } else if symbol.ends_with("USDC") {
                "USDC"
            } else if symbol.ends_with("BUSD") {
                "BUSD"
            } else {
                return Err(ExchangeError::InvalidParameter(format!("Unsupported trading pair: {}", symbol)));
            }
        } else {
            // For sell orders, check base asset
            symbol.trim_end_matches("USDT").trim_end_matches("USDC").trim_end_matches("BUSD")
        };

        let required_amount = if side == OrderSide::Buy {
            price * quantity // Quote asset amount needed
        } else {
            quantity // Base asset amount needed
        };

        match self.get_asset_balance(asset_to_check, WalletType::Spot).await {
            Ok(balance) => {
                if balance.free < required_amount {
                    return Err(ExchangeError::InsufficientBalance(
                        format!("Insufficient {} balance. Required: {}, Available: {}", asset_to_check, required_amount, balance.free)
                    ));
                }
            }
            Err(_) => {} // Continue if we can't check balance
        }

        match self.client.signed_request("order", &params).await {
            Ok(response) => parse_single_order_from_json(response, wallet_type),
            Err(e) => Err(e)
        }
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