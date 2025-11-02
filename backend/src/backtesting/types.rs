use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::exchange_connectors::{KlineInterval};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: Decimal,
    pub strategy_name: String,
    /// Sub-strategy type (e.g., "Simple", "RSIBased" for DCA)
    pub strategy_type: Option<String>,
    pub strategy_parameters: serde_json::Value,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
    /// Enable unlimited capital mode (for DCA strategies)
    /// When true, capital is "injected" for each buy, simulating ongoing income
    #[serde(default)]
    pub unlimited_capital: bool,
    /// Asset type: "crypto" or "stock"
    #[serde(default = "default_asset_type")]
    pub asset_type: String,
}

fn default_asset_type() -> String {
    "crypto".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub config: BacktestConfig,
    pub trades: Vec<BacktestTrade>,
    pub metrics: BacktestMetrics,
    pub performance_chart: Vec<PerformancePoint>,
    pub execution_time_ms: u64,
    pub open_positions: Vec<OpenPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPosition {
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub quantity: Decimal,
    pub total_value: Decimal,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub timestamp: DateTime<Utc>,
    pub trade_type: TradeType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub total_value: Decimal,
    pub portfolio_value: Decimal,
    pub balance_remaining: Decimal,
    pub reason: String,
    pub pnl: Option<Decimal>,
    pub pnl_percentage: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeType {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub annualized_return: Option<Decimal>,
    pub sharpe_ratio: Option<Decimal>,
    pub max_drawdown: Decimal,
    pub volatility: Decimal,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub profit_factor: Option<Decimal>,
    pub final_portfolio_value: Decimal,
    pub benchmark_return: Option<Decimal>,
    pub alpha: Option<Decimal>,
    pub beta: Option<Decimal>,
    /// Total amount invested (for DCA strategies)
    pub total_invested: Decimal,
    /// Number of closed trades (trades with realized P&L)
    pub closed_trades: u32,
    /// Number of open trades (buys without corresponding sells)
    pub open_trades: u32,
    /// Realized profit/loss from closed trades
    pub realized_pnl: Decimal,
    /// Unrealized profit/loss from open positions
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataRequest {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub exchange: String,
}

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub cash_balance: Decimal,
    pub asset_quantity: Decimal,
    pub total_value: Decimal,
    pub initial_value: Decimal,
    /// Total amount invested (for DCA tracking)
    pub total_invested: Decimal,
}

impl Portfolio {
    pub fn new(initial_balance: Decimal) -> Self {
        Self {
            cash_balance: initial_balance,
            asset_quantity: Decimal::ZERO,
            total_value: initial_balance,
            initial_value: initial_balance,
            total_invested: Decimal::ZERO,
        }
    }

    pub fn update_total_value(&mut self, current_price: Decimal) {
        self.total_value = self.cash_balance + (self.asset_quantity * current_price);
    }

    pub fn execute_buy(&mut self, price: Decimal, quantity: Decimal) -> bool {
        let total_cost = price * quantity;
        if self.cash_balance >= total_cost {
            self.cash_balance -= total_cost;
            self.asset_quantity += quantity;
            self.total_invested += total_cost;
            true
        } else {
            false
        }
    }

    /// Inject capital for unlimited capital mode (DCA)
    pub fn inject_capital(&mut self, amount: Decimal) {
        self.cash_balance += amount;
    }

    /// Execute buy with capital injection if needed (for DCA unlimited mode)
    pub fn execute_buy_with_injection(&mut self, price: Decimal, quantity: Decimal) -> bool {
        let total_cost = price * quantity;

        // Inject exactly the amount needed if insufficient
        if self.cash_balance < total_cost {
            let needed = total_cost - self.cash_balance;
            self.inject_capital(needed);
        }

        self.cash_balance -= total_cost;
        self.asset_quantity += quantity;
        self.total_invested += total_cost;
        true
    }

    pub fn execute_sell(&mut self, price: Decimal, quantity: Decimal) -> bool {
        if self.asset_quantity >= quantity {
            self.cash_balance += price * quantity;
            self.asset_quantity -= quantity;
            true
        } else {
            false
        }
    }
}

/// Performance point for charting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePoint {
    pub timestamp: DateTime<Utc>,
    pub portfolio_value: Decimal,
    pub asset_price: Decimal,
    pub trade_marker: Option<TradeType>,
}

/// Backtesting request from API
#[derive(Debug, Clone, Deserialize)]
pub struct BacktestRequest {
    pub symbol: String,
    pub interval: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_balance: Decimal,
    pub strategy_name: String,
    pub strategy_parameters: Option<serde_json::Value>,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
    /// Asset type: "crypto" or "stock" (defaults to "crypto")
    #[serde(default = "default_asset_type")]
    pub asset_type: String,
}