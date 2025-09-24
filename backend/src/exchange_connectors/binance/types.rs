use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::exchange_connectors::common_types;

/// Binance-specific wallet types focused on the main trading functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceWalletType {
    Spot,
    Margin,
    Futures,
}

impl BinanceWalletType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinanceWalletType::Spot => "spot",
            BinanceWalletType::Margin => "margin", 
            BinanceWalletType::Futures => "futures",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "spot" => Some(BinanceWalletType::Spot),
            "margin" => Some(BinanceWalletType::Margin),
            "futures" => Some(BinanceWalletType::Futures),
            _ => None,
        }
    }
}

/// Binance-specific futures type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceFuturesType {
    USDM,  // USD-M Futures
    COINM, // COIN-M Futures
}

impl BinanceFuturesType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinanceFuturesType::USDM => "usdm",
            BinanceFuturesType::COINM => "coinm",
        }
    }
}

/// Binance asset balance with specific wallet type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAssetBalance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
    pub usd_value: Option<Decimal>,
    pub btc_value: Option<Decimal>,
    pub wallet_type: BinanceWalletType,
}

/// Binance Spot Account - Core trading account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceSpotAccount {
    pub balances: Vec<BinanceAssetBalance>,
    pub total_usd_value: Option<Decimal>,
    pub total_btc_value: Option<Decimal>,
    pub maker_commission: Option<Decimal>,
    pub taker_commission: Option<Decimal>,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub last_update_time: DateTime<Utc>,
}

/// Binance Margin Account - For leveraged trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceMarginAccount {
    pub balances: Vec<BinanceAssetBalance>,
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

/// Binance Isolated Margin Account - For symbol-specific margin trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceIsolatedMarginAccount {
    pub symbol: String,
    pub balances: Vec<BinanceAssetBalance>,
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

/// Binance Futures Account - For futures trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFuturesAccount {
    pub account_type: BinanceFuturesType,
    pub balances: Vec<BinanceAssetBalance>,
    pub positions: Vec<BinanceFuturesPosition>,
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

/// Binance Futures Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFuturesPosition {
    pub symbol: String,
    pub position_side: BinancePositionSide,
    pub position_amount: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub margin_type: BinanceMarginType,
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
pub enum BinancePositionSide {
    Long,
    Short,
    Both,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceMarginType {
    Isolated,
    Cross,
}

/// Comprehensive Binance account balances across all wallet types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAccountBalances {
    pub spot: Option<BinanceSpotAccount>,
    pub margin: Option<BinanceMarginAccount>,
    pub futures_usdm: Option<BinanceFuturesAccount>,
    pub futures_coinm: Option<BinanceFuturesAccount>,
    pub total_usd_value: Decimal,
    pub total_btc_value: Decimal,
}

/// Binance-specific trade request for automated trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceTradeRequest {
    pub symbol: String,
    pub side: BinanceOrderSide,
    pub order_type: BinanceOrderType,
    pub wallet_type: BinanceWalletType,
    pub quantity: Option<Decimal>,
    pub quote_quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub stop_price: Option<Decimal>,
    pub time_in_force: Option<BinanceTimeInForce>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceOrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    LimitMaker,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceTimeInForce {
    GTC, // Good Till Canceled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    GTX, // Good Till Crossing
}

/// Binance order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceOrder {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: BinanceOrderSide,
    pub order_type: BinanceOrderType,
    pub status: BinanceOrderStatus,
    pub time_in_force: BinanceTimeInForce,
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
    pub wallet_type: BinanceWalletType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinanceOrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

// Conversion implementations
impl From<BinanceFuturesType> for common_types::FuturesType {
    fn from(val: BinanceFuturesType) -> Self {
        match val {
            BinanceFuturesType::USDM => common_types::FuturesType::USDM,
            BinanceFuturesType::COINM => common_types::FuturesType::COINM,
        }
    }
}

impl From<common_types::FuturesType> for BinanceFuturesType {
    fn from(val: common_types::FuturesType) -> Self {
        match val {
            common_types::FuturesType::USDM => BinanceFuturesType::USDM,
            common_types::FuturesType::COINM => BinanceFuturesType::COINM,
        }
    }
}

impl From<BinanceFuturesAccount> for common_types::FuturesAccount {
    fn from(val: BinanceFuturesAccount) -> Self {
        common_types::FuturesAccount {
            account_type: val.account_type.into(),
            balances: val.balances.into_iter().map(|b| b.into()).collect(),
            total_wallet_balance: val.total_wallet_balance,
            total_unrealized_pnl: val.total_unrealized_pnl,
            total_margin_balance: val.total_margin_balance,
            available_balance: val.available_balance,
            max_withdraw_amount: val.max_withdraw_amount,
            total_initial_margin: val.total_initial_margin,
            total_maintenance_margin: val.total_maintenance_margin,
            margin_ratio: val.margin_ratio,
            can_trade: val.can_trade,
            can_deposit: val.can_deposit,
            can_withdraw: val.can_withdraw,
            last_update_time: val.last_update_time,
        }
    }
}

impl From<BinanceSpotAccount> for common_types::SpotAccount {
    fn from(val: BinanceSpotAccount) -> Self {
        common_types::SpotAccount {
            balances: val.balances.into_iter().map(|b| b.into()).collect(),
            total_usd_value: val.total_usd_value,
            total_btc_value: val.total_btc_value,
            maker_commission: val.maker_commission,
            taker_commission: val.taker_commission,
            can_trade: val.can_trade,
            can_withdraw: val.can_withdraw,
            can_deposit: val.can_deposit,
            last_update_time: val.last_update_time,
        }
    }
}

impl From<BinanceMarginAccount> for common_types::MarginAccount {
    fn from(val: BinanceMarginAccount) -> Self {
        common_types::MarginAccount {
            balances: val.balances.into_iter().map(|b| b.into()).collect(),
            total_asset_value: val.total_asset_value,
            total_liability_value: val.total_liability_value,
            total_net_value: val.total_net_value,
            margin_level: val.margin_level,
            margin_ratio: val.margin_ratio,
            is_margin_enabled: val.is_margin_enabled,
            can_trade: val.can_trade,
            can_borrow: val.can_borrow,
            last_update_time: val.last_update_time,
        }
    }
}

impl From<BinanceAssetBalance> for common_types::AssetBalance {
    fn from(val: BinanceAssetBalance) -> Self {
        common_types::AssetBalance {
            asset: val.asset,
            free: val.free,
            locked: val.locked,
            total: val.total,
            usd_value: val.usd_value,
            btc_value: val.btc_value,
            wallet_type: val.wallet_type.into(),
        }
    }
}

impl From<BinanceWalletType> for common_types::WalletType {
    fn from(val: BinanceWalletType) -> Self {
        match val {
            BinanceWalletType::Spot => common_types::WalletType::Spot,
            BinanceWalletType::Margin => common_types::WalletType::Margin,
            BinanceWalletType::Futures => common_types::WalletType::Futures,
        }
    }
}

impl From<BinanceAccountBalances> for common_types::AccountBalances {
    fn from(val: BinanceAccountBalances) -> Self {
        common_types::AccountBalances {
            spot: val.spot.map(|s| s.into()),
            margin: val.margin.map(|m| m.into()),
            futures_usdm: val.futures_usdm.map(|f| f.into()),
            futures_coinm: val.futures_coinm.map(|f| f.into()),
            total_usd_value: val.total_usd_value,
            total_btc_value: val.total_btc_value,
        }
    }
}
