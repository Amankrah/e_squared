use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{info, warn};

use crate::strategies::core::{
    Strategy, StrategyMetadata, StrategyMode, StrategyContext, StrategySignal,
    StrategyCategory, RiskLevel, LiveExecutableStrategy, ControllableStrategy, IndicatorValue, QuantityType,
};
use crate::strategies::indicators::{self};
use crate::utils::errors::AppError;

use super::config::GridTradingConfig;
use super::types::*;

/// Grid Trading Strategy Implementation
pub struct GridTradingStrategy {
    /// Strategy configuration
    config: Option<GridTradingConfig>,
    /// Current execution state
    state: GridTradingState,
    /// Execution history
    execution_history: Vec<GridExecution>,
    /// Is strategy currently paused
    is_paused: bool,
    /// Is strategy currently running (for live execution)
    is_running: bool,
    /// Last signal reason
    last_signal_reason: String,
    /// Strategy metadata
    metadata: StrategyMetadata,
}

impl GridTradingStrategy {
    /// Create a new Grid Trading strategy instance
    pub fn new() -> Self {
        Self {
            config: None,
            state: GridTradingState::default(),
            execution_history: Vec::new(),
            is_paused: false,
            is_running: false,
            last_signal_reason: String::new(),
            metadata: Self::create_metadata(),
        }
    }

    /// Create strategy metadata
    pub fn create_metadata() -> StrategyMetadata {
        StrategyMetadata {
            id: "grid_trading_v2".to_string(),
            name: "Grid Trading v2".to_string(),
            description: "Advanced grid trading strategy with dynamic adjustment, market making capabilities, and comprehensive risk management".to_string(),
            version: "2.0.0".to_string(),
            author: "E-Squared Trading Bot".to_string(),
            category: StrategyCategory::GridTrading,
            risk_level: RiskLevel::Moderate,
            supported_modes: vec![
                StrategyMode::Backtest,
                StrategyMode::Paper,
                StrategyMode::Live,
            ],
            min_balance: Some(Decimal::from(1000)),
            max_positions: Some(50), // Can have multiple grid levels
            supported_intervals: vec![
                "1m".to_string(), "5m".to_string(), "15m".to_string(),
                "30m".to_string(), "1h".to_string(), "4h".to_string(),
                "1d".to_string()
            ],
            tags: vec![
                "grid".to_string(),
                "market_making".to_string(),
                "systematic".to_string(),
                "range_bound".to_string(),
                "automated".to_string(),
            ],
        }
    }

    /// Initialize grid levels based on current market price
    fn initialize_grid(&mut self, center_price: Decimal) -> Result<(), AppError> {
        let config = self.config.as_ref().unwrap();
        let mut levels = Vec::new();

        // Calculate grid bounds (with auto-adjustment if needed)
        let (upper_bound, lower_bound) = self.calculate_grid_bounds(center_price)?;

        info!("Initializing grid: center_price={}, bounds={}-{}", center_price, lower_bound, upper_bound);

        self.state.grid_center = center_price;
        self.state.grid_upper_bound = upper_bound;
        self.state.grid_lower_bound = lower_bound;

        // Generate grid levels based on spacing mode
        match config.spacing.mode {
            GridTradingMode::Standard => {
                let spacing = config.spacing.fixed_spacing.unwrap();
                self.create_standard_grid(&mut levels, center_price, spacing, upper_bound, lower_bound)?;
            }
            GridTradingMode::Arithmetic => {
                let base_spacing = config.spacing.fixed_spacing.unwrap();
                let increment = config.spacing.arithmetic_increment.unwrap_or(base_spacing * Decimal::new(1, 1));
                self.create_arithmetic_grid(&mut levels, center_price, base_spacing, increment, upper_bound, lower_bound)?;
            }
            GridTradingMode::Geometric => {
                let multiplier = config.spacing.geometric_multiplier.unwrap();
                self.create_geometric_grid(&mut levels, center_price, multiplier, upper_bound, lower_bound)?;
            }
            GridTradingMode::Dynamic => {
                let base_pct = config.spacing.dynamic_base_pct.unwrap();
                self.create_dynamic_grid(&mut levels, center_price, base_pct, upper_bound, lower_bound)?;
            }
            GridTradingMode::ZoneBased => {
                self.create_zone_based_grid(&mut levels, center_price, upper_bound, lower_bound)?;
            }
        }

        self.state.grid_levels = levels;
        self.state.is_active = true;

        info!("Grid initialized with {} levels between {} and {}",
              self.state.grid_levels.len(), lower_bound, upper_bound);

        Ok(())
    }

