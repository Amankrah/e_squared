use chrono::{DateTime, Utc, TimeZone};
use rust_decimal::Decimal;
use std::str::FromStr;
use serde_json::Value;
use tracing::{warn, debug};
use crate::exchange_connectors::{
    ExchangeError,
    shared_types::{Ticker, OrderBook, OrderBookLevel, Trade, Kline, KlineInterval, ExchangeInfo, SymbolInfo},
    common_types::{Order, OrderSide, OrderType, OrderStatus, TimeInForce, WalletType},
};
use super::types::*;
use super::api_client::BinanceApiClient;

pub fn parse_decimal(s: &str) -> Result<Decimal, ExchangeError> {
    Decimal::from_str(s)
        .map_err(|e| ExchangeError::ParseError(format!("Failed to parse decimal: {}", e)))
}

pub fn parse_timestamp(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ts).single()
        .unwrap_or_else(|| Utc::now())
}

pub fn convert_kline_interval_to_string(interval: &KlineInterval) -> &'static str {
    match interval {
        KlineInterval::OneSecond => "1s",
        KlineInterval::OneMinute => "1m",
        KlineInterval::ThreeMinutes => "3m",
        KlineInterval::FiveMinutes => "5m",
        KlineInterval::FifteenMinutes => "15m",
        KlineInterval::ThirtyMinutes => "30m",
        KlineInterval::OneHour => "1h",
        KlineInterval::TwoHours => "2h",
        KlineInterval::FourHours => "4h",
        KlineInterval::SixHours => "6h",
        KlineInterval::EightHours => "8h",
        KlineInterval::TwelveHours => "12h",
        KlineInterval::OneDay => "1d",
        KlineInterval::ThreeDays => "3d",
        KlineInterval::OneWeek => "1w",
        KlineInterval::OneMonth => "1M",
    }
}

pub fn parse_spot_account_from_json(json: Value) -> Result<BinanceSpotAccount, ExchangeError> {
    let mut balances = Vec::new();
    let total_usd_value = Decimal::ZERO;

    if let Some(balance_array) = json.get("balances").and_then(|v| v.as_array()) {
        for balance in balance_array {
            let asset = balance.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
            let free_str = balance.get("free").and_then(|v| v.as_str()).unwrap_or("0");
            let locked_str = balance.get("locked").and_then(|v| v.as_str()).unwrap_or("0");

            let free = parse_decimal(free_str)?;
            let locked = parse_decimal(locked_str)?;
            let total = free + locked;

            if total > Decimal::ZERO {
                balances.push(BinanceAssetBalance {
                    asset: asset.to_string(),
                    free,
                    locked,
                    total,
                    usd_value: None, // TODO: Add price conversion
                    btc_value: None,
                    wallet_type: BinanceWalletType::Spot,
                });
            }
        }
    }

    let maker_commission = json.get("makerCommission")
        .and_then(|v| v.as_i64())
        .map(|c| Decimal::from(c));

    let taker_commission = json.get("takerCommission")
        .and_then(|v| v.as_i64())
        .map(|c| Decimal::from(c));

    let can_trade = json.get("canTrade").and_then(|v| v.as_bool()).unwrap_or(false);
    let can_withdraw = json.get("canWithdraw").and_then(|v| v.as_bool()).unwrap_or(false);
    let can_deposit = json.get("canDeposit").and_then(|v| v.as_bool()).unwrap_or(false);

    let update_time = json.get("updateTime")
        .and_then(|v| v.as_i64())
        .map(parse_timestamp)
        .unwrap_or_else(|| Utc::now());

    Ok(BinanceSpotAccount {
        balances,
        total_usd_value: Some(total_usd_value),
        total_btc_value: None,
        maker_commission: maker_commission,
        taker_commission: taker_commission,
        can_trade,
        can_withdraw,
        can_deposit,
        last_update_time: update_time,
    })
}

