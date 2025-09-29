use crate::strategies::core::{register_strategy, FactorizableStrategy};
use crate::utils::errors::AppError;
use super::{RSIStrategy, RSIStrategyFactory};

/// Register the RSI strategy in the global registry
pub fn register_rsi_strategy() -> Result<(), AppError> {
    let factory = RSIStrategyFactory::new();
    register_strategy(factory)?;
    tracing::info!("RSI strategy registered successfully");
    Ok(())
}

/// Register all RSI strategy variants
pub fn register_all_rsi_strategies() -> Result<(), AppError> {
    // Register the main RSI strategy
    register_rsi_strategy()?;

    // In the future, you could register specialized variants here
    // register_strategy(RSIAdvancedStrategyFactory::new())?;
    // register_strategy(RSIDivergenceStrategyFactory::new())?;

    Ok(())
}

// Implement FactorizableStrategy trait for easier registration
impl FactorizableStrategy for RSIStrategy {
    fn get_metadata() -> crate::strategies::core::StrategyMetadata {
        RSIStrategy::create_metadata()
    }
}

/// Initialize RSI strategies during application startup
pub fn init_rsi_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing RSI strategies...");

    match register_all_rsi_strategies() {
        Ok(_) => {
            tracing::info!("All RSI strategies initialized successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to initialize RSI strategies: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{get_global_registry, create_strategy};

    #[test]
    fn test_rsi_strategy_registration() {
        // Register the strategy
        assert!(register_rsi_strategy().is_ok());

        // Check if it's in the registry
        let registry = get_global_registry();
        let registry = registry.read().unwrap();
        assert!(registry.contains("rsi_v1"));

        // Create an instance
        drop(registry);
        let strategy = create_strategy("rsi_v1");
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        let metadata = strategy.metadata();
        assert_eq!(metadata.id, "rsi_v1");
        assert_eq!(metadata.name, "RSI Strategy");
    }

    #[test]
    fn test_rsi_strategy_factory_creation() {
        let factory = RSIStrategyFactory::new();
        let metadata = factory.metadata();

        assert_eq!(metadata.id, "rsi_v1");
        assert_eq!(metadata.category, crate::strategies::core::StrategyCategory::TechnicalAnalysis);
        assert_eq!(metadata.risk_level, crate::strategies::core::RiskLevel::Moderate);

        let strategy = factory.create();
        assert_eq!(strategy.metadata().id, "rsi_v1");
    }
}