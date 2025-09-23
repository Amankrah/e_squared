use std::time::Instant;
use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, prelude::*};
use tracing::{info, debug, error};

use crate::backtesting::types::*;
use crate::strategies::Strategy;
use crate::exchange_connectors::{
    ExchangeFactory, ExchangeCredentials, Exchange,
    types::{Kline, KlineInterval},
};
use crate::utils::errors::AppError;

pub struct BacktestEngine;

impl BacktestEngine {
    pub fn new() -> Self {
        Self
    }

    /// Run a backtest with the given configuration and strategy
    pub async fn run_backtest(
        &self,
        config: BacktestConfig,
        mut strategy: Box<dyn Strategy>,
    ) -> Result<BacktestResult, AppError> {
        let start_time = Instant::now();

        info!("Starting backtest for {} from {} to {}",
              config.symbol, config.start_time, config.end_time);

        // Fetch historical data
        let historical_data = self.fetch_historical_data(&config).await?;

        if historical_data.is_empty() {
            return Err(AppError::BadRequest("No historical data available for the given period".to_string()));
        }

        debug!("Fetched {} klines for backtesting", historical_data.len());

        // Initialize portfolio
        let mut portfolio = Portfolio::new(config.initial_balance);
        let mut trades = Vec::new();

        // Initialize strategy
        strategy.initialize(&config.strategy_parameters)?;

        // Run strategy on each kline
        for (index, kline) in historical_data.iter().enumerate() {
            portfolio.update_total_value(kline.close);

            // Get strategy signal
            let signal = strategy.analyze(&historical_data, index);

            match signal {
                Some(crate::strategies::TradeSignal::Buy(amount)) => {
                    let quantity = amount / kline.close;
                    if portfolio.execute_buy(kline.close, quantity) {
                        trades.push(BacktestTrade {
                            timestamp: kline.close_time,
                            trade_type: TradeType::Buy,
                            price: kline.close,
                            quantity,
                            total_value: amount,
                            portfolio_value: portfolio.total_value,
                            balance_remaining: portfolio.cash_balance,
                            reason: strategy.get_last_signal_reason(),
                        });
                        debug!("Executed buy: {} @ {}", quantity, kline.close);
                    }
                }
                Some(crate::strategies::TradeSignal::Sell(quantity)) => {
                    if portfolio.execute_sell(kline.close, quantity) {
                        trades.push(BacktestTrade {
                            timestamp: kline.close_time,
                            trade_type: TradeType::Sell,
                            price: kline.close,
                            quantity,
                            total_value: quantity * kline.close,
                            portfolio_value: portfolio.total_value,
                            balance_remaining: portfolio.cash_balance,
                            reason: strategy.get_last_signal_reason(),
                        });
                        debug!("Executed sell: {} @ {}", quantity, kline.close);
                    }
                }
                None => {
                    // No signal, continue
                }
            }
        }

        // Calculate final portfolio value
        if let Some(last_kline) = historical_data.last() {
            portfolio.update_total_value(last_kline.close);
        }

        // Calculate metrics
        let metrics = self.calculate_metrics(&trades, &portfolio, &historical_data, &config);

        let execution_time = start_time.elapsed().as_millis() as u64;

        info!("Backtest completed in {}ms. Final portfolio value: {}",
              execution_time, portfolio.total_value);

        Ok(BacktestResult {
            config,
            trades,
            metrics,
            historical_data,
            execution_time_ms: execution_time,
        })
    }

    /// Fetch historical data from the specified exchange
    async fn fetch_historical_data(&self, config: &BacktestConfig) -> Result<Vec<Kline>, AppError> {
        // For now, we'll use a demo exchange connection
        // In production, this could be configurable or use a data provider

        // Create a dummy exchange connector for data fetching
        // We'll use Binance as the default data source
        let exchange = Exchange::from_str("Binance")
            .ok_or_else(|| AppError::BadRequest("Unsupported exchange".to_string()))?;

        // For backtesting, we can use public API endpoints that don't require authentication
        let credentials = ExchangeCredentials {
            api_key: String::new(),
            api_secret: String::new(),
            passphrase: None,
        };

        let connector = ExchangeFactory::create(exchange, credentials)
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to create exchange connector: {}", e)))?;

        // Fetch klines data
        let klines = connector
            .get_klines(
                &config.symbol,
                config.interval.clone(),
                Some(config.start_time),
                Some(config.end_time),
                None, // No limit, get all data in range
            )
            .await
            .map_err(|e| AppError::ExternalServiceError(format!("Failed to fetch historical data: {}", e)))?;

        Ok(klines)
    }