pub async fn parse_spot_account_from_json_with_prices(
    json: Value,
    client: &BinanceApiClient
) -> Result<BinanceSpotAccount, ExchangeError> {
    let mut balances = Vec::new();
    let mut total_usd_value = Decimal::ZERO;

    if let Some(balance_array) = json.get("balances").and_then(|v| v.as_array()) {
        for balance in balance_array {
            let asset = balance.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
            let free_str = balance.get("free").and_then(|v| v.as_str()).unwrap_or("0");
            let locked_str = balance.get("locked").and_then(|v| v.as_str()).unwrap_or("0");

            let free = parse_decimal(free_str)?;
            let locked = parse_decimal(locked_str)?;
            let total = free + locked;

            if total > Decimal::ZERO {
                // Get USD value for the asset
                let mut usd_value = None;
                
                // Skip USDT, USDC, BUSD (stablecoins) - they're already ~$1
                if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                    usd_value = Some(total); // 1:1 USD value for stablecoins
                } else {
                    // Get current price from Binance directly
                    match client.get_symbol_price(asset).await {
                        Ok(price) if price > Decimal::ZERO => {
                            let asset_usd_value = total * price;
                            usd_value = Some(asset_usd_value);
                            debug!("Got Binance price for {}: ${}, total value: ${}", asset, price, asset_usd_value);
                        }
                        Ok(_) => {
                            debug!("No price data available for {} on Binance", asset);
                        }
                        Err(e) => {
                            warn!("Failed to get Binance price for {}: {:?}", asset, e);
                        }
                    }
                }

                // Add to total if we got a USD value
                if let Some(value) = usd_value {
                    total_usd_value += value;
                }

                balances.push(BinanceAssetBalance {
                    asset: asset.to_string(),
                    free,
                    locked,
                    total,
                    usd_value,
                    btc_value: None, // TODO: Add BTC conversion if needed
                    wallet_type: BinanceWalletType::Spot,
                });
            }
        }
    }

    let maker_commission = json.get("makerCommission")
        .and_then(|v| v.as_i64())
        .map(|c| Decimal::from(c));

    let taker_commission = json.get("takerCommission")
        .and_then(|v| v.as_i64())
        .map(|c| Decimal::from(c));

    let can_trade = json.get("canTrade").and_then(|v| v.as_bool()).unwrap_or(false);
    let can_withdraw = json.get("canWithdraw").and_then(|v| v.as_bool()).unwrap_or(false);
    let can_deposit = json.get("canDeposit").and_then(|v| v.as_bool()).unwrap_or(false);

    let update_time = json.get("updateTime")
        .and_then(|v| v.as_i64())
        .map(parse_timestamp)
        .unwrap_or_else(|| Utc::now());

    Ok(BinanceSpotAccount {
        balances,
        total_usd_value: Some(total_usd_value),
        total_btc_value: None,
        maker_commission,
        taker_commission,
        can_trade,
        can_withdraw,
        can_deposit,
        last_update_time: update_time,
    })
}

pub async fn parse_margin_account_from_json_with_prices(
    json: Value,
    client: &BinanceApiClient
) -> Result<BinanceMarginAccount, ExchangeError> {
    let mut balances = Vec::new();
    let mut total_asset_value = Decimal::ZERO;
    let mut total_liability_value = Decimal::ZERO;

    if let Some(user_assets) = json.get("userAssets").and_then(|v| v.as_array()) {
        for asset_info in user_assets {
            let asset = asset_info.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
            let free_str = asset_info.get("free").and_then(|v| v.as_str()).unwrap_or("0");
            let locked_str = asset_info.get("locked").and_then(|v| v.as_str()).unwrap_or("0");
            let borrowed_str = asset_info.get("borrowed").and_then(|v| v.as_str()).unwrap_or("0");
            let interest_str = asset_info.get("interest").and_then(|v| v.as_str()).unwrap_or("0");

            let free = parse_decimal(free_str)?;
            let locked = parse_decimal(locked_str)?;
            let borrowed = parse_decimal(borrowed_str)?;
            let interest = parse_decimal(interest_str)?;
            let total = free + locked;

            if total > Decimal::ZERO || borrowed > Decimal::ZERO {
                // Get USD value for the asset
                let mut usd_value = None;
                
                // Skip USDT, USDC, BUSD (stablecoins) - they're already ~$1
                if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                    usd_value = Some(total);
                    total_asset_value += total;
                    total_liability_value += borrowed + interest;
                } else {
                    // Get current price from Binance directly
                    match client.get_symbol_price(asset).await {
                        Ok(price) => {
                            let asset_usd_value = total * price;
                            let liability_usd_value = (borrowed + interest) * price;
                            usd_value = Some(asset_usd_value);
                            total_asset_value += asset_usd_value;
                            total_liability_value += liability_usd_value;
                            debug!("Margin - Got Binance price for {}: ${}, asset value: ${}, liability: ${}",
                                   asset, price, asset_usd_value, liability_usd_value);
                        }
                        Err(e) => {
                            warn!("Failed to get Binance price for margin asset {}: {:?}", asset, e);
                        }
                    }
                }

                balances.push(BinanceAssetBalance {
                    asset: asset.to_string(),
                    free,
                    locked,
                    total,
                    usd_value,
                    btc_value: None,
                    wallet_type: BinanceWalletType::Margin,
                });
            }
        }
    }

    let total_net_value = total_asset_value - total_liability_value;
    let margin_level = if total_liability_value > Decimal::ZERO {
        Some(total_asset_value / total_liability_value)
    } else {
        None
    };

    let can_trade = json.get("tradeEnabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let can_borrow = json.get("borrowEnabled").and_then(|v| v.as_bool()).unwrap_or(false);

    Ok(BinanceMarginAccount {
        balances,
        total_asset_value,
        total_liability_value,
        total_net_value,
        margin_level,
        margin_ratio: None,
        is_margin_enabled: can_trade,
        can_trade,
        can_borrow,
        last_update_time: Utc::now(),
    })
}