    /// Calculate grid bounds based on configuration
    fn calculate_grid_bounds(&self, center_price: Decimal) -> Result<(Decimal, Decimal), AppError> {
        let config = self.config.as_ref().unwrap();

        info!("calculate_grid_bounds: type={:?}, auto_adjust={}, upper={}, lower={}, center_price={}",
              config.bounds.bounds_type, config.bounds.auto_adjust,
              config.bounds.upper_bound, config.bounds.lower_bound, center_price);

        match config.bounds.bounds_type {
            BoundsType::AbsolutePrice => {
                // Check if current price is within configured bounds
                let configured_upper = config.bounds.upper_bound;
                let configured_lower = config.bounds.lower_bound;

                // If price is outside configured bounds, auto-adjust to use percentage-based bounds
                // This prevents constant rebalancing when market moves away from initial setup
                if center_price > configured_upper || center_price < configured_lower {
                    // Calculate percentage range from configured bounds
                    let configured_center = (configured_upper + configured_lower) / Decimal::from(2);
                    let range_pct = ((configured_upper - configured_lower) / configured_center) / Decimal::from(2);

                    // Apply same percentage to current price
                    let upper = center_price * (Decimal::ONE + range_pct);
                    let lower = center_price * (Decimal::ONE - range_pct);

                    info!("Auto-adjusted grid bounds from ({}-{}) to ({}-{}) based on current price {} ({}% range)",
                          configured_lower, configured_upper, lower, upper, center_price, range_pct * Decimal::from(100));

                    Ok((upper, lower))
                } else {
                    info!("Using configured bounds without adjustment: {}-{}", configured_lower, configured_upper);
                    Ok((configured_upper, configured_lower))
                }
            }
            BoundsType::PercentageFromCenter => {
                let upper = center_price * (Decimal::ONE + config.bounds.upper_bound / Decimal::from(100));
                let lower = center_price * (Decimal::ONE - config.bounds.lower_bound / Decimal::from(100));
                Ok((upper, lower))
            }
            BoundsType::VolatilityBased => {
                // Would need ATR calculation from indicators
                let volatility_estimate = center_price * Decimal::new(2, 2); // 2% default
                let upper = center_price + (volatility_estimate * config.bounds.upper_bound);
                let lower = center_price - (volatility_estimate * config.bounds.lower_bound);
                Ok((upper, lower))
            }
            BoundsType::TechnicalLevels => {
                // Would integrate with support/resistance calculation
                let upper = center_price * (Decimal::ONE + config.bounds.upper_bound / Decimal::from(100));
                let lower = center_price * (Decimal::ONE - config.bounds.lower_bound / Decimal::from(100));
                Ok((upper, lower))
            }
        }
    }

    /// Create standard grid with fixed spacing
    fn create_standard_grid(
        &self,
        levels: &mut Vec<GridLevel>,
        center_price: Decimal,
        spacing_pct: Decimal,
        upper_bound: Decimal,
        lower_bound: Decimal,
    ) -> Result<(), AppError> {
        let config = self.config.as_ref().unwrap();
        let order_size = config.calculate_order_size_per_level();

        // Calculate spacing based on the actual bounds range, not center price
        let price_range = upper_bound - lower_bound;
        let spacing_amount = price_range / Decimal::from(config.grid_levels - 1);

        // Create grid levels evenly distributed from lower to upper bound
        // Place levels across the entire range
        for i in 0..config.grid_levels {
            let price = lower_bound + (spacing_amount * Decimal::from(i));

            // Determine order type based on position relative to current price
            let order_type = if price < center_price {
                GridOrderType::Buy  // Buy orders below current price
            } else if price > center_price {
                GridOrderType::Sell // Sell orders above current price
            } else {
                // Price exactly at center - skip or make it a buy order
                continue;
            };

            levels.push(GridLevel {
                price,
                order_type,
                quantity: order_size,
                is_active: true,
                fill_count: 0,
                last_fill_time: None,
                total_filled: Decimal::ZERO,
            });
        }

        Ok(())
    }