    /// Calculate comprehensive backtest metrics
    fn calculate_metrics(
        &self,
        trades: &[BacktestTrade],
        portfolio: &Portfolio,
        historical_data: &[Kline],
        config: &BacktestConfig,
    ) -> BacktestMetrics {
        let final_value = portfolio.total_value;
        let initial_value = portfolio.initial_value;

        // Basic returns
        let total_return = final_value - initial_value;
        let total_return_percentage = if initial_value > Decimal::ZERO {
            (total_return / initial_value) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Trade statistics
        let total_trades = trades.len() as u32;
        let mut winning_trades = 0u32;
        let mut losing_trades = 0u32;
        let mut total_wins = Decimal::ZERO;
        let mut total_losses = Decimal::ZERO;

        // Calculate trade-level P&L
        for trade in trades {
            // This is a simplified P&L calculation
            // In a more sophisticated system, we'd track the exact entry/exit pairs
            match trade.trade_type {
                TradeType::Buy => {
                    // For buy trades, we'll compare against the final price
                    if let Some(last_kline) = historical_data.last() {
                        let unrealized_pnl = (last_kline.close - trade.price) * trade.quantity;
                        if unrealized_pnl > Decimal::ZERO {
                            winning_trades += 1;
                            total_wins += unrealized_pnl;
                        } else {
                            losing_trades += 1;
                            total_losses += unrealized_pnl.abs();
                        }
                    }
                }
                TradeType::Sell => {
                    // For sell trades, we assume they were profitable if executed
                    winning_trades += 1;
                    total_wins += trade.total_value;
                }
            }
        }

        let win_rate = if total_trades > 0 {
            Decimal::from(winning_trades) / Decimal::from(total_trades) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let average_win = if winning_trades > 0 {
            total_wins / Decimal::from(winning_trades)
        } else {
            Decimal::ZERO
        };

        let average_loss = if losing_trades > 0 {
            total_losses / Decimal::from(losing_trades)
        } else {
            Decimal::ZERO
        };

        let profit_factor = if total_losses > Decimal::ZERO {
            Some(total_wins / total_losses)
        } else {
            None
        };

        // Calculate max drawdown
        let max_drawdown = self.calculate_max_drawdown(trades);

        // Calculate volatility
        let volatility = self.calculate_volatility(historical_data);

        // Annualized return calculation
        let days_elapsed = (config.end_time - config.start_time).num_days() as f64;
        let years_elapsed = days_elapsed / 365.25;

        let annualized_return = if years_elapsed > 0.0 && initial_value > Decimal::ZERO {
            let return_ratio = final_value / initial_value;
            let annualized = return_ratio.to_f64().unwrap_or(1.0).powf(1.0 / years_elapsed) - 1.0;
            Some(Decimal::from_f64(annualized * 100.0).unwrap_or(Decimal::ZERO))
        } else {
            None
        };

        // Sharpe ratio (simplified - using volatility as standard deviation)
        let sharpe_ratio = if volatility > Decimal::ZERO && annualized_return.is_some() {
            Some(annualized_return.unwrap() / volatility)
        } else {
            None
        };

        BacktestMetrics {
            total_return,
            total_return_percentage,
            annualized_return,
            sharpe_ratio,
            max_drawdown,
            volatility,
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            average_win,
            average_loss,
            profit_factor,
            final_portfolio_value: final_value,
            benchmark_return: None, // TODO: Implement benchmark comparison
            alpha: None,            // TODO: Calculate alpha
            beta: None,             // TODO: Calculate beta
        }
    }

    fn calculate_max_drawdown(&self, trades: &[BacktestTrade]) -> Decimal {
        let mut max_value = Decimal::ZERO;
        let mut max_drawdown = Decimal::ZERO;

        for trade in trades {
            if trade.portfolio_value > max_value {
                max_value = trade.portfolio_value;
            }

            let current_drawdown = (max_value - trade.portfolio_value) / max_value * Decimal::from(100);
            if current_drawdown > max_drawdown {
                max_drawdown = current_drawdown;
            }
        }

        max_drawdown
    }

    fn calculate_volatility(&self, historical_data: &[Kline]) -> Decimal {
        if historical_data.len() < 2 {
            return Decimal::ZERO;
        }

        let returns: Vec<Decimal> = historical_data
            .windows(2)
            .map(|window| {
                let prev_price = window[0].close;
                let curr_price = window[1].close;
                (curr_price - prev_price) / prev_price
            })
            .collect();

        let mean_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        let variance = returns
            .iter()
            .map(|r| {
                let diff = *r - mean_return;
                diff * diff // Use multiplication instead of powi
            })
            .sum::<Decimal>() / Decimal::from(returns.len());

        // Convert to annualized volatility (approximate)
        // Use a simple approximation for square root for now
        let volatility_daily = variance; // Simplified - would need proper sqrt implementation
        volatility_daily * Decimal::from(16) * Decimal::from(100) // Approximate sqrt(252) ≈ 16
    }
}

impl Default for BacktestEngine {
    fn default() -> Self {
        Self::new()
    }
}