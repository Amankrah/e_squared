use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::exchange_connectors::Kline;
use super::traits::{StrategyMode, Position, MarketData};

/// Context builder for creating strategy contexts
pub struct StrategyContextBuilder {
    strategy_id: Option<Uuid>,
    user_id: Option<Uuid>,
    symbol: Option<String>,
    interval: Option<String>,
    mode: Option<StrategyMode>,
    current_time: Option<DateTime<Utc>>,
    historical_data: Vec<Kline>,
    current_price: Option<Decimal>,
    available_balance: Option<Decimal>,
    current_positions: Vec<Position>,
    market_data: MarketData,
}

impl StrategyContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            strategy_id: None,
            user_id: None,
            symbol: None,
            interval: None,
            mode: None,
            current_time: None,
            historical_data: Vec::new(),
            current_price: None,
            available_balance: None,
            current_positions: Vec::new(),
            market_data: MarketData::default(),
        }
    }

    /// Set strategy ID
    pub fn strategy_id(mut self, strategy_id: Uuid) -> Self {
        self.strategy_id = Some(strategy_id);
        self
    }

    /// Set user ID
    pub fn user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set trading symbol
    pub fn symbol(mut self, symbol: String) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Set time interval
    pub fn interval(mut self, interval: String) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set strategy mode
    pub fn mode(mut self, mode: StrategyMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set current time
    pub fn current_time(mut self, current_time: DateTime<Utc>) -> Self {
        self.current_time = Some(current_time);
        self
    }

    /// Set historical data
    pub fn historical_data(mut self, historical_data: Vec<Kline>) -> Self {
        self.historical_data = historical_data;
        self
    }

    /// Add historical data
    pub fn add_historical_data(mut self, klines: &[Kline]) -> Self {
        self.historical_data.extend_from_slice(klines);
        self
    }

    /// Set current price
    pub fn current_price(mut self, current_price: Decimal) -> Self {
        self.current_price = Some(current_price);
        self
    }

    /// Set available balance
    pub fn available_balance(mut self, available_balance: Decimal) -> Self {
        self.available_balance = Some(available_balance);
        self
    }

    /// Set current positions
    pub fn current_positions(mut self, current_positions: Vec<Position>) -> Self {
        self.current_positions = current_positions;
        self
    }

    /// Add a position
    pub fn add_position(mut self, position: Position) -> Self {
        self.current_positions.push(position);
        self
    }

    /// Set market data
    pub fn market_data(mut self, market_data: MarketData) -> Self {
        self.market_data = market_data;
        self
    }

    /// Set 24h volume
    pub fn volume_24h(mut self, volume: Decimal) -> Self {
        self.market_data.volume_24h = Some(volume);
        self
    }

    /// Set 24h price change
    pub fn price_change_24h(mut self, change: Decimal) -> Self {
        self.market_data.price_change_24h = Some(change);
        self
    }

    /// Set bid price
    pub fn bid_price(mut self, bid: Decimal) -> Self {
        self.market_data.bid_price = Some(bid);
        self
    }

    /// Set ask price
    pub fn ask_price(mut self, ask: Decimal) -> Self {
        self.market_data.ask_price = Some(ask);
        self
    }

    /// Build the context
    pub fn build(self) -> Result<super::traits::StrategyContext, crate::utils::errors::AppError> {
        let strategy_id = self.strategy_id
            .ok_or_else(|| crate::utils::errors::AppError::BadRequest("Strategy ID is required".to_string()))?;

        let user_id = self.user_id
            .ok_or_else(|| crate::utils::errors::AppError::BadRequest("User ID is required".to_string()))?;

        let symbol = self.symbol
            .ok_or_else(|| crate::utils::errors::AppError::BadRequest("Symbol is required".to_string()))?;

        let interval = self.interval
            .ok_or_else(|| crate::utils::errors::AppError::BadRequest("Interval is required".to_string()))?;

        let mode = self.mode
            .ok_or_else(|| crate::utils::errors::AppError::BadRequest("Mode is required".to_string()))?;

        let current_time = self.current_time.unwrap_or_else(|| Utc::now());

        let current_price = self.current_price
            .or_else(|| self.historical_data.last().map(|k| k.close))
            .unwrap_or(Decimal::ZERO);

        let available_balance = self.available_balance.unwrap_or(Decimal::ZERO);

        // Calculate spread if bid/ask are available
        let mut market_data = self.market_data;
        if let (Some(bid), Some(ask)) = (market_data.bid_price, market_data.ask_price) {
            market_data.spread = Some(ask - bid);
        }

        Ok(super::traits::StrategyContext {
            strategy_id,
            user_id,
            symbol,
            interval,
            mode,
            current_time,
            historical_data: self.historical_data,
            current_price,
            available_balance,
            current_positions: self.current_positions,
            market_data,
        })
    }
}