    /// Create arithmetic progression grid
    fn create_arithmetic_grid(
        &self,
        levels: &mut Vec<GridLevel>,
        center_price: Decimal,
        base_spacing: Decimal,
        increment: Decimal,
        upper_bound: Decimal,
        lower_bound: Decimal,
    ) -> Result<(), AppError> {
        let config = self.config.as_ref().unwrap();
        let order_size = config.calculate_order_size_per_level();

        // Create buy levels below center
        let mut price = center_price;
        let mut spacing = base_spacing;
        let mut level_count = 0;

        while level_count < config.grid_levels / 2 {
            price -= center_price * spacing / Decimal::from(100);
            if price < lower_bound { break; }

            levels.push(GridLevel {
                price,
                order_type: GridOrderType::Buy,
                quantity: order_size,
                is_active: true,
                fill_count: 0,
                last_fill_time: None,
                total_filled: Decimal::ZERO,
            });

            spacing += increment;
            level_count += 1;
        }

        // Create sell levels above center
        let mut price = center_price;
        let mut spacing = base_spacing;

        while levels.len() < config.grid_levels {
            price += center_price * spacing / Decimal::from(100);
            if price > upper_bound { break; }

            levels.push(GridLevel {
                price,
                order_type: GridOrderType::Sell,
                quantity: order_size,
                is_active: true,
                fill_count: 0,
                last_fill_time: None,
                total_filled: Decimal::ZERO,
            });

            spacing += increment;
        }

        Ok(())
    }

    /// Create geometric progression grid
    fn create_geometric_grid(
        &self,
        levels: &mut Vec<GridLevel>,
        center_price: Decimal,
        multiplier: Decimal,
        upper_bound: Decimal,
        lower_bound: Decimal,
    ) -> Result<(), AppError> {
        let config = self.config.as_ref().unwrap();
        let order_size = config.calculate_order_size_per_level();
        let base_spacing = Decimal::new(1, 2); // 1% base

        // Create buy levels below center
        let mut price = center_price;
        let mut spacing_multiplier = Decimal::ONE;
        let mut level_count = 0;

        while level_count < config.grid_levels / 2 {
            let spacing = base_spacing * spacing_multiplier;
            price -= center_price * spacing / Decimal::from(100);
            if price < lower_bound { break; }

            levels.push(GridLevel {
                price,
                order_type: GridOrderType::Buy,
                quantity: order_size,
                is_active: true,
                fill_count: 0,
                last_fill_time: None,
                total_filled: Decimal::ZERO,
            });

            spacing_multiplier *= multiplier;
            level_count += 1;
        }

        // Create sell levels above center
        let mut price = center_price;
        let mut spacing_multiplier = Decimal::ONE;

        while levels.len() < config.grid_levels {
            let spacing = base_spacing * spacing_multiplier;
            price += center_price * spacing / Decimal::from(100);
            if price > upper_bound { break; }

            levels.push(GridLevel {
                price,
                order_type: GridOrderType::Sell,
                quantity: order_size,
                is_active: true,
                fill_count: 0,
                last_fill_time: None,
                total_filled: Decimal::ZERO,
            });

            spacing_multiplier *= multiplier;
        }

        Ok(())
    }

    /// Create dynamic grid based on volatility
    fn create_dynamic_grid(
        &self,
        levels: &mut Vec<GridLevel>,
        center_price: Decimal,
        base_pct: Decimal,
        upper_bound: Decimal,
        lower_bound: Decimal,
    ) -> Result<(), AppError> {
        // For now, use standard grid with dynamic base
        // In a full implementation, this would adjust based on calculated volatility
        self.create_standard_grid(levels, center_price, base_pct, upper_bound, lower_bound)
    }

