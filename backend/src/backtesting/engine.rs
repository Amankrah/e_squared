use std::sync::Arc;
use std::time::Instant;
use rust_decimal::{Decimal, prelude::*};
use tracing::{info, debug, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::backtesting::types::*;
use std::collections::VecDeque;
use crate::backtesting::binance_fetcher::BinanceFetcher;
use crate::backtesting::stock_fetcher::StockFetcher;
use crate::strategies::{Strategy, create_strategy, StrategySignal, StrategySignalType, QuantityType, StrategyMode, StrategyContext, MarketData};
use crate::strategies::core::traits::{OrderUpdate, OrderStatus, OrderType as TraitsOrderType};
use crate::exchange_connectors::{Kline};
use crate::utils::errors::AppError;

/// Backtesting engine with integrated caching and optimization
pub struct BacktestEngine {
    binance_fetcher: Arc<BinanceFetcher>,
    stock_fetcher: Option<Arc<StockFetcher>>,
}

impl BacktestEngine {
    /// Create a new backtest engine
    pub fn new() -> Self {
        Self {
            binance_fetcher: Arc::new(BinanceFetcher::new()),
            stock_fetcher: None,
        }
    }

    /// Create a new backtest engine with stock support
    pub fn new_with_stock_support(alpha_vantage_api_key: String) -> Self {
        Self {
            binance_fetcher: Arc::new(BinanceFetcher::new()),
            stock_fetcher: Some(Arc::new(StockFetcher::new(alpha_vantage_api_key))),
        }
    }

    /// Run a backtest with the given configuration
    pub async fn run_backtest(
        &self,
        config: BacktestConfig,
    ) -> Result<BacktestResult, AppError> {
        let start_time = Instant::now();

        info!(
            "Starting backtest for {} on {} from {} to {}",
            config.strategy_name, config.symbol, config.start_time, config.end_time
        );

        // Validate inputs
        self.validate_config(&config)?;

        // Create strategy instance
        let mut strategy = create_strategy(&config.strategy_name)?;

        // Fetch historical data based on asset type
        let historical_data = match config.asset_type.as_str() {
            "stock" => {
                let fetcher = self.stock_fetcher.as_ref()
                    .ok_or_else(|| AppError::InternalServerError)?;
                fetcher.fetch_klines(
                    &config.symbol,
                    &config.interval,
                    config.start_time,
                    config.end_time,
                ).await?
            }
            "crypto" | _ => {
                self.binance_fetcher.fetch_klines(
                    &config.symbol,
                    &config.interval,
                    config.start_time,
                    config.end_time,
                ).await?
            }
        };

        if historical_data.is_empty() {
            return Err(AppError::BadRequest(
                "No historical data available for the given period".to_string(),
            ));
        }

        debug!("Fetched {} klines for backtesting", historical_data.len());

        // Run the backtest simulation
        let (trades, portfolio, open_positions) = self.run_simulation(
            &historical_data,
            &mut *strategy,
            config.initial_balance,
            &config,
        ).await?;

        // Calculate comprehensive metrics
        let metrics = self.calculate_metrics(
            &trades,
            &portfolio,
            &historical_data,
            &config,
        );

        let execution_time = start_time.elapsed().as_millis() as u64;

        info!(
            "Backtest completed in {}ms. Final portfolio value: {} ({:+.2}%)",
            execution_time,
            portfolio.total_value,
            metrics.total_return_percentage
        );

        let performance_chart = self.generate_performance_chart(&trades, &historical_data);

        Ok(BacktestResult {
            config,
            trades,
            metrics,
            performance_chart,
            execution_time_ms: execution_time,
            open_positions,
        })
    }

    /// Validate backtest configuration
    fn validate_config(&self, config: &BacktestConfig) -> Result<(), AppError> {
        // Validate symbol
        BinanceFetcher::validate_symbol(&config.symbol)?;

        // Validate time range
        if config.start_time >= config.end_time {
            return Err(AppError::BadRequest(
                "Start time must be before end time".to_string(),
            ));
        }

        // Check if time range is reasonable (max 5 years to prevent excessive API calls)
        let max_days = 1825; // ~5 years
        let days_diff = (config.end_time - config.start_time).num_days();
        if days_diff > max_days {
            return Err(AppError::BadRequest(format!(
                "Time range too large. Maximum {} days (~5 years) allowed",
                max_days
            )));
        }

        // Warn if using very long timeframes with small intervals
        if days_diff > 730 && config.interval.as_str() == "1h" {
            warn!("Long backtest period ({} days) with 1h interval may be slow", days_diff);
        }

        // Validate initial balance
        if config.initial_balance <= Decimal::ZERO {
            return Err(AppError::BadRequest(
                "Initial balance must be positive".to_string(),
            ));
        }

        // Check for minimum balance
        let min_balance = Decimal::from(100);
        if config.initial_balance < min_balance {
            return Err(AppError::BadRequest(format!(
                "Minimum initial balance is {}",
                min_balance
            )));
        }

        Ok(())
    }

    /// Run the backtest simulation
    async fn run_simulation(
        &self,
        historical_data: &[Kline],
        strategy: &mut dyn Strategy,
        initial_balance: Decimal,
        config: &BacktestConfig,
    ) -> Result<(Vec<BacktestTrade>, Portfolio, Vec<OpenPosition>), AppError> {
        let mut portfolio = Portfolio::new(initial_balance);
        let mut trades = Vec::new();
        let mut position_tracker = PositionTracker::new();
        let mut open_positions: VecDeque<OpenPosition> = VecDeque::new();

        debug!("BACKTEST START - Initial Balance: ${}, Strategy: {}", initial_balance, config.strategy_name);

        // Create basic strategy context for initialization
        let init_context = StrategyContext {
            strategy_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            symbol: config.symbol.clone(),
            interval: config.interval.to_string(),
            mode: StrategyMode::Backtest,
            current_time: Utc::now(),
            historical_data: historical_data.to_vec(),
            current_price: historical_data.first().map(|k| k.close).unwrap_or(Decimal::ZERO),
            available_balance: config.initial_balance,
            current_positions: Vec::new(),
            market_data: MarketData::default(),
        };

        // Initialize strategy
        strategy.initialize(&config.strategy_parameters, StrategyMode::Backtest, &init_context).await?;

        // Process each kline
        for (index, kline) in historical_data.iter().enumerate() {
            // Update portfolio value
            portfolio.update_total_value(kline.close);

            // Create context for this analysis
            let context = StrategyContext {
                strategy_id: init_context.strategy_id,
                user_id: init_context.user_id,
                symbol: config.symbol.clone(),
                interval: config.interval.to_string(),
                mode: StrategyMode::Backtest,
                current_time: kline.close_time,
                historical_data: historical_data[..=index].to_vec(),
                current_price: kline.close,
                available_balance: portfolio.cash_balance,
                current_positions: Vec::new(), // TODO: Convert from position tracker
                market_data: MarketData::default(),
            };

            // Get strategy signal
            let signal_result = strategy.analyze(&context).await;

            // Execute trades based on signal
            if let Ok(Some(signal)) = signal_result {
                if let Some(trade) = self.execute_signal(
                    signal,
                    kline,
                    &mut portfolio,
                    &mut position_tracker,
                    &mut open_positions,
                    "Strategy signal".to_string(),
                    strategy,
                    &config.symbol,
                    &config,
                ).await {
                    trades.push(trade);
                }
            }

            // Check stop loss and take profit
            self.check_exit_conditions(
                kline,
                &mut portfolio,
                &mut position_tracker,
                &mut trades,
                config,
            );
        }

        // Close any remaining positions at the end
        if let Some(last_kline) = historical_data.last() {
            if position_tracker.has_position() {
                let close_trade = self.close_position(
                    last_kline,
                    &mut portfolio,
                    &mut position_tracker,
                    "End of backtest period",
                );
                if let Some(trade) = close_trade {
                    trades.push(trade);
                }
            }
            portfolio.update_total_value(last_kline.close);
        }

        debug!("BACKTEST END - Final Portfolio Value: ${}, Cash: ${}, Asset Quantity: {}, Total Invested: ${}, Open Positions: {}",
               portfolio.total_value, portfolio.cash_balance, portfolio.asset_quantity, portfolio.total_invested, open_positions.len());

        Ok((trades, portfolio, open_positions.into_iter().collect()))
    }

    /// Execute a trading signal
    async fn execute_signal(
        &self,
        signal: StrategySignal,
        kline: &Kline,
        portfolio: &mut Portfolio,
        position_tracker: &mut PositionTracker,
        open_positions: &mut VecDeque<OpenPosition>,
        reason: String,
        strategy: &mut dyn Strategy,
        symbol: &str,
        backtest_config: &BacktestConfig,
    ) -> Option<BacktestTrade> {
        match signal.signal_type {
            StrategySignalType::Enter => {
                let amount = match &signal.action.quantity {
                    QuantityType::DollarAmount(amt) => *amt,
                    QuantityType::Fixed(qty) => *qty * kline.close,
                    QuantityType::BalancePercentage(pct) => portfolio.cash_balance * pct / Decimal::from(100),
                    _ => Decimal::from(100), // Default amount
                };
                // Check if we already have a position
                if position_tracker.has_position() {
                    debug!("Skipping buy signal - already have position");
                    return None;
                }

                let quantity = amount / kline.close;
                if portfolio.execute_buy(kline.close, quantity) {
                    position_tracker.open_position(kline.close, quantity);

                    // Notify strategy about order execution
                    let order_update = OrderUpdate {
                        order_id: Uuid::new_v4().to_string(),
                        symbol: symbol.to_string(),
                        order_type: TraitsOrderType::Market,
                        status: OrderStatus::Filled,
                        quantity,
                        price: Some(kline.close),
                        filled_quantity: quantity,
                        timestamp: kline.close_time,
                    };
                    let _ = strategy.on_order_update(&order_update).await;

                    let trade = BacktestTrade {
                        timestamp: kline.close_time,
                        trade_type: TradeType::Buy,
                        price: kline.close,
                        quantity,
                        total_value: amount,
                        portfolio_value: portfolio.total_value,
                        balance_remaining: portfolio.cash_balance,
                        reason,
                        pnl: None,
                        pnl_percentage: None,
                    };

                    debug!("Executed BUY: {} @ {}", quantity, kline.close);
                    Some(trade)
                } else {
                    warn!("Failed to execute buy - insufficient balance");
                    None
                }
            }
            StrategySignalType::AddToPosition => {
                // DCA-style accumulation - buy regardless of current position
                let amount = match &signal.action.quantity {
                    QuantityType::DollarAmount(amt) => *amt,
                    QuantityType::Fixed(qty) => *qty * kline.close,
                    QuantityType::BalancePercentage(pct) => portfolio.cash_balance * pct / Decimal::from(100),
                    _ => Decimal::from(100), // Default amount
                };

                let quantity = amount / kline.close;
                debug!("DCA buy attempt: amount=${}, price={}, quantity={}, cash_balance={}",
                       amount, kline.close, quantity, portfolio.cash_balance);

                // For DCA strategies, use unlimited capital mode (continuous investment simulation)
                // Grid trading should use actual capital constraint
                let is_grid_trading = backtest_config.strategy_name.contains("grid");
                let buy_success = if !is_grid_trading && (backtest_config.unlimited_capital || backtest_config.strategy_name.contains("dca")) {
                    portfolio.execute_buy_with_injection(kline.close, quantity);
                    true
                } else {
                    portfolio.execute_buy(kline.close, quantity)
                };

                if buy_success {
                    // Update position tracker with new average entry price
                    position_tracker.add_to_position(kline.close, quantity);

                    // Track this as an open position
                    open_positions.push_back(OpenPosition {
                        timestamp: kline.close_time,
                        price: kline.close,
                        quantity,
                        total_value: amount,
                        reason: reason.clone(),
                    });

                    debug!("BUY EXECUTED - Amount: ${}, Quantity: {}, Price: {}, Cash Remaining: ${}, Total Invested: ${}, Open Positions: {}",
                           amount, quantity, kline.close, portfolio.cash_balance, portfolio.total_invested, open_positions.len());

                    // Notify strategy about order execution
                    let order_update = OrderUpdate {
                        order_id: Uuid::new_v4().to_string(),
                        symbol: symbol.to_string(),
                        order_type: TraitsOrderType::Market,
                        status: OrderStatus::Filled,
                        quantity,
                        price: Some(kline.close),
                        filled_quantity: quantity,
                        timestamp: kline.close_time,
                    };
                    let _ = strategy.on_order_update(&order_update).await;

                    let trade = BacktestTrade {
                        timestamp: kline.close_time,
                        trade_type: TradeType::Buy,
                        price: kline.close,
                        quantity,
                        total_value: amount,
                        portfolio_value: portfolio.total_value,
                        balance_remaining: portfolio.cash_balance,
                        reason,
                        pnl: None,
                        pnl_percentage: None,
                    };

                    debug!("Executed DCA BUY: {} @ {} (total position: {}, open positions: {})",
                           quantity, kline.close, position_tracker.entry_quantity, open_positions.len());
                    Some(trade)
                } else {
                    warn!("Failed to execute DCA buy - insufficient balance");
                    None
                }
            }
            StrategySignalType::Exit => {
                let quantity = match &signal.action.quantity {
                    QuantityType::Fixed(qty) => *qty,
                    QuantityType::PositionPercentage(pct) => position_tracker.entry_quantity * pct / Decimal::from(100),
                    QuantityType::AllPosition => position_tracker.entry_quantity,
                    _ => position_tracker.entry_quantity, // Default to full position
                };
                // Check if we have a position to sell
                if !position_tracker.has_position() {
                    debug!("Skipping sell signal - no position");
                    return None;
                }

                let actual_quantity = quantity.min(portfolio.asset_quantity);
                if portfolio.execute_sell(kline.close, actual_quantity) {
                    let pnl_data = position_tracker.close_position(kline.close, actual_quantity);

                    let trade = BacktestTrade {
                        timestamp: kline.close_time,
                        trade_type: TradeType::Sell,
                        price: kline.close,
                        quantity: actual_quantity,
                        total_value: actual_quantity * kline.close,
                        portfolio_value: portfolio.total_value,
                        balance_remaining: portfolio.cash_balance,
                        reason,
                        pnl: pnl_data.0,
                        pnl_percentage: pnl_data.1,
                    };

                    debug!(
                        "Executed SELL: {} @ {} (PnL: {:+.2})",
                        actual_quantity,
                        kline.close,
                        pnl_data.0.unwrap_or(Decimal::ZERO)
                    );
                    Some(trade)
                } else {
                    warn!("Failed to execute sell");
                    None
                }
            }
            StrategySignalType::ReducePosition => {
                // Grid trading style - sell from position
                let quantity = match &signal.action.quantity {
                    QuantityType::Fixed(qty) => *qty,
                    QuantityType::PositionPercentage(pct) => position_tracker.entry_quantity * pct / Decimal::from(100),
                    _ => position_tracker.entry_quantity, // Default to full position
                };

                // Check if we have enough to sell
                if portfolio.asset_quantity < quantity {
                    debug!("Skipping sell signal - insufficient asset quantity ({} < {})", portfolio.asset_quantity, quantity);
                    return None;
                }

                if portfolio.execute_sell(kline.close, quantity) {
                    let pnl_data = position_tracker.close_position(kline.close, quantity);

                    // Remove from open positions (FIFO - remove oldest first)
                    let mut remaining_to_close = quantity;
                    while remaining_to_close > Decimal::ZERO && !open_positions.is_empty() {
                        if let Some(mut open_pos) = open_positions.pop_front() {
                            if open_pos.quantity <= remaining_to_close {
                                // Close this entire position
                                remaining_to_close -= open_pos.quantity;
                            } else {
                                // Partially close this position
                                open_pos.quantity -= remaining_to_close;
                                open_pos.total_value = open_pos.quantity * open_pos.price;
                                open_positions.push_front(open_pos);
                                remaining_to_close = Decimal::ZERO;
                            }
                        }
                    }

                    // Notify strategy about order execution
                    let order_update = OrderUpdate {
                        order_id: Uuid::new_v4().to_string(),
                        symbol: symbol.to_string(),
                        order_type: TraitsOrderType::Market,
                        status: OrderStatus::Filled,
                        quantity,
                        price: Some(kline.close),
                        filled_quantity: quantity,
                        timestamp: kline.close_time,
                    };
                    let _ = strategy.on_order_update(&order_update).await;

                    let trade = BacktestTrade {
                        timestamp: kline.close_time,
                        trade_type: TradeType::Sell,
                        price: kline.close,
                        quantity,
                        total_value: quantity * kline.close,
                        portfolio_value: portfolio.total_value,
                        balance_remaining: portfolio.cash_balance,
                        reason,
                        pnl: pnl_data.0,
                        pnl_percentage: pnl_data.1,
                    };

                    debug!("Executed GRID SELL: {} @ {} (open positions remaining: {})", quantity, kline.close, open_positions.len());
                    Some(trade)
                } else {
                    warn!("Failed to execute grid sell");
                    None
                }
            }
            _ => {
                // Handle other signal types
                debug!("Ignoring unsupported signal type: {:?}", signal.signal_type);
                None
            }
        }
    }

    /// Check and execute stop loss and take profit conditions
    fn check_exit_conditions(
        &self,
        kline: &Kline,
        portfolio: &mut Portfolio,
        position_tracker: &mut PositionTracker,
        trades: &mut Vec<BacktestTrade>,
        config: &BacktestConfig,
    ) {
        if !position_tracker.has_position() {
            return;
        }

        let entry_price = position_tracker.entry_price;
        let current_price = kline.close;
        let price_change_pct = ((current_price - entry_price) / entry_price) * Decimal::from(100);

        // Check stop loss
        if let Some(stop_loss_pct) = config.stop_loss_percentage {
            if price_change_pct <= -stop_loss_pct {
                if let Some(trade) = self.close_position(
                    kline,
                    portfolio,
                    position_tracker,
                    &format!("Stop loss triggered at {:.2}%", price_change_pct),
                ) {
                    trades.push(trade);
                }
            }
        }

        // Check take profit
        if let Some(take_profit_pct) = config.take_profit_percentage {
            if price_change_pct >= take_profit_pct {
                if let Some(trade) = self.close_position(
                    kline,
                    portfolio,
                    position_tracker,
                    &format!("Take profit triggered at {:.2}%", price_change_pct),
                ) {
                    trades.push(trade);
                }
            }
        }
    }

    /// Close current position
    fn close_position(
        &self,
        kline: &Kline,
        portfolio: &mut Portfolio,
        position_tracker: &mut PositionTracker,
        reason: &str,
    ) -> Option<BacktestTrade> {
        if !position_tracker.has_position() {
            return None;
        }

        let quantity = portfolio.asset_quantity;
        if portfolio.execute_sell(kline.close, quantity) {
            let pnl_data = position_tracker.close_position(kline.close, quantity);

            Some(BacktestTrade {
                timestamp: kline.close_time,
                trade_type: TradeType::Sell,
                price: kline.close,
                quantity,
                total_value: quantity * kline.close,
                portfolio_value: portfolio.total_value,
                balance_remaining: portfolio.cash_balance,
                reason: reason.to_string(),
                pnl: pnl_data.0,
                pnl_percentage: pnl_data.1,
            })
        } else {
            None
        }
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
        let total_invested = portfolio.total_invested;

        // Basic returns calculation:
        // - For unlimited capital mode (DCA): calculate based on total invested
        // - For fixed capital (grid trading, normal strategies): use initial value
        let base_amount = if config.unlimited_capital && total_invested > Decimal::ZERO {
            total_invested
        } else {
            initial_value
        };

        let total_return = final_value - base_amount;
        let total_return_percentage = if base_amount > Decimal::ZERO {
            (total_return / base_amount) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        // Trade statistics - count all trades but only sell trades for win/loss
        let total_trades = trades.len() as u32;
        let mut winning_trades = 0u32;
        let mut losing_trades = 0u32;
        let mut total_wins = Decimal::ZERO;
        let mut total_losses = Decimal::ZERO;
        let mut closed_trades = 0u32; // Trades with realized P&L (sells)

        for trade in trades {
            if let Some(pnl) = trade.pnl {
                closed_trades += 1;
                if pnl > Decimal::ZERO {
                    winning_trades += 1;
                    total_wins += pnl;
                } else if pnl < Decimal::ZERO {
                    losing_trades += 1;
                    total_losses += pnl.abs();
                }
            }
        }

        // Win rate should be based on closed trades (sells) only, not all trades
        let win_rate = if closed_trades > 0 {
            Decimal::from(winning_trades) / Decimal::from(closed_trades) * Decimal::from(100)
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
        } else if total_wins > Decimal::ZERO {
            Some(Decimal::from(999)) // Max profit factor
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

        // Sharpe ratio (simplified)
        let risk_free_rate = Decimal::from_str("2.0").unwrap(); // 2% annual risk-free rate
        let sharpe_ratio = if volatility > Decimal::ZERO && annualized_return.is_some() {
            Some((annualized_return.unwrap() - risk_free_rate) / volatility)
        } else {
            None
        };

        // Calculate buy & hold return for benchmark
        let benchmark_return = if let (Some(first), Some(last)) =
            (historical_data.first(), historical_data.last())
        {
            let buy_hold_return = ((last.close - first.close) / first.close) * Decimal::from(100);
            Some(buy_hold_return)
        } else {
            None
        };

        // Calculate open trades (buys without sells) and unrealized P&L
        let open_trades = total_trades - closed_trades;

        // Calculate realized P&L (from closed trades only)
        let realized_pnl = total_wins - total_losses;

        // Calculate unrealized P&L (total return minus realized P&L)
        let unrealized_pnl = total_return - realized_pnl;

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
            benchmark_return,
            alpha: None, // Can be calculated if needed
            beta: None,  // Can be calculated if needed
            total_invested,
            closed_trades,
            open_trades,
            realized_pnl,
            unrealized_pnl,
        }
    }

    /// Calculate maximum drawdown
    fn calculate_max_drawdown(&self, trades: &[BacktestTrade]) -> Decimal {
        if trades.is_empty() {
            return Decimal::ZERO;
        }

        let mut max_value = trades[0].portfolio_value;
        let mut max_drawdown = Decimal::ZERO;

        for trade in trades {
            if trade.portfolio_value > max_value {
                max_value = trade.portfolio_value;
            }

            let current_drawdown = if max_value > Decimal::ZERO {
                ((max_value - trade.portfolio_value) / max_value) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            if current_drawdown > max_drawdown {
                max_drawdown = current_drawdown;
            }
        }

        max_drawdown
    }

    /// Calculate volatility (annualized)
    fn calculate_volatility(&self, historical_data: &[Kline]) -> Decimal {
        if historical_data.len() < 2 {
            return Decimal::ZERO;
        }

        // Calculate daily returns
        let returns: Vec<Decimal> = historical_data
            .windows(2)
            .map(|window| {
                let prev_price = window[0].close;
                let curr_price = window[1].close;
                if prev_price > Decimal::ZERO {
                    (curr_price - prev_price) / prev_price
                } else {
                    Decimal::ZERO
                }
            })
            .collect();

        if returns.is_empty() {
            return Decimal::ZERO;
        }

        // Calculate mean return
        let mean_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

        // Calculate variance
        let variance = returns
            .iter()
            .map(|r| {
                let diff = *r - mean_return;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        // Approximate annualized volatility
        // sqrt(variance) * sqrt(252) ≈ variance^0.5 * 15.87
        // Using approximation since Decimal doesn't have sqrt
        let daily_vol = variance.to_f64().unwrap_or(0.0).sqrt();
        let annual_vol = daily_vol * (252_f64).sqrt() * 100.0;

        Decimal::from_f64(annual_vol).unwrap_or(Decimal::ZERO)
    }

    /// Generate performance chart data
    fn generate_performance_chart(
        &self,
        trades: &[BacktestTrade],
        historical_data: &[Kline],
    ) -> Vec<PerformancePoint> {
        let mut chart_data = Vec::new();

        // Add initial point
        if let Some(first_kline) = historical_data.first() {
            chart_data.push(PerformancePoint {
                timestamp: first_kline.open_time,
                portfolio_value: Decimal::from(10000), // Assuming initial value
                asset_price: first_kline.close,
                trade_marker: None,
            });
        }

        // Add trade points
        for trade in trades {
            chart_data.push(PerformancePoint {
                timestamp: trade.timestamp,
                portfolio_value: trade.portfolio_value,
                asset_price: trade.price,
                trade_marker: Some(trade.trade_type.clone()),
            });
        }

        chart_data
    }
}

/// Position tracker for managing open positions
#[derive(Debug, Clone)]
struct PositionTracker {
    entry_price: Decimal,
    entry_quantity: Decimal,
    is_open: bool,
}

impl PositionTracker {
    fn new() -> Self {
        Self {
            entry_price: Decimal::ZERO,
            entry_quantity: Decimal::ZERO,
            is_open: false,
        }
    }

    fn has_position(&self) -> bool {
        self.is_open
    }

    fn open_position(&mut self, price: Decimal, quantity: Decimal) {
        self.entry_price = price;
        self.entry_quantity = quantity;
        self.is_open = true;
    }

    fn add_to_position(&mut self, price: Decimal, quantity: Decimal) {
        if !self.is_open {
            // If no position exists, treat as opening position
            self.open_position(price, quantity);
        } else {
            // Calculate new average entry price
            let total_cost = (self.entry_price * self.entry_quantity) + (price * quantity);
            let total_quantity = self.entry_quantity + quantity;

            if total_quantity > Decimal::ZERO {
                self.entry_price = total_cost / total_quantity;
                self.entry_quantity = total_quantity;
            }
        }
    }

    fn close_position(&mut self, exit_price: Decimal, quantity: Decimal) -> (Option<Decimal>, Option<Decimal>) {
        if !self.is_open {
            return (None, None);
        }

        let pnl = (exit_price - self.entry_price) * quantity;
        let pnl_percentage = if self.entry_price > Decimal::ZERO {
            ((exit_price - self.entry_price) / self.entry_price) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        self.is_open = false;
        self.entry_price = Decimal::ZERO;
        self.entry_quantity = Decimal::ZERO;

        (Some(pnl), Some(pnl_percentage))
    }
}

impl Default for BacktestEngine {
    fn default() -> Self {
        Self::new()
    }
}