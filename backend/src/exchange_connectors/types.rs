use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalletType {
    Spot,
    Margin,
    IsolatedMargin,
    Futures,
    FuturesCoin,
    Savings,
    Earn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FuturesType {
    USDM,
    COINM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
    pub usd_value: Option<Decimal>,
    pub btc_value: Option<Decimal>,
    pub wallet_type: WalletType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotAccount {
    pub balances: Vec<AssetBalance>,
    pub total_usd_value: Option<Decimal>,
    pub total_btc_value: Option<Decimal>,
    pub maker_commission: Option<Decimal>,
    pub taker_commission: Option<Decimal>,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub last_update_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginAccount {
    pub balances: Vec<AssetBalance>,
    pub total_asset_value: Decimal,
    pub total_liability_value: Decimal,
    pub total_net_value: Decimal,
    pub margin_level: Option<Decimal>,
    pub margin_ratio: Option<Decimal>,
    pub is_margin_enabled: bool,
    pub can_trade: bool,
    pub can_borrow: bool,
    pub last_update_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedMarginAccount {
    pub symbol: String,
    pub balances: Vec<AssetBalance>,
    pub total_asset_value: Decimal,
    pub total_liability_value: Decimal,
    pub total_net_value: Decimal,
    pub margin_level: Option<Decimal>,
    pub margin_ratio: Option<Decimal>,
    pub can_liquidate: bool,
    pub can_trade: bool,
    pub can_transfer: bool,
    pub last_update_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalances {
    pub spot: Option<SpotAccount>,
    pub margin: Option<MarginAccount>,
    pub isolated_margin: Option<Vec<IsolatedMarginAccount>>,
    pub futures_usdm: Option<FuturesAccount>,
    pub futures_coinm: Option<FuturesAccount>,
    pub earn: Option<Vec<AssetBalance>>,
    pub total_usd_value: Decimal,
    pub total_btc_value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesAccount {
    pub account_type: FuturesType,
    pub balances: Vec<AssetBalance>,
    pub positions: Vec<FuturesPosition>,
    pub total_wallet_balance: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_margin_balance: Decimal,
    pub available_balance: Decimal,
    pub max_withdraw_amount: Decimal,
    pub total_initial_margin: Decimal,
    pub total_maintenance_margin: Decimal,
    pub margin_ratio: Option<Decimal>,
    pub can_trade: bool,
    pub can_deposit: bool,
    pub can_withdraw: bool,
    pub last_update_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPosition {
    pub symbol: String,
    pub position_side: PositionSide,
    pub position_amount: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub margin_type: MarginType,
    pub isolated_margin: Option<Decimal>,
    pub leverage: u32,
    pub liquidation_price: Option<Decimal>,
    pub margin_ratio: Option<Decimal>,
    pub maintenance_margin: Decimal,
    pub initial_margin: Decimal,
    pub position_initial_margin: Decimal,
    pub open_order_initial_margin: Decimal,
    pub adl_quantile: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
    Both,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarginType {
    Isolated,
    Cross,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub quantity: Decimal,
    pub executed_quantity: Decimal,
    pub cumulative_quote_quantity: Decimal,
    pub average_price: Option<Decimal>,
    pub fee: Option<Decimal>,
    pub fee_asset: Option<String>,
    pub pnl: Option<Decimal>,
    pub created_time: DateTime<Utc>,
    pub updated_time: DateTime<Utc>,
    pub wallet_type: WalletType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcoOrder {
    pub order_list_id: String,
    pub contingency_type: String,
    pub list_status: String,
    pub list_order_status: String,
    pub transaction_time: DateTime<Utc>,
    pub symbol: String,
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub order_id: Option<String>,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub quote_quantity: Decimal,
    pub commission: Option<Decimal>,
    pub commission_asset: Option<String>,
    pub time: DateTime<Utc>,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub bid_price: Decimal,
    pub bid_quantity: Decimal,
    pub ask_price: Decimal,
    pub ask_quantity: Decimal,
    pub last_price: Decimal,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub last_update_id: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trades_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KlineInterval {
    OneSecond,
    OneMinute,
    ThreeMinutes,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
    ThreeDays,
    OneWeek,
    OneMonth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: DateTime<Utc>,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub status: String,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub tick_size: Decimal,
    pub min_quantity: Decimal,
    pub max_quantity: Decimal,
    pub step_size: Decimal,
    pub min_notional: Decimal,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub wallet_type: WalletType,
    pub quantity: Option<Decimal>,
    pub quote_quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub time_in_force: Option<TimeInForce>,
    pub client_order_id: Option<String>,
}