    /// Create zone-based grid
    fn create_zone_based_grid(
        &self,
        levels: &mut Vec<GridLevel>,
        center_price: Decimal,
        upper_bound: Decimal,
        lower_bound: Decimal,
    ) -> Result<(), AppError> {
        // For now, use standard grid
        // In a full implementation, this would create zones based on support/resistance
        let spacing = Decimal::new(1, 2); // 1%
        self.create_standard_grid(levels, center_price, spacing, upper_bound, lower_bound)
    }

    /// Check if any grid levels should be filled
    fn check_grid_fills(&mut self, context: &StrategyContext) -> Vec<(usize, TradeSide)> {
        let current_price = context.current_price;
        let mut fills = Vec::new();
        let config = self.config.as_ref().unwrap();

        for (index, level) in self.state.grid_levels.iter().enumerate() {
            if !level.is_active {
                continue;
            }

            match level.order_type {
                GridOrderType::Buy => {
                    if current_price <= level.price {
                        // Check if we have sufficient balance for this buy
                        // The backtesting engine will also check this, but we check here
                        // to avoid generating signals that will fail
                        let order_cost = level.quantity; // quantity is dollar amount

                        // Allow buys only if we have sufficient balance
                        // Note: context.available_balance is updated by the backtesting engine
                        if context.available_balance >= order_cost {
                            fills.push((index, TradeSide::Buy));
                        }
                    }
                }
                GridOrderType::Sell => {
                    // Calculate how much BTC this sell would require
                    let btc_quantity_needed = level.quantity / current_price;

                    // Only allow sells if:
                    // 1. Market making is enabled (can short), OR
                    // 2. We have sufficient inventory to sell
                    let can_sell = config.market_making.enabled ||
                                   self.state.inventory >= btc_quantity_needed;

                    if can_sell && current_price >= level.price {
                        fills.push((index, TradeSide::Sell));
                    }
                }
                _ => {} // Handle other order types if needed
            }
        }

        fills
    }

    /// Execute grid level fill
    fn execute_grid_fill(&mut self, context: &StrategyContext, level_index: usize, side: TradeSide) -> Result<StrategySignal, AppError> {
        let price = context.current_price;
        let quantity;
        let order_type;

        // Extract values from level before mutable operations
        let level_price;
        {
            let level = &mut self.state.grid_levels[level_index];
            quantity = level.quantity;
            order_type = level.order_type.clone();
            level_price = level.price;
        }

        // quantity represents dollar amount per level, need to convert to BTC quantity
        let btc_quantity = quantity / price;

        // For sells, check if we have enough inventory
        // This prevents the strategy from getting out of sync with the actual portfolio
        if matches!(side, TradeSide::Sell) {
            if self.state.inventory < btc_quantity {
                warn!("Insufficient inventory for grid sell: have {}, need {}",
                      self.state.inventory, btc_quantity);
                return Err(AppError::BadRequest(
                    "Insufficient inventory for grid sell".to_string()
                ));
            }
        }

        // Update level state only after validation
        {
            let level = &mut self.state.grid_levels[level_index];
            level.fill_count += 1;
            level.last_fill_time = Some(context.current_time);
            level.total_filled += quantity;
        }

        // Update strategy state
        match side {
            TradeSide::Buy => {
                self.state.inventory += btc_quantity;
                self.state.stats.buy_fills += 1;
                self.state.stats.total_deployed += quantity; // Track capital deployed
                self.update_average_price(price, btc_quantity, true);
            }
            TradeSide::Sell => {
                self.state.inventory -= btc_quantity;
                self.state.stats.sell_fills += 1;
                self.update_average_price(price, btc_quantity, false);

                // Calculate realized PnL
                if let Some(avg_price) = self.state.average_entry_price {
                    let pnl = (price - avg_price) * btc_quantity;
                    self.state.realized_pnl += pnl;
                }
            }
        }

        self.state.total_trades += 1;
        self.state.stats.total_volume += quantity;

        // Update statistics
        self.state.stats.max_inventory = self.state.stats.max_inventory.max(self.state.inventory);
        self.state.stats.min_inventory = self.state.stats.min_inventory.min(self.state.inventory);

        // Create execution record
        let execution = GridExecution {
            timestamp: context.current_time,
            grid_level_index: level_index,
            price,
            quantity,
            order_type,
            grid_level_before: level_price,
            grid_level_after: level_price, // Same for grid trading
            inventory_before: self.state.inventory - if matches!(side, TradeSide::Buy) { quantity } else { -quantity },
            inventory_after: self.state.inventory,
            realized_pnl: self.state.realized_pnl,
            market_conditions: self.capture_market_conditions(context),
            reason: format!("Grid level {} filled", level_index),
        };

        self.execution_history.push(execution);

        // Keep only last 1000 executions
        if self.execution_history.len() > 1000 {
            self.execution_history.remove(0);
        }

        self.last_signal_reason = format!("Grid level {} filled at {}", level_index, price);

        // Create strategy signal
        // Use AddToPosition/ReducePosition for grid trading to allow multiple buys/sells
        // quantity here represents dollar amount to invest per level
        let signal = match side {
            TradeSide::Buy => StrategySignal::add_to_position(
                context.symbol.clone(),
                QuantityType::DollarAmount(quantity), // quantity is dollar amount, not BTC units
                self.last_signal_reason.clone(),
                None,
            ),
            TradeSide::Sell => {
                // For selling, we need to sell the equivalent BTC quantity
                // Convert dollar amount to BTC quantity based on current price
                let btc_quantity = quantity / price;
                StrategySignal::reduce_position(
                    context.symbol.clone(),
                    QuantityType::Fixed(btc_quantity), // Sell specific BTC quantity
                    self.last_signal_reason.clone(),
                    None,
                )
            },
        };

        info!("Grid level {} executed: {:?} {} at {} (Inventory: {})",
              level_index, side, quantity, price, self.state.inventory);

        Ok(signal)
    }

