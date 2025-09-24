#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{StrategyContext, StrategyMode, MarketData, StrategyContextBuilder};
    use crate::exchange_connectors::Kline;
    use chrono::{Utc, Duration};
    use rust_decimal::{Decimal, prelude::FromPrimitive};
    use uuid::Uuid;

    /// Helper function to create test kline data
    fn create_test_klines(count: usize, base_price: Decimal, volatility: Decimal) -> Vec<Kline> {
        let mut klines = Vec::new();
        let mut current_price = base_price;
        let base_time = Utc::now() - Duration::hours(count as i64);

        for i in 0..count {
            let price_change = (volatility * Decimal::from(i % 10) / Decimal::from(10)) - (volatility / Decimal::from(2));
            current_price += price_change;

            let kline = Kline {
                open_time: base_time + Duration::hours(i as i64),
                close_time: base_time + Duration::hours(i as i64) + Duration::minutes(59),
                open: current_price,
                high: current_price + volatility,
                low: current_price - volatility,
                close: current_price,
                volume: Decimal::from(1000),
                number_of_trades: 100,
                taker_buy_base_asset_volume: Decimal::from(500),
                taker_buy_quote_asset_volume: Decimal::from(500) * current_price,
            };
            klines.push(kline);
        }

        klines
    }

    /// Helper function to create test context
    fn create_test_context(
        historical_data: Vec<Kline>,
        current_price: Decimal,
        balance: Decimal,
    ) -> StrategyContext {
        StrategyContextBuilder::new()
            .strategy_id(Uuid::new_v4())
            .user_id(Uuid::new_v4())
            .symbol("BTC/USDT".to_string())
            .interval("1h".to_string())
            .mode(StrategyMode::Paper)
            .historical_data(historical_data)
            .current_price(current_price)
            .available_balance(balance)
            .build()
            .expect("Failed to build test context")
    }

    #[tokio::test]
    async fn test_simple_dca_strategy() {
        let mut strategy = DCAStrategy::new();
        let config = DCAConfig::simple(Decimal::from(100), DCAFrequency::Daily(1));
        let config_json = serde_json::to_value(&config).unwrap();

        let historical_data = create_test_klines(50, Decimal::from(50000), Decimal::from(1000));
        let context = create_test_context(
            historical_data,
            Decimal::from(50000),
            Decimal::from(10000),
        );

        // Initialize strategy
        let result = strategy.initialize(&config_json, StrategyMode::Paper, &context).await;
        assert!(result.is_ok(), "Strategy initialization should succeed");

        // Test signal generation (should not generate on first call due to no last execution)
        let signal = strategy.analyze(&context).await;
        assert!(signal.is_ok());
        let signal = signal.unwrap();
        assert!(signal.is_some(), "Should generate a buy signal");

        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, StrategySignalType::Enter);
        assert_eq!(signal.symbol, "BTC/USDT");

        if let QuantityType::DollarAmount(amount) = signal.action.quantity {
            assert_eq!(amount, Decimal::from(100));
        } else {
            panic!("Expected DollarAmount quantity type");
        }
    }

    #[tokio::test]
    async fn test_rsi_based_dca_strategy() {
        let mut strategy = DCAStrategy::new();
        let rsi_config = RSIConfig {
            period: 14,
            oversold_threshold: Decimal::from(30),
            overbought_threshold: Decimal::from(70),
            oversold_multiplier: Decimal::new(2, 0), // 2.0
            overbought_multiplier: Decimal::new(5, 1), // 0.5
            normal_multiplier: Decimal::from(1),
        };

        let config = DCAConfig::rsi_based(
            Decimal::from(100),
            DCAFrequency::Hourly(12),
            rsi_config,
        );
        let config_json = serde_json::to_value(&config).unwrap();

        // Create data that should result in oversold RSI
        let mut historical_data = create_test_klines(30, Decimal::from(50000), Decimal::from(500));

        // Make prices decline to create oversold condition
        for (i, kline) in historical_data.iter_mut().enumerate() {
            let decline_factor = Decimal::from(i) / Decimal::from(30);
            kline.close -= decline_factor * Decimal::from(5000);
            kline.open = kline.close;
            kline.high = kline.close + Decimal::from(100);
            kline.low = kline.close - Decimal::from(100);
        }

        let context = create_test_context(
            historical_data,
            Decimal::from(45000), // Lower current price
            Decimal::from(10000),
        );

        // Initialize strategy
        let result = strategy.initialize(&config_json, StrategyMode::Paper, &context).await;
        assert!(result.is_ok());

        // Should generate a signal with higher multiplier due to oversold RSI
        let signal = strategy.analyze(&context).await;
        assert!(signal.is_ok());

        let signal = signal.unwrap();
        assert!(signal.is_some(), "Should generate oversold RSI signal");

        let signal = signal.unwrap();
        if let QuantityType::DollarAmount(amount) = signal.action.quantity {
            // Should be higher than base amount due to oversold multiplier
            assert!(amount > Decimal::from(100), "Amount should be multiplied for oversold RSI: {}", amount);
        } else {
            panic!("Expected DollarAmount quantity type");
        }
    }

    #[tokio::test]
    async fn test_dip_buying_strategy() {
        let mut strategy = DCAStrategy::new();

        let dip_levels = vec![
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(5),
                amount_multiplier: Decimal::new(15, 1), // 1.5
                max_triggers: Some(3),
            },
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(10),
                amount_multiplier: Decimal::new(25, 1), // 2.5
                max_triggers: Some(2),
            },
        ];

        let config = DCAConfig::dip_buying(
            Decimal::from(100),
            DCAFrequency::Hourly(4),
            dip_levels,
            Some(Decimal::from(50000)), // Reference price
        );
        let config_json = serde_json::to_value(&config).unwrap();

        let historical_data = create_test_klines(25, Decimal::from(47500), Decimal::from(500)); // 5% drop
        let context = create_test_context(
            historical_data,
            Decimal::from(47500), // 5% below reference
            Decimal::from(10000),
        );

        // Initialize strategy
        let result = strategy.initialize(&config_json, StrategyMode::Paper, &context).await;
        assert!(result.is_ok());

        // Should generate a signal with 1.5x multiplier for 5% dip
        let signal = strategy.analyze(&context).await;
        assert!(signal.is_ok());

        let signal = signal.unwrap();
        assert!(signal.is_some(), "Should generate dip buying signal");

        let signal = signal.unwrap();
        if let QuantityType::DollarAmount(amount) = signal.action.quantity {
            let expected = Decimal::from(100) * Decimal::new(15, 1); // 1.5
            assert_eq!(amount, expected, "Amount should be 1.5x base for 5% dip");
        }
    }

    #[tokio::test]
    async fn test_strategy_filters() {
        let mut strategy = DCAStrategy::new();
        let mut config = DCAConfig::simple(Decimal::from(100), DCAFrequency::Hourly(1));

        // Set filters to only allow trading on Sunday (0)
        config.filters.allowed_weekdays = Some(vec![0]);

        let config_json = serde_json::to_value(&config).unwrap();

        let historical_data = create_test_klines(25, Decimal::from(50000), Decimal::from(500));
        let context = create_test_context(
            historical_data,
            Decimal::from(50000),
            Decimal::from(10000),
        );

        // Initialize strategy
        let result = strategy.initialize(&config_json, StrategyMode::Paper, &context).await;
        assert!(result.is_ok());

        // Should not generate signal because current day is not Sunday
        let signal = strategy.analyze(&context).await;
        assert!(signal.is_ok());

        let signal = signal.unwrap();
        assert!(signal.is_none(), "Should not generate signal due to weekday filter");
    }

    #[tokio::test]
    async fn test_strategy_pause_resume() {
        let mut strategy = DCAStrategy::new();
        let config = DCAConfig::simple(Decimal::from(100), DCAFrequency::Daily(1));
        let config_json = serde_json::to_value(&config).unwrap();

        let historical_data = create_test_klines(25, Decimal::from(50000), Decimal::from(500));
        let context = create_test_context(
            historical_data,
            Decimal::from(50000),
            Decimal::from(10000),
        );

        // Initialize and verify not paused
        strategy.initialize(&config_json, StrategyMode::Paper, &context).await.unwrap();
        assert!(!strategy.is_paused());

        // Pause strategy
        strategy.pause().await.unwrap();
        assert!(strategy.is_paused());

        // Should not generate signal when paused
        let signal = strategy.analyze(&context).await.unwrap();
        assert!(signal.is_none(), "Paused strategy should not generate signals");

        // Resume strategy
        strategy.resume().await.unwrap();
        assert!(!strategy.is_paused());

        // Should generate signal after resume
        let signal = strategy.analyze(&context).await.unwrap();
        assert!(signal.is_some(), "Resumed strategy should generate signals");
    }

    #[tokio::test]
    async fn test_config_presets() {
        let base_amount = Decimal::from(100);

        // Test all presets compile and validate
        for (name, _, preset_fn) in DCAPresets::get_all_presets() {
            let config = preset_fn(base_amount);
            assert!(config.validate().is_ok(), "Preset {} should be valid", name);

            let mut strategy = DCAStrategy::new();
            let config_json = serde_json::to_value(&config).unwrap();

            let historical_data = create_test_klines(50, Decimal::from(50000), Decimal::from(1000));
            let context = create_test_context(
                historical_data,
                Decimal::from(50000),
                Decimal::from(10000),
            );

            let result = strategy.initialize(&config_json, StrategyMode::Paper, &context).await;
            assert!(result.is_ok(), "Preset {} should initialize successfully", name);
        }
    }

    #[test]
    fn test_strategy_metadata() {
        let strategy = DCAStrategy::new();
        let metadata = strategy.metadata();

        assert_eq!(metadata.id, "dca_v2");
        assert_eq!(metadata.name, "Dollar Cost Averaging v2");
        assert_eq!(metadata.category, StrategyCategory::DCA);
        assert_eq!(metadata.risk_level, RiskLevel::Conservative);
        assert!(metadata.supported_modes.contains(&StrategyMode::Live));
        assert!(metadata.supported_modes.contains(&StrategyMode::Paper));
        assert!(metadata.min_balance.is_some());
        assert!(metadata.min_balance.unwrap() > Decimal::ZERO);
    }
}