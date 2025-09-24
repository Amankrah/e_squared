use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::exchange_connectors::{Kline, KlineInterval};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: Decimal,
    pub strategy_name: String,
    pub strategy_parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub config: BacktestConfig,
    pub trades: Vec<BacktestTrade>,
    pub metrics: BacktestMetrics,
    pub historical_data: Vec<Kline>,
    pub execution_time_ms: u64,
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
}

impl Portfolio {
    pub fn new(initial_balance: Decimal) -> Self {
        Self {
            cash_balance: initial_balance,
            asset_quantity: Decimal::ZERO,
            total_value: initial_balance,
            initial_value: initial_balance,
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
            true
        } else {
            false
        }
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