    /// Update average entry price
    fn update_average_price(&mut self, price: Decimal, quantity: Decimal, is_buy: bool) {
        if is_buy {
            match self.state.average_entry_price {
                Some(avg_price) => {
                    let total_cost = avg_price * self.state.inventory.abs() + price * quantity;
                    let total_quantity = self.state.inventory.abs() + quantity;
                    self.state.average_entry_price = Some(total_cost / total_quantity);
                }
                None => {
                    self.state.average_entry_price = Some(price);
                }
            }
        }
    }

    /// Check if grid needs rebalancing
    fn needs_rebalancing(&self, context: &StrategyContext) -> bool {
        let config = self.config.as_ref().unwrap();

        if !config.enable_rebalancing {
            return false;
        }

        let current_price = context.current_price;

        // Check if price is outside grid bounds
        if current_price >= self.state.grid_upper_bound || current_price <= self.state.grid_lower_bound {
            return true;
        }

        // Check time-based rebalancing
        if let Some(interval_hours) = config.rebalancing_interval {
            if let Some(last_rebalance) = self.state.last_rebalance_time {
                let time_diff = context.current_time.signed_duration_since(last_rebalance);
                if time_diff >= Duration::hours(interval_hours as i64) {
                    return true;
                }
            } else {
                return true; // First rebalancing
            }
        }

        false
    }

    /// Rebalance the grid
    fn rebalance_grid(&mut self, context: &StrategyContext, reason: RebalanceReason) -> Result<(), AppError> {
        let current_price = context.current_price;

        info!("Rebalancing grid due to {:?} at price {}", reason, current_price);

        // DON'T close positions on rebalance - just recalculate grid levels
        // The inventory tracking continues, but we adjust the grid around the new price
        // This allows open positions to remain until they're naturally closed by grid levels

        // Clear existing grid
        self.state.grid_levels.clear();

        // Reinitialize grid with current price as center
        self.initialize_grid(current_price)?;

        self.state.last_rebalance_time = Some(context.current_time);

        Ok(())
    }