pub async fn parse_isolated_margin_accounts_from_json(
    json: Value,
    client: &BinanceApiClient
) -> Result<Vec<BinanceIsolatedMarginAccount>, ExchangeError> {
    let mut isolated_accounts = Vec::new();

    if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
        for asset_info in assets {
            let symbol = asset_info.get("symbol").and_then(|v| v.as_str()).unwrap_or_default();
            let mut balances = Vec::new();
            let mut total_asset_value = Decimal::ZERO;
            let mut total_liability_value = Decimal::ZERO;

            // Parse base asset
            if let Some(base_asset) = asset_info.get("baseAsset") {
                let asset = base_asset.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
                let free_str = base_asset.get("free").and_then(|v| v.as_str()).unwrap_or("0");
                let locked_str = base_asset.get("locked").and_then(|v| v.as_str()).unwrap_or("0");
                let borrowed_str = base_asset.get("borrowed").and_then(|v| v.as_str()).unwrap_or("0");
                let interest_str = base_asset.get("interest").and_then(|v| v.as_str()).unwrap_or("0");

                let free = parse_decimal(free_str)?;
                let locked = parse_decimal(locked_str)?;
                let borrowed = parse_decimal(borrowed_str)?;
                let interest = parse_decimal(interest_str)?;
                let total = free + locked;

                if total > Decimal::ZERO || borrowed > Decimal::ZERO {
                    let mut usd_value = None;

                    if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                        usd_value = Some(total);
                        total_asset_value += total;
                        total_liability_value += borrowed + interest;
                    } else {
                        match client.get_symbol_price(asset).await {
                            Ok(price) => {
                                let asset_usd_value = total * price;
                                let liability_usd_value = (borrowed + interest) * price;
                                usd_value = Some(asset_usd_value);
                                total_asset_value += asset_usd_value;
                                total_liability_value += liability_usd_value;
                            }
                            Err(e) => {
                                warn!("Failed to get price for isolated margin base asset {}: {:?}", asset, e);
                            }
                        }
                    }

                    balances.push(BinanceAssetBalance {
                        asset: asset.to_string(),
                        free,
                        locked,
                        total,
                        usd_value,
                        btc_value: None,
                        wallet_type: BinanceWalletType::Margin, // Map isolated margin to margin for compatibility
                    });
                }
            }

            // Parse quote asset
            if let Some(quote_asset) = asset_info.get("quoteAsset") {
                let asset = quote_asset.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
                let free_str = quote_asset.get("free").and_then(|v| v.as_str()).unwrap_or("0");
                let locked_str = quote_asset.get("locked").and_then(|v| v.as_str()).unwrap_or("0");
                let borrowed_str = quote_asset.get("borrowed").and_then(|v| v.as_str()).unwrap_or("0");
                let interest_str = quote_asset.get("interest").and_then(|v| v.as_str()).unwrap_or("0");

                let free = parse_decimal(free_str)?;
                let locked = parse_decimal(locked_str)?;
                let borrowed = parse_decimal(borrowed_str)?;
                let interest = parse_decimal(interest_str)?;
                let total = free + locked;

                if total > Decimal::ZERO || borrowed > Decimal::ZERO {
                    let mut usd_value = None;

                    if asset == "USDT" || asset == "USDC" || asset == "BUSD" || asset == "DAI" {
                        usd_value = Some(total);
                        total_asset_value += total;
                        total_liability_value += borrowed + interest;
                    } else {
                        match client.get_symbol_price(asset).await {
                            Ok(price) => {
                                let asset_usd_value = total * price;
                                let liability_usd_value = (borrowed + interest) * price;
                                usd_value = Some(asset_usd_value);
                                total_asset_value += asset_usd_value;
                                total_liability_value += liability_usd_value;
                            }
                            Err(e) => {
                                warn!("Failed to get price for isolated margin quote asset {}: {:?}", asset, e);
                            }
                        }
                    }

                    balances.push(BinanceAssetBalance {
                        asset: asset.to_string(),
                        free,
                        locked,
                        total,
                        usd_value,
                        btc_value: None,
                        wallet_type: BinanceWalletType::Margin, // Map isolated margin to margin for compatibility
                    });
                }
            }

            // Only add the isolated margin account if there are balances
            if !balances.is_empty() {
                let total_net_value = total_asset_value - total_liability_value;
                let margin_level = if total_liability_value > Decimal::ZERO {
                    Some(total_asset_value / total_liability_value)
                } else {
                    None
                };

                let margin_ratio_str = asset_info.get("marginRatio").and_then(|v| v.as_str()).unwrap_or("0");
                let margin_ratio = parse_decimal(margin_ratio_str).ok();

                let can_liquidate = asset_info.get("liquidatePrice").and_then(|v| v.as_str()).is_some();
                let can_trade = asset_info.get("tradeEnabled").and_then(|v| v.as_bool()).unwrap_or(false);

                isolated_accounts.push(BinanceIsolatedMarginAccount {
                    symbol: symbol.to_string(),
                    balances,
                    total_asset_value,
                    total_liability_value,
                    total_net_value,
                    margin_level,
                    margin_ratio,
                    can_liquidate,
                    can_trade,
                    can_transfer: true, // Default assumption
                    last_update_time: Utc::now(),
                });
            }
        }
    }

    Ok(isolated_accounts)
}

