use crate::strategies::core::{register_strategy, FactorizableStrategy};
use crate::utils::errors::AppError;
use super::{MACDStrategy, MACDStrategyFactory};

/// Register the MACD strategy in the global registry
pub fn register_macd_strategy() -> Result<(), AppError> {
    let factory = MACDStrategyFactory::new();
    register_strategy(factory)?;
    tracing::info!("MACD strategy registered successfully");
    Ok(())
}

/// Register all MACD strategy variants
pub fn register_all_macd_strategies() -> Result<(), AppError> {
    // Register the main MACD strategy
    register_macd_strategy()?;

    // In the future, you could register specialized variants here
    // register_strategy(MACDAdvancedStrategyFactory::new())?;
    // register_strategy(MACDHistogramStrategyFactory::new())?;

    Ok(())
}

// Implement FactorizableStrategy trait for easier registration
impl FactorizableStrategy for MACDStrategy {
    fn get_metadata() -> crate::strategies::core::StrategyMetadata {
        MACDStrategy::create_metadata()
    }
}

/// Initialize MACD strategies during application startup
pub fn init_macd_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing MACD strategies...");

    match register_all_macd_strategies() {
        Ok(_) => {
            tracing::info!("All MACD strategies initialized successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to initialize MACD strategies: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{get_global_registry, create_strategy};

    #[test]
    fn test_macd_strategy_registration() {
        // Register the strategy
        assert!(register_macd_strategy().is_ok());

        // Check if it's in the registry
        let registry = get_global_registry();
        let registry = registry.read().unwrap();
        assert!(registry.contains("macd_v1"));

        // Create an instance
        drop(registry);
        let strategy = create_strategy("macd_v1");
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        let metadata = strategy.metadata();
        assert_eq!(metadata.id, "macd_v1");
        assert_eq!(metadata.name, "MACD Strategy");
    }

    #[test]
    fn test_macd_strategy_factory_creation() {
        let factory = MACDStrategyFactory::new();
        let metadata = factory.metadata();

        assert_eq!(metadata.id, "macd_v1");
        assert_eq!(metadata.category, crate::strategies::core::StrategyCategory::TechnicalAnalysis);
        assert_eq!(metadata.risk_level, crate::strategies::core::RiskLevel::Moderate);

        let strategy = factory.create();
        assert_eq!(strategy.metadata().id, "macd_v1");
    }
}