    /// Check risk management conditions
    fn check_risk_management(&self, context: &StrategyContext) -> Option<String> {
        let config = self.config.as_ref().unwrap();

        // Check maximum inventory
        if self.state.inventory.abs() > config.risk_settings.max_inventory {
            return Some("Maximum inventory exceeded".to_string());
        }

        // TODO: Fix drawdown calculation - currently using total_investment instead of initial_balance
        // This causes premature strategy termination. Commenting out for now.
        //
        // The correct calculation should be:
        // drawdown_pct = (total_pnl / initial_balance) * 100
        //
        // But we don't have access to initial_balance in the strategy context.
        // The backtesting engine should handle this check instead.

        // Check maximum time in position
        if let Some(max_hours) = config.risk_settings.max_time_in_position {
            if self.state.inventory != Decimal::ZERO {
                // Find the oldest position (earliest fill time)
                let oldest_fill = self.state.grid_levels
                    .iter()
                    .filter(|level| level.fill_count > 0)
                    .filter_map(|level| level.last_fill_time)
                    .min();

                if let Some(oldest_time) = oldest_fill {
                    let position_age = context.current_time.signed_duration_since(oldest_time);
                    if position_age >= Duration::hours(max_hours as i64) {
                        return Some(format!("Maximum time in position exceeded: {} hours", position_age.num_hours()));
                    }
                }
            }
        }

        None
    }

    /// Capture current market conditions
    fn capture_market_conditions(&self, context: &StrategyContext) -> GridMarketConditions {
        let mut conditions = GridMarketConditions {
            price: context.current_price,
            volume: context.market_data.volume_24h,
            ..Default::default()
        };

        // Calculate spread if available
        if let (Some(bid), Some(ask)) = (context.market_data.bid_price, context.market_data.ask_price) {
            conditions.spread = Some(ask - bid);
        }

        // Add volatility calculation using ATR
        if context.historical_data.len() >= 14 {
            if let Some(atr) = indicators::atr(&context.historical_data, 14) {
                conditions.volatility = Some(atr);
            }
        }

        conditions
    }

    /// Calculate unrealized PnL
    fn calculate_unrealized_pnl(&mut self, current_price: Decimal) {
        if let Some(avg_price) = self.state.average_entry_price {
            self.state.unrealized_pnl = (current_price - avg_price) * self.state.inventory;
        } else {
            self.state.unrealized_pnl = Decimal::ZERO;
        }
    }
}

#[async_trait]
impl Strategy for GridTradingStrategy {
    fn metadata(&self) -> StrategyMetadata {
        self.metadata.clone()
    }

    async fn initialize(
        &mut self,
        parameters: &Value,
        _mode: StrategyMode,
        context: &StrategyContext,
    ) -> Result<(), AppError> {
        let config: GridTradingConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid Grid Trading parameters: {}", e)))?;

        // Validate configuration
        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        info!("GRID STRATEGY CONFIG - Total Investment: ${}, Grid Levels: {}, Investment per Grid: ${}, Available Balance: ${}",
              config.total_investment, config.grid_levels, config.calculate_order_size_per_level(), context.available_balance);

        self.config = Some(config);
        self.state = GridTradingState::default();
        self.execution_history.clear();
        self.is_paused = false;
        self.last_signal_reason = "Strategy initialized".to_string();

        // Initialize grid with current market price
        self.initialize_grid(context.current_price)?;

        info!("Grid Trading strategy initialized successfully");
        Ok(())
    }

