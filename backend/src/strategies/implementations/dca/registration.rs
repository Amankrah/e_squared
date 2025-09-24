use crate::strategies::core::{register_strategy, FactorizableStrategy};
use crate::utils::errors::AppError;
use super::{DCAStrategy, DCAStrategyFactory};

/// Register the DCA strategy in the global registry
pub fn register_dca_strategy() -> Result<(), AppError> {
    let factory = DCAStrategyFactory::new();
    register_strategy(factory)?;
    tracing::info!("DCA strategy registered successfully");
    Ok(())
}

/// Register all DCA strategy variants
pub fn register_all_dca_strategies() -> Result<(), AppError> {
    // Register the main DCA strategy
    register_dca_strategy()?;

    // In the future, you could register specialized variants here
    // register_strategy(DCASimpleStrategyFactory::new())?;
    // register_strategy(DCARSIStrategyFactory::new())?;
    // register_strategy(DCAVolatilityStrategyFactory::new())?;

    Ok(())
}

// Implement FactorizableStrategy trait for easier registration
impl FactorizableStrategy for DCAStrategy {
    fn get_metadata() -> crate::strategies::core::StrategyMetadata {
        DCAStrategy::create_metadata()
    }
}

/// Initialize DCA strategies during application startup
pub fn init_dca_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing DCA strategies...");

    match register_all_dca_strategies() {
        Ok(_) => {
            tracing::info!("All DCA strategies initialized successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to initialize DCA strategies: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{get_global_registry, create_strategy};

    #[test]
    fn test_dca_strategy_registration() {
        // Register the strategy
        assert!(register_dca_strategy().is_ok());

        // Check if it's in the registry
        let registry = get_global_registry();
        let registry = registry.read().unwrap();
        assert!(registry.contains("dca_v2"));

        // Create an instance
        drop(registry);
        let strategy = create_strategy("dca_v2");
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        let metadata = strategy.metadata();
        assert_eq!(metadata.id, "dca_v2");
        assert_eq!(metadata.name, "Dollar Cost Averaging v2");
    }

    #[test]
    fn test_dca_strategy_factory_creation() {
        let factory = DCAStrategyFactory::new();
        let metadata = factory.metadata();

        assert_eq!(metadata.id, "dca_v2");
        assert_eq!(metadata.category, crate::strategies::core::StrategyCategory::DCA);
        assert_eq!(metadata.risk_level, crate::strategies::core::RiskLevel::Conservative);

        let strategy = factory.create();
        assert_eq!(strategy.metadata().id, "dca_v2");
    }
}