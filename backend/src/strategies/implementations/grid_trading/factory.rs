use crate::strategies::core::{Strategy, StrategyFactory, StrategyMetadata};
use super::GridTradingStrategy;

/// Factory for creating Grid Trading strategy instances
pub struct GridTradingStrategyFactory {
    metadata: StrategyMetadata,
}

impl GridTradingStrategyFactory {
    /// Create a new Grid Trading strategy factory
    pub fn new() -> Self {
        Self {
            metadata: GridTradingStrategy::create_metadata(),
        }
    }
}

impl StrategyFactory for GridTradingStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        Box::new(GridTradingStrategy::new())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

impl Default for GridTradingStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}