    async fn analyze(
        &mut self,
        context: &StrategyContext,
    ) -> Result<Option<StrategySignal>, AppError> {
        if self.is_paused || !self.state.is_active {
            return Ok(None);
        }

        let _config = self.config.as_ref()
            .ok_or_else(|| AppError::BadRequest("Strategy not initialized".to_string()))?;

        // Update unrealized PnL
        self.calculate_unrealized_pnl(context.current_price);

        // Check risk management
        if let Some(risk_message) = self.check_risk_management(context) {
            warn!("Risk management triggered: {}", risk_message);
            self.state.is_active = false;
            return Ok(None);
        }

        // Check if grid needs rebalancing
        if self.needs_rebalancing(context) {
            let reason = if context.current_price >= self.state.grid_upper_bound || context.current_price <= self.state.grid_lower_bound {
                RebalanceReason::PriceOutOfBounds
            } else {
                RebalanceReason::TimeInterval
            };

            self.rebalance_grid(context, reason)?;
        }

        // Check for grid fills
        let fills = self.check_grid_fills(context);

        if !fills.is_empty() {
            // Execute the first fill (in practice, you might want to handle multiple fills)
            let (level_index, side) = fills[0];
            let signal = self.execute_grid_fill(context, level_index, side)?;

            // Add grid-specific indicators to signal
            let indicators = vec![
                IndicatorValue {
                    name: "Grid Level".to_string(),
                    value: Decimal::from(level_index),
                    signal: "grid_fill".to_string(),
                },
                IndicatorValue {
                    name: "Inventory".to_string(),
                    value: self.state.inventory,
                    signal: "position".to_string(),
                },
                IndicatorValue {
                    name: "Realized PnL".to_string(),
                    value: self.state.realized_pnl,
                    signal: "pnl".to_string(),
                },
            ];

            let enhanced_signal = signal
                .with_indicators(indicators)
                .with_confidence(Decimal::new(9, 1)); // High confidence for grid fills

            return Ok(Some(enhanced_signal));
        }

        Ok(None)
    }

    fn validate_parameters(&self, parameters: &Value) -> Result<(), AppError> {
        let config: GridTradingConfig = serde_json::from_value(parameters.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid parameters: {}", e)))?;

        config.validate()
            .map_err(|e| AppError::BadRequest(e))?;

        Ok(())
    }

    fn parameter_schema(&self) -> Value {
        GridTradingConfig::json_schema()
    }

    fn get_state(&self) -> Result<Value, AppError> {
        let mut state_with_metadata = serde_json::to_value(&self.state)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize state: {}", e)))?;

        // Add execution history summary
        if let Some(state_obj) = state_with_metadata.as_object_mut() {
            state_obj.insert("execution_count".to_string(), serde_json::Value::Number(
                serde_json::Number::from(self.execution_history.len())
            ));

            if let Some(last_execution) = self.execution_history.last() {
                state_obj.insert("last_execution_reason".to_string(),
                    serde_json::Value::String(last_execution.reason.clone()));
            }

            // Calculate grid efficiency
            let active_levels = self.state.grid_levels.iter().filter(|l| l.is_active).count();
            let filled_levels = self.state.grid_levels.iter().filter(|l| l.fill_count > 0).count();
            let efficiency = if active_levels > 0 {
                Decimal::from(filled_levels) / Decimal::from(active_levels)
            } else {
                Decimal::ZERO
            };

            state_obj.insert("grid_efficiency".to_string(),
                serde_json::Value::String(efficiency.to_string()));
        }

        Ok(state_with_metadata)
    }

    fn restore_state(&mut self, state: &Value) -> Result<(), AppError> {
        self.state = serde_json::from_value(state.clone())
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize state: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl LiveExecutableStrategy for GridTradingStrategy {
    async fn start_live_execution(&mut self, _context: &StrategyContext) -> Result<(), AppError> {
        self.is_running = true;
        self.state.is_active = true;
        info!("Grid Trading strategy started for live execution");
        Ok(())
    }

    async fn stop_live_execution(&mut self) -> Result<(), AppError> {
        self.is_running = false;
        self.state.is_active = false;
        info!("Grid Trading strategy stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn next_execution_time(&self) -> Option<DateTime<Utc>> {
        // Grid trading is event-driven, but we might have rebalancing intervals
        if let Some(config) = &self.config {
            if let Some(interval_hours) = config.rebalancing_interval {
                if let Some(last_rebalance) = self.state.last_rebalance_time {
                    return Some(last_rebalance + Duration::hours(interval_hours as i64));
                }
            }
        }
        None
    }
}

#[async_trait]
impl ControllableStrategy for GridTradingStrategy {
    async fn pause(&mut self) -> Result<(), AppError> {
        self.is_paused = true;
        info!("Grid Trading strategy paused");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), AppError> {
        self.is_paused = false;
        info!("Grid Trading strategy resumed");
        Ok(())
    }

    fn is_paused(&self) -> bool {
        self.is_paused
    }
}

impl Default for GridTradingStrategy {
    fn default() -> Self {
        Self::new()
    }
}