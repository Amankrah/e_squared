use crate::strategies::core::{Strategy, StrategyFactory, StrategyMetadata};
use super::DCAStrategy;

/// Factory for creating DCA strategy instances
pub struct DCAStrategyFactory {
    metadata: StrategyMetadata,
}

impl DCAStrategyFactory {
    pub fn new() -> Self {
        Self {
            metadata: DCAStrategy::create_metadata(),
        }
    }
}

impl StrategyFactory for DCAStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        Box::new(DCAStrategy::new())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

impl Default for DCAStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}