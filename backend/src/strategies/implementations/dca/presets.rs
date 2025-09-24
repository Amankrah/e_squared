use rust_decimal::{Decimal, prelude::FromPrimitive};
use super::{DCAConfig, DCAFrequency, DCAType, RSIConfig, VolatilityConfig, DipBuyingLevel, DynamicFactors, SentimentConfig, DCAFilters};

/// Preset configurations for common DCA strategies
pub struct DCAPresets;

impl DCAPresets {
    /// Conservative DCA - Simple regular purchases
    pub fn conservative(base_amount: Decimal) -> DCAConfig {
        DCAConfig::simple(base_amount, DCAFrequency::Daily(1))
    }

    /// Aggressive DCA - RSI-based with higher multipliers
    pub fn aggressive_rsi(base_amount: Decimal) -> DCAConfig {
        let rsi_config = RSIConfig {
            period: 14,
            oversold_threshold: Decimal::from(25),
            overbought_threshold: Decimal::from(75),
            oversold_multiplier: Decimal::new(3, 0), // 3.0
            overbought_multiplier: Decimal::new(2, 1), // 0.2
            normal_multiplier: Decimal::from(1),
        };

        DCAConfig::rsi_based(base_amount, DCAFrequency::Hourly(12), rsi_config)
    }

    /// Volatility-based DCA - Buy more during high volatility
    pub fn volatility_hunter(base_amount: Decimal) -> DCAConfig {
        let vol_config = VolatilityConfig {
            period: 20,
            low_threshold: Decimal::from(5),
            high_threshold: Decimal::from(25),
            low_volatility_multiplier: Decimal::new(7, 1), // 0.7
            high_volatility_multiplier: Decimal::new(2, 0), // 2.0
            normal_multiplier: Decimal::from(1),
        };

        DCAConfig::volatility_based(base_amount, DCAFrequency::Hourly(6), vol_config)
    }

