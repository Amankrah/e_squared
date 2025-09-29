use crate::strategies::core::{Strategy, StrategyFactory, StrategyMetadata};
use super::SMACrossoverStrategy;

/// Factory for creating SMA Crossover strategy instances
pub struct SMACrossoverStrategyFactory {
    metadata: StrategyMetadata,
}

impl SMACrossoverStrategyFactory {
    /// Create a new SMA Crossover strategy factory
    pub fn new() -> Self {
        Self {
            metadata: SMACrossoverStrategy::create_metadata(),
        }
    }
}

impl StrategyFactory for SMACrossoverStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        Box::new(SMACrossoverStrategy::new())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

impl Default for SMACrossoverStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}