pub async fn parse_futures_account_from_json_with_prices(
    json: Value,
    client: &BinanceApiClient,
    account_type: BinanceFuturesType,
) -> Result<BinanceFuturesAccount, ExchangeError> {
    let mut balances = Vec::new();
    let mut positions = Vec::new();

    // Parse assets/balances
    if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
        for asset_info in assets {
            let asset = asset_info.get("asset").and_then(|v| v.as_str()).unwrap_or_default();
            let wallet_balance_str = asset_info.get("walletBalance").and_then(|v| v.as_str()).unwrap_or("0");
            let unrealized_profit_str = asset_info.get("unrealizedProfit").and_then(|v| v.as_str()).unwrap_or("0");
            let margin_balance_str = asset_info.get("marginBalance").and_then(|v| v.as_str()).unwrap_or("0");

            let wallet_balance = parse_decimal(wallet_balance_str)?;
            let _unrealized_profit = parse_decimal(unrealized_profit_str)?;
            let margin_balance = parse_decimal(margin_balance_str)?;

            if wallet_balance > Decimal::ZERO || margin_balance > Decimal::ZERO {
                // Get USD value for the asset
                let mut usd_value = None;
                
                if asset == "USDT" || asset == "USDC" || asset == "BUSD" {
                    usd_value = Some(margin_balance);
                } else {
                    match client.get_symbol_price(asset).await {
                        Ok(price) => {
                            usd_value = Some(margin_balance * price);
                            debug!("Futures - Got Binance price for {}: ${}, value: ${}", asset, price, margin_balance * price);
                        }
                        Err(e) => {
                            warn!("Failed to get Binance price for futures asset {}: {:?}", asset, e);
                        }
                    }
                }

                balances.push(BinanceAssetBalance {
                    asset: asset.to_string(),
                    free: wallet_balance,
                    locked: Decimal::ZERO,
                    total: wallet_balance,
                    usd_value,
                    btc_value: None,
                    wallet_type: BinanceWalletType::Futures,
                });
            }
        }
    }

    // Parse positions
    if let Some(position_list) = json.get("positions").and_then(|v| v.as_array()) {
        for position_info in position_list {
            let symbol = position_info.get("symbol").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let position_amount_str = position_info.get("positionAmt").and_then(|v| v.as_str()).unwrap_or("0");
            let entry_price_str = position_info.get("entryPrice").and_then(|v| v.as_str()).unwrap_or("0");
            let mark_price_str = position_info.get("markPrice").and_then(|v| v.as_str()).unwrap_or("0");
            let unrealized_pnl_str = position_info.get("unRealizedProfit").and_then(|v| v.as_str()).unwrap_or("0");

            let position_amount = parse_decimal(position_amount_str)?;
            let entry_price = parse_decimal(entry_price_str)?;
            let mark_price = parse_decimal(mark_price_str)?;
            let unrealized_pnl = parse_decimal(unrealized_pnl_str)?;

            if position_amount != Decimal::ZERO {
                positions.push(BinanceFuturesPosition {
                    symbol,
                    position_side: if position_amount > Decimal::ZERO { BinancePositionSide::Long } else { BinancePositionSide::Short },
                    position_amount,
                    entry_price,
                    mark_price,
                    unrealized_pnl,
                    realized_pnl: Decimal::ZERO,
                    margin_type: BinanceMarginType::Cross, // Default, could be parsed
                    isolated_margin: None,
                    leverage: 1, // Default, could be parsed
                    liquidation_price: None,
                    margin_ratio: None,
                    maintenance_margin: Decimal::ZERO,
                    initial_margin: Decimal::ZERO,
                    position_initial_margin: Decimal::ZERO,
                    open_order_initial_margin: Decimal::ZERO,
                    adl_quantile: None,
                });
            }
        }
    }

    let total_wallet_balance = parse_decimal(json.get("totalWalletBalance").and_then(|v| v.as_str()).unwrap_or("0"))?;
    let total_unrealized_pnl = parse_decimal(json.get("totalUnrealizedProfit").and_then(|v| v.as_str()).unwrap_or("0"))?;
    let total_margin_balance = parse_decimal(json.get("totalMarginBalance").and_then(|v| v.as_str()).unwrap_or("0"))?;
    let available_balance = parse_decimal(json.get("availableBalance").and_then(|v| v.as_str()).unwrap_or("0"))?;

    let can_trade = json.get("canTrade").and_then(|v| v.as_bool()).unwrap_or(true);
    let can_deposit = json.get("canDeposit").and_then(|v| v.as_bool()).unwrap_or(true);
    let can_withdraw = json.get("canWithdraw").and_then(|v| v.as_bool()).unwrap_or(true);

    Ok(BinanceFuturesAccount {
        account_type,
        balances,
        positions,
        total_wallet_balance,
        total_unrealized_pnl,
        total_margin_balance,
        available_balance,
        max_withdraw_amount: available_balance, // Approximation
        total_initial_margin: Decimal::ZERO,
        total_maintenance_margin: Decimal::ZERO,
        margin_ratio: None,
        can_trade,
        can_deposit,
        can_withdraw,
        last_update_time: Utc::now(),
    })
}

