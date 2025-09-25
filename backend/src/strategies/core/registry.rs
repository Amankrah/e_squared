use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::traits::{Strategy, StrategyFactory, StrategyMetadata, StrategyCategory, RiskLevel, StrategyMode};
use crate::utils::errors::AppError;

/// Global strategy registry
static STRATEGY_REGISTRY: once_cell::sync::Lazy<Arc<RwLock<StrategyRegistry>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(StrategyRegistry::new())));

/// Registry for managing all available trading strategies
pub struct StrategyRegistry {
    /// Map of strategy ID to factory
    strategies: HashMap<String, Box<dyn StrategyFactory>>,
    /// Strategy metadata cache
    metadata_cache: HashMap<String, StrategyMetadata>,
}

/// Strategy search filters
#[derive(Debug, Clone, Deserialize)]
pub struct StrategyFilter {
    pub category: Option<StrategyCategory>,
    pub risk_level: Option<RiskLevel>,
    pub mode: Option<StrategyMode>,
    pub min_balance: Option<rust_decimal::Decimal>,
    pub tags: Option<Vec<String>>,
    pub search_text: Option<String>,
}

/// Strategy list response
#[derive(Debug, Clone, Serialize)]
pub struct StrategyListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: StrategyCategory,
    pub risk_level: RiskLevel,
    pub supported_modes: Vec<StrategyMode>,
    pub tags: Vec<String>,
    pub version: String,
    pub author: String,
}

impl StrategyRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            metadata_cache: HashMap::new(),
        }
    }

    /// Register a strategy factory
    pub fn register<F>(&mut self, factory: F) -> Result<(), AppError>
    where
        F: StrategyFactory + 'static,
    {
        let metadata = factory.metadata().clone();
        let strategy_id = metadata.id.clone();

        if self.strategies.contains_key(&strategy_id) {
            warn!("Strategy {} is already registered, overwriting", strategy_id);
        }

        self.metadata_cache.insert(strategy_id.clone(), metadata);
        self.strategies.insert(strategy_id.clone(), Box::new(factory));

        info!("Registered strategy: {}", strategy_id);
        Ok(())
    }

    /// Unregister a strategy
    pub fn unregister(&mut self, strategy_id: &str) -> Result<(), AppError> {
        if self.strategies.remove(strategy_id).is_some() {
            self.metadata_cache.remove(strategy_id);
            info!("Unregistered strategy: {}", strategy_id);
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Strategy not found: {}", strategy_id)))
        }
    }

    /// Create a strategy instance by ID
    pub fn create_strategy(&self, strategy_id: &str) -> Result<Box<dyn Strategy>, AppError> {
        let factory = self.strategies.get(strategy_id)
            .ok_or_else(|| AppError::NotFound(format!("Strategy not found: {}", strategy_id)))?;

        Ok(factory.create())
    }

    /// Get strategy metadata by ID
    pub fn get_metadata(&self, strategy_id: &str) -> Result<&StrategyMetadata, AppError> {
        self.metadata_cache.get(strategy_id)
            .ok_or_else(|| AppError::NotFound(format!("Strategy metadata not found: {}", strategy_id)))
    }

    /// List all registered strategies
    pub fn list_all(&self) -> Vec<StrategyListItem> {
        self.metadata_cache.values()
            .map(|metadata| StrategyListItem {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                category: metadata.category.clone(),
                risk_level: metadata.risk_level.clone(),
                supported_modes: metadata.supported_modes.clone(),
                tags: metadata.tags.clone(),
                version: metadata.version.clone(),
                author: metadata.author.clone(),
            })
            .collect()
    }

    /// List strategies with filters
    pub fn list_filtered(&self, filter: &StrategyFilter) -> Vec<StrategyListItem> {
        self.metadata_cache.values()
            .filter(|metadata| self.matches_filter(metadata, filter))
            .map(|metadata| StrategyListItem {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                description: metadata.description.clone(),
                category: metadata.category.clone(),
                risk_level: metadata.risk_level.clone(),
                supported_modes: metadata.supported_modes.clone(),
                tags: metadata.tags.clone(),
                version: metadata.version.clone(),
                author: metadata.author.clone(),
            })
            .collect()
    }

    /// Get strategies by category
    pub fn get_by_category(&self, category: &StrategyCategory) -> Vec<StrategyListItem> {
        let filter = StrategyFilter {
            category: Some(category.clone()),
            risk_level: None,
            mode: None,
            min_balance: None,
            tags: None,
            search_text: None,
        };
        self.list_filtered(&filter)
    }

    /// Get strategies by risk level
    pub fn get_by_risk_level(&self, risk_level: &RiskLevel) -> Vec<StrategyListItem> {
        let filter = StrategyFilter {
            category: None,
            risk_level: Some(risk_level.clone()),
            mode: None,
            min_balance: None,
            tags: None,
            search_text: None,
        };
        self.list_filtered(&filter)
    }

    /// Get strategies that support a specific mode
    pub fn get_by_mode(&self, mode: &StrategyMode) -> Vec<StrategyListItem> {
        let filter = StrategyFilter {
            category: None,
            risk_level: None,
            mode: Some(mode.clone()),
            min_balance: None,
            tags: None,
            search_text: None,
        };
        self.list_filtered(&filter)
    }

    /// Check if strategy exists
    pub fn contains(&self, strategy_id: &str) -> bool {
        self.strategies.contains_key(strategy_id)
    }

    /// Get all strategy IDs
    pub fn get_strategy_ids(&self) -> Vec<String> {
        self.strategies.keys().cloned().collect()
    }

    /// Get statistics about registered strategies
    pub fn get_stats(&self) -> RegistryStats {
        let total = self.strategies.len();
        let mut by_category = HashMap::new();
        let mut by_risk_level = HashMap::new();
        let mut by_mode = HashMap::new();

        for metadata in self.metadata_cache.values() {
            // Count by category
            *by_category.entry(metadata.category.clone()).or_insert(0) += 1;

            // Count by risk level
            *by_risk_level.entry(metadata.risk_level.clone()).or_insert(0) += 1;

            // Count by supported modes
            for mode in &metadata.supported_modes {
                *by_mode.entry(mode.clone()).or_insert(0) += 1;
            }
        }

        RegistryStats {
            total_strategies: total,
            by_category,
            by_risk_level,
            by_mode,
        }
    }

    /// Filter matching logic
    fn matches_filter(&self, metadata: &StrategyMetadata, filter: &StrategyFilter) -> bool {
        // Category filter
        if let Some(ref category) = filter.category {
            if metadata.category != *category {
                return false;
            }
        }

        // Risk level filter
        if let Some(ref risk_level) = filter.risk_level {
            if metadata.risk_level != *risk_level {
                return false;
            }
        }

        // Mode filter
        if let Some(ref mode) = filter.mode {
            if !metadata.supported_modes.contains(mode) {
                return false;
            }
        }

        // Min balance filter
        if let Some(min_balance) = filter.min_balance {
            if let Some(strategy_min_balance) = metadata.min_balance {
                if strategy_min_balance > min_balance {
                    return false;
                }
            }
        }

        // Tags filter
        if let Some(ref tags) = filter.tags {
            if !tags.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }

        // Search text filter
        if let Some(ref search_text) = filter.search_text {
            let search_lower = search_text.to_lowercase();
            let matches = metadata.name.to_lowercase().contains(&search_lower) ||
                metadata.description.to_lowercase().contains(&search_lower) ||
                metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower));

            if !matches {
                return false;
            }
        }

        true
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    pub total_strategies: usize,
    pub by_category: HashMap<StrategyCategory, usize>,
    pub by_risk_level: HashMap<RiskLevel, usize>,
    pub by_mode: HashMap<StrategyMode, usize>,
}