    /// Dip buyer - Aggressive purchases on price drops
    pub fn dip_buyer(base_amount: Decimal) -> DCAConfig {
        let dip_levels = vec![
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(5),
                amount_multiplier: Decimal::new(15, 1), // 1.5
                max_triggers: Some(5),
            },
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(10),
                amount_multiplier: Decimal::new(25, 1), // 2.5
                max_triggers: Some(3),
            },
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(20),
                amount_multiplier: Decimal::from(5),
                max_triggers: Some(2),
            },
            DipBuyingLevel {
                price_drop_percentage: Decimal::from(30),
                amount_multiplier: Decimal::from(10),
                max_triggers: Some(1),
            },
        ];

        DCAConfig::dip_buying(
            base_amount,
            DCAFrequency::Hourly(4),
            dip_levels,
            None,
        )
    }

    /// Balanced dynamic strategy combining multiple factors
    pub fn balanced_dynamic(base_amount: Decimal) -> DCAConfig {
        let rsi_config = RSIConfig::default();
        let vol_config = VolatilityConfig::default();

        let dynamic_factors = DynamicFactors {
            rsi_weight: Decimal::new(4, 1), // 0.4
            volatility_weight: Decimal::new(3, 1), // 0.3
            sentiment_weight: Decimal::new(2, 1), // 0.2
            trend_weight: Decimal::new(1, 1), // 0.1
            max_multiplier: Decimal::new(3, 0), // 3.0
            min_multiplier: Decimal::new(3, 1), // 0.3
        };

        DCAConfig::dynamic(
            base_amount,
            DCAFrequency::Hourly(8),
            rsi_config,
            vol_config,
            dynamic_factors,
        )
    }

    /// Weekend warrior - Only buy on weekends
    pub fn weekend_warrior(base_amount: Decimal) -> DCAConfig {
        let mut config = Self::conservative(base_amount);
        config.filters = DCAFilters {
            allowed_weekdays: Some(vec![0, 6]), // Sunday and Saturday
            ..Default::default()
        };
        config
    }

    /// Business hours only - Trade during market hours
    pub fn business_hours(base_amount: Decimal) -> DCAConfig {
        let mut config = Self::conservative(base_amount);
        config.filters = DCAFilters {
            allowed_hours: Some(vec![9, 10, 11, 12, 13, 14, 15, 16]), // 9 AM to 4 PM UTC
            allowed_weekdays: Some(vec![1, 2, 3, 4, 5]), // Monday to Friday
            ..Default::default()
        };
        config
    }

    /// Bear market strategy - Increase purchases during downtrends
    pub fn bear_market_hunter(base_amount: Decimal) -> DCAConfig {
        let mut config = Self::dip_buyer(base_amount);
        config.bear_market_threshold = Some(Decimal::from(10)); // 10% drop threshold
        config.pause_on_bear_market = false; // Don't pause, we want to buy!
        config
    }

    /// Risk-managed DCA with strict limits
    pub fn risk_managed(base_amount: Decimal, max_position: Decimal) -> DCAConfig {
        let mut config = Self::balanced_dynamic(base_amount);
        config.max_position_size = Some(max_position);
        config.max_single_amount = Some(base_amount * Decimal::from(2));
        config.min_single_amount = Some(base_amount / Decimal::from(2));
        config.pause_on_high_volatility = true;
        config.volatility_pause_threshold = Some(Decimal::from(50));
        config
    }

    /// High frequency micro DCA
    pub fn micro_dca(base_amount: Decimal) -> DCAConfig {
        let mut config = Self::conservative(base_amount / Decimal::from(24)); // Smaller amounts
        config.frequency = DCAFrequency::Hourly(1); // Every hour
        config.filters.max_spread_percentage = Some(Decimal::new(1, 3)); // 0.001 = 0.1% max spread
        config
    }

    /// Get all available presets with descriptions
    pub fn get_all_presets() -> Vec<(&'static str, &'static str, fn(Decimal) -> DCAConfig)> {
        vec![
            ("conservative", "Simple daily DCA purchases", Self::conservative),
            ("aggressive_rsi", "RSI-based with aggressive multipliers", Self::aggressive_rsi),
            ("volatility_hunter", "Buy more during high volatility", Self::volatility_hunter),
            ("dip_buyer", "Aggressive purchases on price drops", Self::dip_buyer),
            ("balanced_dynamic", "Multi-factor balanced approach", Self::balanced_dynamic),
            ("weekend_warrior", "Only buy on weekends", Self::weekend_warrior),
            ("business_hours", "Trade during market hours only", Self::business_hours),
            ("bear_market_hunter", "Increase purchases during downtrends", Self::bear_market_hunter),
            ("micro_dca", "High frequency micro purchases", Self::micro_dca),
        ]
    }

    /// Get risk-managed preset (takes additional parameter)
    pub fn get_risk_managed_preset() -> (&'static str, &'static str, fn(Decimal, Decimal) -> DCAConfig) {
        ("risk_managed", "Risk-managed DCA with strict limits", Self::risk_managed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_configurations() {
        let base_amount = Decimal::from(100);

        // Test all basic presets
        for (name, _, preset_fn) in DCAPresets::get_all_presets() {
            let config = preset_fn(base_amount);

            // micro_dca preset modifies the base amount, so we handle it separately
            if name == "micro_dca" {
                assert_eq!(config.base_amount, base_amount / Decimal::from(24));
            } else {
                assert_eq!(config.base_amount, base_amount);
            }

            assert!(config.validate().is_ok(), "Preset {} should be valid", name);
        }

        // Test risk-managed preset
        let (_, _, risk_managed_fn) = DCAPresets::get_risk_managed_preset();
        let config = risk_managed_fn(base_amount, Decimal::from(10000));
        assert_eq!(config.base_amount, base_amount);
        assert!(config.validate().is_ok(), "Risk managed preset should be valid");
    }

    #[test]
    fn test_dip_buyer_levels() {
        let config = DCAPresets::dip_buyer(Decimal::from(100));
        let levels = config.dip_levels.unwrap();

        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0].price_drop_percentage, Decimal::from(5));
        assert_eq!(levels[3].price_drop_percentage, Decimal::from(30));
    }

    #[test]
    fn test_filter_configurations() {
        let weekend_config = DCAPresets::weekend_warrior(Decimal::from(100));
        assert!(weekend_config.filters.allowed_weekdays.is_some());

        let business_config = DCAPresets::business_hours(Decimal::from(100));
        assert!(business_config.filters.allowed_hours.is_some());
        assert!(business_config.filters.allowed_weekdays.is_some());
    }
}