pub fn parse_orders_from_json(json: Value, wallet_type: WalletType) -> Result<Vec<Order>, ExchangeError> {
    let mut orders = Vec::new();

    if let Some(order_array) = json.as_array() {
        for order_json in order_array {
            let order = parse_single_order_from_json(order_json.clone(), wallet_type.clone())?;
            orders.push(order);
        }
    }

    Ok(orders)
}

pub fn parse_single_order_from_json(json: Value, wallet_type: WalletType) -> Result<Order, ExchangeError> {
    let order_id = json.get("orderId")
        .and_then(|v| v.as_i64())
        .map(|id| id.to_string())
        .unwrap_or_default();

    let client_order_id = json.get("clientOrderId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let symbol = json.get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let side = match json.get("side").and_then(|v| v.as_str()).unwrap_or_default() {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };

    let order_type = match json.get("type").and_then(|v| v.as_str()).unwrap_or_default() {
        "MARKET" => OrderType::Market,
        "LIMIT" => OrderType::Limit,
        "STOP_LOSS" => OrderType::StopLoss,
        "STOP_LOSS_LIMIT" => OrderType::StopLossLimit,
        "TAKE_PROFIT" => OrderType::TakeProfit,
        "TAKE_PROFIT_LIMIT" => OrderType::TakeProfitLimit,
        "LIMIT_MAKER" => OrderType::LimitMaker,
        _ => OrderType::Limit,
    };

    let status = match json.get("status").and_then(|v| v.as_str()).unwrap_or_default() {
        "NEW" => OrderStatus::New,
        "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
        "FILLED" => OrderStatus::Filled,
        "CANCELED" => OrderStatus::Canceled,
        "REJECTED" => OrderStatus::Rejected,
        "EXPIRED" => OrderStatus::Expired,
        _ => OrderStatus::New,
    };

    let time_in_force = match json.get("timeInForce").and_then(|v| v.as_str()).unwrap_or_default() {
        "GTC" => TimeInForce::GTC,
        "IOC" => TimeInForce::IOC,
        "FOK" => TimeInForce::FOK,
        _ => TimeInForce::GTC,
    };

    let price = json.get("price")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?;

    let stop_price = json.get("stopPrice")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?;

    let quantity = json.get("origQty")
        .and_then(|v| v.as_str())
        .map(|q| parse_decimal(q))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let executed_quantity = json.get("executedQty")
        .and_then(|v| v.as_str())
        .map(|q| parse_decimal(q))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let cumulative_quote_quantity = json.get("cummulativeQuoteQty")
        .and_then(|v| v.as_str())
        .map(|q| parse_decimal(q))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let created_time = json.get("time")
        .and_then(|v| v.as_i64())
        .map(parse_timestamp)
        .unwrap_or_else(|| Utc::now());

    let updated_time = json.get("updateTime")
        .and_then(|v| v.as_i64())
        .map(parse_timestamp)
        .unwrap_or_else(|| Utc::now());

    Ok(Order {
        order_id,
        client_order_id,
        symbol,
        side,
        order_type,
        status,
        time_in_force,
        price: price,
        stop_price: stop_price,
        quantity: quantity,
        executed_quantity: executed_quantity,
        cumulative_quote_quantity: cumulative_quote_quantity,
        average_price: None,
        fee: None,
        fee_asset: None,
        pnl: None,
        created_time: created_time,
        updated_time: updated_time,
        wallet_type,
    })
}

pub fn parse_ticker_from_json(json: Value, symbol: &str) -> Result<Ticker, ExchangeError> {
    let bid_price = json.get("bidPrice")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let ask_price = json.get("askPrice")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let last_price = json.get("lastPrice")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let price_change = json.get("priceChange")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    let price_change_percent = json.get("priceChangePercent")
        .and_then(|v| v.as_str())
        .map(|p| parse_decimal(p))
        .transpose()?
        .unwrap_or(Decimal::ZERO);

    Ok(Ticker {
        symbol: symbol.to_string(),
        bid_price: bid_price,
        bid_quantity: Decimal::ZERO,
        ask_price: ask_price,
        ask_quantity: Decimal::ZERO,
        last_price: last_price,
        price_change: price_change,
        price_change_percent: price_change_percent,
        high_price: Decimal::ZERO,
        low_price: Decimal::ZERO,
        volume: Decimal::ZERO,
        quote_volume: Decimal::ZERO,
        open_time: Utc::now(),
        close_time: Utc::now(),
    })
}

pub fn parse_order_book_from_json(json: Value, symbol: &str) -> Result<OrderBook, ExchangeError> {
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    if let Some(bid_array) = json.get("bids").and_then(|v| v.as_array()) {
        for bid in bid_array {
            if let Some(bid_array) = bid.as_array() {
                if bid_array.len() >= 2 {
                    let price = bid_array[0].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let quantity = bid_array[1].as_str().map(|q| parse_decimal(q)).transpose()?;

                    if let (Some(p), Some(q)) = (price, quantity) {
                        bids.push(OrderBookLevel {
                            price: p,
                            quantity: q,
                        });
                    }
                }
            }
        }
    }

    if let Some(ask_array) = json.get("asks").and_then(|v| v.as_array()) {
        for ask in ask_array {
            if let Some(ask_array) = ask.as_array() {
                if ask_array.len() >= 2 {
                    let price = ask_array[0].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let quantity = ask_array[1].as_str().map(|q| parse_decimal(q)).transpose()?;

                    if let (Some(p), Some(q)) = (price, quantity) {
                        asks.push(OrderBookLevel {
                            price: p,
                            quantity: q,
                        });
                    }
                }
            }
        }
    }

    let last_update_id = json.get("lastUpdateId")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(OrderBook {
        symbol: symbol.to_string(),
        bids,
        asks,
        last_update_id,
        timestamp: Utc::now(),
    })
}

pub fn parse_trades_from_json(json: Value, symbol: &str) -> Result<Vec<Trade>, ExchangeError> {
    let mut trades = Vec::new();

    if let Some(trade_array) = json.as_array() {
        for trade_json in trade_array {
            let id = trade_json.get("id")
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string())
                .unwrap_or_default();

            let price = trade_json.get("price")
                .and_then(|v| v.as_str())
                .map(|p| parse_decimal(p))
                .transpose()?
                .unwrap_or(Decimal::ZERO);

            let quantity = trade_json.get("qty")
                .and_then(|v| v.as_str())
                .map(|q| parse_decimal(q))
                .transpose()?
                .unwrap_or(Decimal::ZERO);

            let quote_quantity = trade_json.get("quoteQty")
                .and_then(|v| v.as_str())
                .map(|q| parse_decimal(q))
                .transpose()?
                .unwrap_or(Decimal::ZERO);

            let time = trade_json.get("time")
                .and_then(|v| v.as_i64())
                .map(parse_timestamp)
                .unwrap_or_else(|| Utc::now());

            let is_buyer_maker = trade_json.get("isBuyerMaker")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            trades.push(Trade {
                id,
                order_id: None,
                symbol: symbol.to_string(),
                price: price,
                quantity: quantity,
                quote_quantity: quote_quantity,
                commission: None,
                commission_asset: None,
                time: time,
                is_buyer: !is_buyer_maker,
                is_maker: is_buyer_maker,
                is_best_match: Some(true),
            });
        }
    }

    Ok(trades)
}

