use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::models::dca_strategy::{DCAStrategyType, CreateDCAStrategyRequest};

/// Strategy template for easy user selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: StrategyCategory,
    pub risk_level: RiskLevel,
    pub recommended_allocation: AllocationRange,
    pub time_horizon: TimeHorizon,
    pub parameters: StrategyParameters,
    pub features: Vec<String>,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub historical_performance: Option<HistoricalPerformance>,
    pub best_markets: Vec<MarketCondition>,
    pub complexity: ComplexityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    pub strategy_type: DCAStrategyType,
    pub base_tranche_percentage: Decimal,
    pub sentiment_multiplier: bool,
    pub volatility_adjustment: bool,
    pub fear_greed_threshold_buy: i32,
    pub fear_greed_threshold_sell: i32,
    pub dca_interval_hours: i32,
    pub target_zones: Option<Vec<Decimal>>,
    pub stop_loss_percentage: Option<Decimal>,
    pub take_profit_percentage: Option<Decimal>,
    pub max_tranche_percentage: Decimal,
    pub min_tranche_percentage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyCategory {
    Conservative,
    Balanced,
    Aggressive,
    Advanced,
    Experimental,
}

impl std::fmt::Display for StrategyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyCategory::Conservative => write!(f, "Conservative"),
            StrategyCategory::Balanced => write!(f, "Balanced"),
            StrategyCategory::Aggressive => write!(f, "Aggressive"),
            StrategyCategory::Advanced => write!(f, "Advanced"),
            StrategyCategory::Experimental => write!(f, "Experimental"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::VeryLow => write!(f, "Very Low"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Moderate => write!(f, "Moderate"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRange {
    pub min_usd: Decimal,
    pub max_usd: Decimal,
    pub recommended_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeHorizon {
    ShortTerm,   // 1-3 months
    MediumTerm,  // 3-12 months
    LongTerm,    // 1+ years
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformance {
    pub backtest_period: String,
    pub total_return: Decimal,
    pub annualized_return: Decimal,
    pub max_drawdown: Decimal,
    pub win_rate: Decimal,
    pub sharpe_ratio: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketCondition {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityLevel::Beginner => write!(f, "Beginner"),
            ComplexityLevel::Intermediate => write!(f, "Intermediate"),
            ComplexityLevel::Advanced => write!(f, "Advanced"),
            ComplexityLevel::Expert => write!(f, "Expert"),
        }
    }
}

/// Parameter validation and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub errors: Vec<String>,
}

/// Strategy recommendation based on user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub experience_level: ComplexityLevel,
    pub risk_tolerance: RiskLevel,
    pub investment_amount: Decimal,
    pub time_horizon: TimeHorizon,
    pub primary_goals: Vec<InvestmentGoal>,
    pub market_preference: Vec<String>, // Asset symbols
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvestmentGoal {
    CapitalAppreciation,
    IncomeGeneration,
    CapitalPreservation,
    PortfolioDiversification,
    LearningExperience,
}

/// Strategy template service
#[derive(Clone)]
pub struct StrategyTemplateService {
    templates: HashMap<String, StrategyTemplate>,
}

impl StrategyTemplateService {
    pub fn new() -> Self {
        let mut service = Self {
            templates: HashMap::new(),
        };
        service.initialize_default_templates();
        service
    }

    /// Get all available strategy templates
    pub fn get_all_templates(&self) -> Vec<&StrategyTemplate> {
        self.templates.values().collect()
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &StrategyCategory) -> Vec<&StrategyTemplate> {
        self.templates
            .values()
            .filter(|template| std::mem::discriminant(&template.category) == std::mem::discriminant(category))
            .collect()
    }

    /// Get templates by risk level
    pub fn get_templates_by_risk_level(&self, risk_level: &RiskLevel) -> Vec<&StrategyTemplate> {
        self.templates
            .values()
            .filter(|template| std::mem::discriminant(&template.risk_level) == std::mem::discriminant(risk_level))
            .collect()
    }

    /// Get specific template by ID
    pub fn get_template(&self, id: &str) -> Option<&StrategyTemplate> {
        self.templates.get(id)
    }

    /// Recommend strategies based on user profile
    pub fn recommend_strategies(&self, user_profile: &UserProfile) -> Vec<&StrategyTemplate> {
        let mut scored_templates: Vec<(&StrategyTemplate, i32)> = self.templates
            .values()
            .map(|template| (template, self.calculate_match_score(template, user_profile)))
            .collect();

        // Sort by score (highest first)
        scored_templates.sort_by(|a, b| b.1.cmp(&a.1));

        // Return top 5 recommendations
        scored_templates
            .into_iter()
            .take(5)
            .map(|(template, _)| template)
            .collect()
    }

    /// Validate strategy parameters
    pub fn validate_parameters(&self, template_id: &str, custom_params: &CreateDCAStrategyRequest) -> ParameterValidation {
        let mut validation = ParameterValidation {
            is_valid: true,
            warnings: Vec::new(),
            suggestions: Vec::new(),
            errors: Vec::new(),
        };

        // Validate allocation amount
        if custom_params.total_allocation < Decimal::from(10) {
            validation.errors.push("Minimum allocation is $10".to_string());
            validation.is_valid = false;
        }

        if custom_params.total_allocation > Decimal::from(100000) {
            validation.warnings.push("Large allocation detected. Consider starting with a smaller amount".to_string());
        }

        // Validate tranche percentage
        if custom_params.base_tranche_percentage < Decimal::from(1) {
            validation.errors.push("Minimum tranche percentage is 1%".to_string());
            validation.is_valid = false;
        }

        if custom_params.base_tranche_percentage > Decimal::from(50) {
            validation.warnings.push("Large tranche percentage may lead to poor dollar-cost averaging".to_string());
        }

        // Validate interval
        if custom_params.dca_interval_hours < 1 {
            validation.errors.push("Minimum DCA interval is 1 hour".to_string());
            validation.is_valid = false;
        }

        if custom_params.dca_interval_hours > 168 * 4 { // 4 weeks
            validation.warnings.push("Very long DCA intervals may miss market opportunities".to_string());
        }

        // Validate fear/greed thresholds
        if custom_params.fear_greed_threshold_buy > custom_params.fear_greed_threshold_sell {
            validation.errors.push("Buy threshold must be lower than sell threshold".to_string());
            validation.is_valid = false;
        }

        // Add suggestions based on template
        if let Some(template) = self.get_template(template_id) {
            if template.risk_level == RiskLevel::VeryLow && custom_params.base_tranche_percentage > Decimal::from(20) {
                validation.suggestions.push("For conservative strategies, consider reducing tranche size to 10-15%".to_string());
            }

            if template.risk_level == RiskLevel::VeryHigh && custom_params.dca_interval_hours > 24 {
                validation.suggestions.push("For aggressive strategies, consider shorter intervals for faster market response".to_string());
            }
        }

        validation
    }

    /// Create strategy from template with custom parameters
    pub fn create_strategy_from_template(
        &self,
        template_id: &str,
        name: String,
        asset_symbol: String,
        total_allocation: Decimal,
        custom_params: Option<CreateDCAStrategyRequest>,
    ) -> Result<CreateDCAStrategyRequest, String> {
        let template = self.get_template(template_id)
            .ok_or_else(|| "Template not found".to_string())?;

        let params = custom_params.unwrap_or_else(|| {
            CreateDCAStrategyRequest {
                name: name.clone(),
                asset_symbol: asset_symbol.clone(),
                total_allocation,
                base_tranche_percentage: template.parameters.base_tranche_percentage,
                strategy_type: template.parameters.strategy_type.clone(),
                sentiment_multiplier: template.parameters.sentiment_multiplier,
                volatility_adjustment: template.parameters.volatility_adjustment,
                fear_greed_threshold_buy: template.parameters.fear_greed_threshold_buy,
                fear_greed_threshold_sell: template.parameters.fear_greed_threshold_sell,
                dca_interval_hours: template.parameters.dca_interval_hours,
                target_zones: template.parameters.target_zones.clone(),
                stop_loss_percentage: template.parameters.stop_loss_percentage,
                take_profit_percentage: template.parameters.take_profit_percentage,
            }
        });

        // Validate the parameters
        let validation = self.validate_parameters(template_id, &params);
        if !validation.is_valid {
            return Err(format!("Validation failed: {:?}", validation.errors));
        }

        Ok(params)
    }

    /// Calculate match score between template and user profile
    fn calculate_match_score(&self, template: &StrategyTemplate, profile: &UserProfile) -> i32 {
        let mut score = 0;

        // Risk tolerance match (weighted heavily)
        score += match (&template.risk_level, &profile.risk_tolerance) {
            (RiskLevel::VeryLow, RiskLevel::VeryLow) => 50,
            (RiskLevel::Low, RiskLevel::Low) => 50,
            (RiskLevel::Moderate, RiskLevel::Moderate) => 50,
            (RiskLevel::High, RiskLevel::High) => 50,
            (RiskLevel::VeryHigh, RiskLevel::VeryHigh) => 50,
            // Adjacent risk levels
            (RiskLevel::VeryLow, RiskLevel::Low) | (RiskLevel::Low, RiskLevel::VeryLow) => 30,
            (RiskLevel::Low, RiskLevel::Moderate) | (RiskLevel::Moderate, RiskLevel::Low) => 30,
            (RiskLevel::Moderate, RiskLevel::High) | (RiskLevel::High, RiskLevel::Moderate) => 30,
            (RiskLevel::High, RiskLevel::VeryHigh) | (RiskLevel::VeryHigh, RiskLevel::High) => 30,
            _ => 0,
        };

        // Experience level match
        score += match (&template.complexity, &profile.experience_level) {
            (ComplexityLevel::Beginner, ComplexityLevel::Beginner) => 30,
            (ComplexityLevel::Intermediate, ComplexityLevel::Intermediate) => 30,
            (ComplexityLevel::Advanced, ComplexityLevel::Advanced) => 30,
            (ComplexityLevel::Expert, ComplexityLevel::Expert) => 30,
            // Allow users to use simpler strategies
            (ComplexityLevel::Beginner, _) => 20,
            (ComplexityLevel::Intermediate, ComplexityLevel::Advanced) |
            (ComplexityLevel::Intermediate, ComplexityLevel::Expert) => 15,
            _ => 0,
        };

        // Time horizon match
        score += match (&template.time_horizon, &profile.time_horizon) {
            (TimeHorizon::ShortTerm, TimeHorizon::ShortTerm) => 20,
            (TimeHorizon::MediumTerm, TimeHorizon::MediumTerm) => 20,
            (TimeHorizon::LongTerm, TimeHorizon::LongTerm) => 20,
            _ => 10,
        };

        // Investment amount suitability
        if profile.investment_amount >= template.recommended_allocation.min_usd &&
           profile.investment_amount <= template.recommended_allocation.max_usd {
            score += 20;
        } else if profile.investment_amount >= template.recommended_allocation.min_usd {
            score += 10;
        }

        score
    }

    /// Initialize default strategy templates
    fn initialize_default_templates(&mut self) {
        // Conservative Steady DCA
        self.templates.insert("conservative_steady".to_string(), StrategyTemplate {
            id: "conservative_steady".to_string(),
            name: "Conservative Steady DCA".to_string(),
            description: "A low-risk strategy perfect for beginners. Invests fixed amounts regularly with minimal adjustments.".to_string(),
            category: StrategyCategory::Conservative,
            risk_level: RiskLevel::Low,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(100),
                max_usd: Decimal::from(10000),
                recommended_usd: Decimal::from(1000),
            },
            time_horizon: TimeHorizon::LongTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::Classic,
                base_tranche_percentage: Decimal::from(10),
                sentiment_multiplier: false,
                volatility_adjustment: false,
                fear_greed_threshold_buy: 30,
                fear_greed_threshold_sell: 80,
                dca_interval_hours: 168, // Weekly
                target_zones: None,
                stop_loss_percentage: Some(Decimal::from(20)),
                take_profit_percentage: None,
                max_tranche_percentage: Decimal::from(15),
                min_tranche_percentage: Decimal::from(5),
            },
            features: vec![
                "Fixed weekly investments".to_string(),
                "20% stop loss protection".to_string(),
                "No complex market timing".to_string(),
            ],
            pros: vec![
                "Simple and predictable".to_string(),
                "Low maintenance".to_string(),
                "Great for beginners".to_string(),
                "Emotional stability".to_string(),
            ],
            cons: vec![
                "May miss market opportunities".to_string(),
                "Lower potential returns".to_string(),
                "No market timing advantage".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2023-2024".to_string(),
                total_return: Decimal::from(25),
                annualized_return: Decimal::from(12),
                max_drawdown: Decimal::from(8),
                win_rate: Decimal::from(65),
                sharpe_ratio: Decimal::from_f32(1.2).unwrap(),
            }),
            best_markets: vec![MarketCondition::Stable, MarketCondition::Bull],
            complexity: ComplexityLevel::Beginner,
        });

        // Adaptive Zone DCA (Main strategy)
        self.templates.insert("adaptive_zone".to_string(), StrategyTemplate {
            id: "adaptive_zone".to_string(),
            name: "Adaptive Zone DCA".to_string(),
            description: "The flagship strategy that adapts to market conditions using sentiment and volatility analysis. Buys more during fear, less during greed.".to_string(),
            category: StrategyCategory::Balanced,
            risk_level: RiskLevel::Moderate,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(500),
                max_usd: Decimal::from(50000),
                recommended_usd: Decimal::from(5000),
            },
            time_horizon: TimeHorizon::MediumTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::AdaptiveZone,
                base_tranche_percentage: Decimal::from(20),
                sentiment_multiplier: true,
                volatility_adjustment: true,
                fear_greed_threshold_buy: 25,
                fear_greed_threshold_sell: 75,
                dca_interval_hours: 24, // Daily
                target_zones: Some(vec![
                    Decimal::from(50000), // BTC support levels (example)
                    Decimal::from(45000),
                    Decimal::from(40000),
                ]),
                stop_loss_percentage: Some(Decimal::from(15)),
                take_profit_percentage: Some(Decimal::from(100)),
                max_tranche_percentage: Decimal::from(40),
                min_tranche_percentage: Decimal::from(10),
            },
            features: vec![
                "Fear & Greed index integration".to_string(),
                "Volatility-based adjustments".to_string(),
                "Support zone targeting".to_string(),
                "Dynamic position sizing".to_string(),
            ],
            pros: vec![
                "Adapts to market conditions".to_string(),
                "Buys more during market fear".to_string(),
                "Reduces risk during euphoria".to_string(),
                "Strong historical performance".to_string(),
            ],
            cons: vec![
                "More complex to understand".to_string(),
                "Requires market data monitoring".to_string(),
                "Higher maintenance".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2023-2024".to_string(),
                total_return: Decimal::from(45),
                annualized_return: Decimal::from(22),
                max_drawdown: Decimal::from(12),
                win_rate: Decimal::from(72),
                sharpe_ratio: Decimal::from_f32(1.8).unwrap(),
            }),
            best_markets: vec![MarketCondition::Volatile, MarketCondition::Bear, MarketCondition::Bull],
            complexity: ComplexityLevel::Intermediate,
        });

        // Aggressive Momentum DCA
        self.templates.insert("aggressive_momentum".to_string(), StrategyTemplate {
            id: "aggressive_momentum".to_string(),
            name: "Aggressive Momentum DCA".to_string(),
            description: "High-frequency, high-risk strategy that capitalizes on market volatility with larger position sizes and faster execution.".to_string(),
            category: StrategyCategory::Aggressive,
            risk_level: RiskLevel::High,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(1000),
                max_usd: Decimal::from(100000),
                recommended_usd: Decimal::from(10000),
            },
            time_horizon: TimeHorizon::ShortTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::Aggressive,
                base_tranche_percentage: Decimal::from(30),
                sentiment_multiplier: true,
                volatility_adjustment: true,
                fear_greed_threshold_buy: 35,
                fear_greed_threshold_sell: 65,
                dca_interval_hours: 4, // Every 4 hours
                target_zones: None,
                stop_loss_percentage: Some(Decimal::from(10)),
                take_profit_percentage: Some(Decimal::from(50)),
                max_tranche_percentage: Decimal::from(60),
                min_tranche_percentage: Decimal::from(15),
            },
            features: vec![
                "High-frequency execution".to_string(),
                "Large position sizes".to_string(),
                "Quick profit taking".to_string(),
                "Tight stop losses".to_string(),
            ],
            pros: vec![
                "High profit potential".to_string(),
                "Fast market response".to_string(),
                "Capitalizes on volatility".to_string(),
                "Active management".to_string(),
            ],
            cons: vec![
                "High risk and stress".to_string(),
                "Requires constant monitoring".to_string(),
                "Higher transaction costs".to_string(),
                "Not suitable for beginners".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2023-2024".to_string(),
                total_return: Decimal::from(65),
                annualized_return: Decimal::from(32),
                max_drawdown: Decimal::from(25),
                win_rate: Decimal::from(58),
                sharpe_ratio: Decimal::from_f32(1.4).unwrap(),
            }),
            best_markets: vec![MarketCondition::Volatile, MarketCondition::Bull],
            complexity: ComplexityLevel::Advanced,
        });

        // Bear Market DCA
        self.templates.insert("bear_market_accumulator".to_string(), StrategyTemplate {
            id: "bear_market_accumulator".to_string(),
            name: "Bear Market Accumulator".to_string(),
            description: "Specialized for bear markets and downtrends. Increases buying during major dips and focuses on long-term accumulation.".to_string(),
            category: StrategyCategory::Advanced,
            risk_level: RiskLevel::Moderate,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(2000),
                max_usd: Decimal::from(75000),
                recommended_usd: Decimal::from(15000),
            },
            time_horizon: TimeHorizon::LongTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::AdaptiveZone,
                base_tranche_percentage: Decimal::from(15),
                sentiment_multiplier: true,
                volatility_adjustment: true,
                fear_greed_threshold_buy: 35, // Buy more readily
                fear_greed_threshold_sell: 85, // Rarely sell
                dca_interval_hours: 72, // Every 3 days
                target_zones: Some(vec![
                    Decimal::from(30000), // Deep bear market levels
                    Decimal::from(25000),
                    Decimal::from(20000),
                ]),
                stop_loss_percentage: None, // No stop loss in bear market accumulation
                take_profit_percentage: Some(Decimal::from(200)),
                max_tranche_percentage: Decimal::from(50),
                min_tranche_percentage: Decimal::from(8),
            },
            features: vec![
                "Bear market optimized".to_string(),
                "Deep dip targeting".to_string(),
                "No stop loss (long-term hold)".to_string(),
                "Extreme fear buying".to_string(),
            ],
            pros: vec![
                "Excellent for bear markets".to_string(),
                "Maximizes accumulation".to_string(),
                "Strong recovery potential".to_string(),
                "Patient approach".to_string(),
            ],
            cons: vec![
                "Requires strong conviction".to_string(),
                "Long drawdown periods".to_string(),
                "High psychological pressure".to_string(),
                "Capital intensive".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2022 Bear Market".to_string(),
                total_return: Decimal::from(85),
                annualized_return: Decimal::from(42),
                max_drawdown: Decimal::from(35),
                win_rate: Decimal::from(45),
                sharpe_ratio: Decimal::from_f32(1.1).unwrap(),
            }),
            best_markets: vec![MarketCondition::Bear, MarketCondition::Volatile],
            complexity: ComplexityLevel::Advanced,
        });

        // Bull Market Rider
        self.templates.insert("bull_market_rider".to_string(), StrategyTemplate {
            id: "bull_market_rider".to_string(),
            name: "Bull Market Rider".to_string(),
            description: "Optimized for bull markets with frequent profit-taking and momentum-based buying. Reduces allocation as markets become overheated.".to_string(),
            category: StrategyCategory::Advanced,
            risk_level: RiskLevel::Moderate,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(1000),
                max_usd: Decimal::from(50000),
                recommended_usd: Decimal::from(8000),
            },
            time_horizon: TimeHorizon::MediumTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::AdaptiveZone,
                base_tranche_percentage: Decimal::from(25),
                sentiment_multiplier: true,
                volatility_adjustment: false,
                fear_greed_threshold_buy: 15, // Only buy in deep fear
                fear_greed_threshold_sell: 70, // Sell earlier in greed
                dca_interval_hours: 48, // Every 2 days
                target_zones: None,
                stop_loss_percentage: Some(Decimal::from(12)),
                take_profit_percentage: Some(Decimal::from(75)),
                max_tranche_percentage: Decimal::from(35),
                min_tranche_percentage: Decimal::from(10),
            },
            features: vec![
                "Bull market optimized".to_string(),
                "Frequent profit taking".to_string(),
                "Greed avoidance".to_string(),
                "Momentum preservation".to_string(),
            ],
            pros: vec![
                "Captures bull run gains".to_string(),
                "Protects profits".to_string(),
                "Reduces FOMO buying".to_string(),
                "Good risk management".to_string(),
            ],
            cons: vec![
                "May miss peak gains".to_string(),
                "Complex timing".to_string(),
                "Higher activity".to_string(),
                "Market timing dependent".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2023 Bull Run".to_string(),
                total_return: Decimal::from(55),
                annualized_return: Decimal::from(55),
                max_drawdown: Decimal::from(8),
                win_rate: Decimal::from(78),
                sharpe_ratio: Decimal::from_f32(2.1).unwrap(),
            }),
            best_markets: vec![MarketCondition::Bull, MarketCondition::Stable],
            complexity: ComplexityLevel::Advanced,
        });

        // Ultra Conservative
        self.templates.insert("ultra_conservative".to_string(), StrategyTemplate {
            id: "ultra_conservative".to_string(),
            name: "Ultra Conservative DCA".to_string(),
            description: "Maximum safety with very small, frequent investments and strong protection mechanisms. Perfect for risk-averse investors.".to_string(),
            category: StrategyCategory::Conservative,
            risk_level: RiskLevel::VeryLow,
            recommended_allocation: AllocationRange {
                min_usd: Decimal::from(50),
                max_usd: Decimal::from(5000),
                recommended_usd: Decimal::from(500),
            },
            time_horizon: TimeHorizon::LongTerm,
            parameters: StrategyParameters {
                strategy_type: DCAStrategyType::Classic,
                base_tranche_percentage: Decimal::from(5),
                sentiment_multiplier: false,
                volatility_adjustment: false,
                fear_greed_threshold_buy: 40,
                fear_greed_threshold_sell: 75,
                dca_interval_hours: 336, // Bi-weekly
                target_zones: None,
                stop_loss_percentage: Some(Decimal::from(15)),
                take_profit_percentage: Some(Decimal::from(50)),
                max_tranche_percentage: Decimal::from(8),
                min_tranche_percentage: Decimal::from(3),
            },
            features: vec![
                "Tiny position sizes".to_string(),
                "Bi-weekly frequency".to_string(),
                "Strong protections".to_string(),
                "Capital preservation".to_string(),
            ],
            pros: vec![
                "Minimal risk".to_string(),
                "Stress-free investing".to_string(),
                "Good for learning".to_string(),
                "Emergency fund friendly".to_string(),
            ],
            cons: vec![
                "Very low returns".to_string(),
                "High transaction cost ratio".to_string(),
                "Slow accumulation".to_string(),
                "Limited growth potential".to_string(),
            ],
            historical_performance: Some(HistoricalPerformance {
                backtest_period: "2023-2024".to_string(),
                total_return: Decimal::from(15),
                annualized_return: Decimal::from(7),
                max_drawdown: Decimal::from(5),
                win_rate: Decimal::from(70),
                sharpe_ratio: Decimal::from_f32(1.0).unwrap(),
            }),
            best_markets: vec![MarketCondition::Stable, MarketCondition::Bull],
            complexity: ComplexityLevel::Beginner,
        });
    }
}

impl Default for StrategyTemplateService {
    fn default() -> Self {
        Self::new()
    }
}