impl Default for StrategyContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for context manipulation
impl super::traits::StrategyContext {
    /// Get the latest kline data
    pub fn latest_kline(&self) -> Option<&Kline> {
        self.historical_data.last()
    }

    /// Get klines for a specific period (last N klines)
    pub fn recent_klines(&self, count: usize) -> &[Kline] {
        let start = self.historical_data.len().saturating_sub(count);
        &self.historical_data[start..]
    }

    /// Get position for the current symbol
    pub fn current_symbol_position(&self) -> Option<&Position> {
        self.current_positions.iter()
            .find(|p| p.symbol == self.symbol)
    }

    /// Check if we have a position in the current symbol
    pub fn has_position(&self) -> bool {
        self.current_symbol_position().is_some()
    }

    /// Get total position value for current symbol
    pub fn position_value(&self) -> Decimal {
        self.current_symbol_position()
            .map(|p| p.quantity * p.current_price)
            .unwrap_or(Decimal::ZERO)
    }

    /// Get unrealized PnL for current symbol
    pub fn unrealized_pnl(&self) -> Decimal {
        self.current_symbol_position()
            .map(|p| p.pnl)
            .unwrap_or(Decimal::ZERO)
    }

    /// Get unrealized PnL percentage for current symbol
    pub fn unrealized_pnl_percentage(&self) -> Decimal {
        self.current_symbol_position()
            .map(|p| p.pnl_percentage)
            .unwrap_or(Decimal::ZERO)
    }

    /// Check if enough historical data is available
    pub fn has_min_data(&self, min_count: usize) -> bool {
        self.historical_data.len() >= min_count
    }

    /// Get current spread percentage
    pub fn spread_percentage(&self) -> Option<Decimal> {
        if let (Some(spread), price) = (self.market_data.spread, self.current_price) {
            if price > Decimal::ZERO {
                Some((spread / price) * Decimal::from(100))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if market conditions are favorable for trading
    pub fn is_market_liquid(&self, max_spread_percentage: Decimal) -> bool {
        self.spread_percentage()
            .map(|spread| spread <= max_spread_percentage)
            .unwrap_or(true) // Assume liquid if spread data unavailable
    }

    /// Update context with new market data
    pub fn update_market_data(&mut self, market_data: MarketData) {
        self.market_data = market_data;
    }

    /// Update current price
    pub fn update_current_price(&mut self, price: Decimal) {
        self.current_price = price;
        self.current_time = Utc::now();
    }

    /// Add new historical data point
    pub fn add_kline(&mut self, kline: Kline) {
        self.historical_data.push(kline);

        // Keep only last 1000 klines to prevent memory growth
        if self.historical_data.len() > 1000 {
            self.historical_data.remove(0);
        }
    }

    /// Update position information
    pub fn update_positions(&mut self, positions: Vec<Position>) {
        self.current_positions = positions;
    }

    /// Update available balance
    pub fn update_balance(&mut self, balance: Decimal) {
        self.available_balance = balance;
    }

    /// Create a context for backtesting
    pub fn for_backtest(
        strategy_id: Uuid,
        user_id: Uuid,
        symbol: String,
        interval: String,
        historical_data: Vec<Kline>,
        initial_balance: Decimal,
    ) -> Result<Self, crate::utils::errors::AppError> {
        StrategyContextBuilder::new()
            .strategy_id(strategy_id)
            .user_id(user_id)
            .symbol(symbol)
            .interval(interval)
            .mode(StrategyMode::Backtest)
            .historical_data(historical_data)
            .available_balance(initial_balance)
            .build()
    }

    /// Create a context for live trading
    pub fn for_live_trading(
        strategy_id: Uuid,
        user_id: Uuid,
        symbol: String,
        interval: String,
        current_price: Decimal,
        available_balance: Decimal,
        positions: Vec<Position>,
        market_data: MarketData,
    ) -> Result<Self, crate::utils::errors::AppError> {
        StrategyContextBuilder::new()
            .strategy_id(strategy_id)
            .user_id(user_id)
            .symbol(symbol)
            .interval(interval)
            .mode(StrategyMode::Live)
            .current_price(current_price)
            .available_balance(available_balance)
            .current_positions(positions)
            .market_data(market_data)
            .build()
    }
}