pub fn parse_klines_from_json(json: Value) -> Result<Vec<Kline>, ExchangeError> {
    let mut klines = Vec::new();

    if let Some(kline_array) = json.as_array() {
        for kline_json in kline_array {
            if let Some(kline_array) = kline_json.as_array() {
                if kline_array.len() >= 12 {
                    let open_time = kline_array[0].as_i64().map(parse_timestamp);
                    let open = kline_array[1].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let high = kline_array[2].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let low = kline_array[3].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let close = kline_array[4].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let volume = kline_array[5].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let close_time = kline_array[6].as_i64().map(parse_timestamp);
                    let quote_volume = kline_array[7].as_str().map(|p| parse_decimal(p)).transpose()?;
                    let trades_count = kline_array[8].as_u64();

                    if let (Some(open_time), Some(close_time), Some(open), Some(high), Some(low), Some(close), Some(volume), Some(quote_volume), Some(trades_count)) =
                        (open_time, close_time, open, high, low, close, volume, quote_volume, trades_count) {
                        klines.push(Kline {
                            open_time: open_time,
                            close_time: close_time,
                            open: open,
                            high: high,
                            low: low,
                            close: close,
                            volume: volume,
                            quote_volume: quote_volume,
                            trades_count,
                        });
                    }
                }
            }
        }
    }

    Ok(klines)
}

