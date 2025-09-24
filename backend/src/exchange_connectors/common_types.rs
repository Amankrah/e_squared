use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Generic order side for all exchanges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Generic order type for all exchanges
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

/// Generic order status for all exchanges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

/// Generic time in force for all exchanges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Canceled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTX, // Good Till Crossing
}

/// Generic wallet type for all exchanges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalletType {
    Spot,
    Margin,
    Futures,
}

/// Generic futures type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FuturesType {
    USDM,
    COINM,
}

/// Generic order for all exchanges
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

/// Generic OCO order for all exchanges
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

/// Generic asset balance for all exchanges
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

/// Generic spot account - all exchanges must implement this basic interface
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

/// Generic margin account - all exchanges must implement this basic interface
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

/// Generic futures account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesAccount {
    pub account_type: FuturesType,
    pub balances: Vec<AssetBalance>,
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

/// Generic account balances - all exchanges must implement this basic interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalances {
    pub spot: Option<SpotAccount>,
    pub margin: Option<MarginAccount>,
    pub futures_usdm: Option<FuturesAccount>,
    pub futures_coinm: Option<FuturesAccount>,
    pub total_usd_value: Decimal,
    pub total_btc_value: Decimal,
}