/// Global registry access functions
pub fn get_global_registry() -> Arc<RwLock<StrategyRegistry>> {
    STRATEGY_REGISTRY.clone()
}

/// Register a strategy in the global registry
pub fn register_strategy<F>(factory: F) -> Result<(), AppError>
where
    F: StrategyFactory + 'static,
{
    let registry = get_global_registry();
    let mut registry = registry.write().map_err(|e| {
        AppError::InternalServerError
    })?;

    registry.register(factory)
}

/// Create strategy from global registry
pub fn create_strategy(strategy_id: &str) -> Result<Box<dyn Strategy>, AppError> {
    let registry = get_global_registry();
    let registry = registry.read().map_err(|e| {
        AppError::InternalServerError
    })?;

    registry.create_strategy(strategy_id)
}

/// Get strategy metadata from global registry
pub fn get_strategy_metadata(strategy_id: &str) -> Result<StrategyMetadata, AppError> {
    let registry = get_global_registry();
    let registry = registry.read().map_err(|e| {
        AppError::InternalServerError
    })?;

    registry.get_metadata(strategy_id).map(|m| m.clone())
}

/// List all strategies from global registry
pub fn list_all_strategies() -> Result<Vec<StrategyListItem>, AppError> {
    let registry = get_global_registry();
    let registry = registry.read().map_err(|e| {
        AppError::InternalServerError
    })?;

    Ok(registry.list_all())
}

/// List strategies with filter from global registry
pub fn list_strategies_filtered(filter: &StrategyFilter) -> Result<Vec<StrategyListItem>, AppError> {
    let registry = get_global_registry();
    let registry = registry.read().map_err(|e| {
        AppError::InternalServerError
    })?;

    Ok(registry.list_filtered(filter))
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}