pub fn parse_exchange_info_from_json(json: Value) -> Result<ExchangeInfo, ExchangeError> {
    let timezone = json.get("timezone")
        .and_then(|v| v.as_str())
        .unwrap_or("UTC")
        .to_string();

    let server_time = json.get("serverTime")
        .and_then(|v| v.as_i64())
        .map(parse_timestamp)
        .unwrap_or_else(|| Utc::now());

    let mut symbols = Vec::new();
    if let Some(symbol_array) = json.get("symbols").and_then(|v| v.as_array()) {
        for symbol_json in symbol_array {
            let symbol = symbol_json.get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let base_asset = symbol_json.get("baseAsset")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let quote_asset = symbol_json.get("quoteAsset")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let status = symbol_json.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            symbols.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status,
                min_price: Decimal::ZERO,
                max_price: Decimal::ZERO,
                tick_size: Decimal::ZERO,
                min_quantity: Decimal::ZERO,
                max_quantity: Decimal::ZERO,
                step_size: Decimal::ZERO,
                min_notional: Decimal::ZERO,
                is_spot_trading_allowed: true,
                is_margin_trading_allowed: false,
                permissions: vec!["SPOT".to_string()],
            });
        }
    }

    Ok(ExchangeInfo {
        timezone,
        server_time: server_time,
        rate_limits: vec![], // TODO: Parse rate limits
        symbols,
    })
}