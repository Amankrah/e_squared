import {
  StrategyType,
  StrategyInfo,
  Strategy,
  StrategyConfig,
  DCAStrategy,
  GridTradingStrategy,
  SMACrossoverStrategy
} from './api'

// Strategy metadata for UI
export const STRATEGY_INFO: Record<StrategyType, StrategyInfo> = {
  dca: {
    type: 'dca',
    name: 'Dollar Cost Averaging',
    description: 'Systematically invest fixed amounts at regular intervals to reduce the impact of volatility',
    icon: '📊',
    color: 'from-blue-500/20 to-cyan-500/20',
    riskLevel: 'Low',
    timeHorizon: 'Long-term (6+ months)',
    features: [
      'Automated recurring purchases',
      'Market sentiment analysis',
      'Volatility-based adjustments',
      'Fear & Greed index integration',
      'Target zone optimization'
    ],
    minInvestment: 100
  },
  grid_trading: {
    type: 'grid_trading',
    name: 'Grid Trading',
    description: 'Execute buy and sell orders at predetermined price levels in a grid pattern',
    icon: '🎯',
    color: 'from-green-500/20 to-emerald-500/20',
    riskLevel: 'Medium',
    timeHorizon: 'Medium-term (1-6 months)',
    features: [
      'Automated grid execution',
      'Profit from sideways markets',
      'Configurable price ranges',
      'Rebalancing mechanisms',
      'Risk management controls'
    ],
    minInvestment: 500
  },
  sma_crossover: {
    type: 'sma_crossover',
    name: 'SMA Crossover',
    description: 'Buy when short-term moving average crosses above long-term, sell on reverse',
    icon: '📈',
    color: 'from-purple-500/20 to-pink-500/20',
    riskLevel: 'Medium',
    timeHorizon: 'Medium-term (2-4 months)',
    features: [
      'Moving average crossover signals',
      'Trend following strategy',
      'Configurable periods',
      'Signal confirmation filters',
      'Momentum validation'
    ],
    minInvestment: 250
  }
}

// Strategy status helpers
export const STRATEGY_STATUS_COLORS = {
  active: 'bg-green-500/20 text-green-400 border-green-500/30',
  paused: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
  stopped: 'bg-red-500/20 text-red-400 border-red-500/30',
  draft: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
  error: 'bg-red-500/20 text-red-400 border-red-500/30'
}

// Risk level colors
export const RISK_LEVEL_COLORS = {
  Low: 'text-green-400',
  Medium: 'text-yellow-400',
  High: 'text-red-400'
}

// Utility functions
export function getStrategyInfo(type: StrategyType): StrategyInfo {
  const info = STRATEGY_INFO[type]
  if (!info) {
    console.warn(`Strategy info not found for type: ${type}`)
    // Return a fallback info object
    return {
      type,
      name: type.toUpperCase(),
      description: 'Strategy information not available',
      icon: '📊',
      color: 'from-gray-500/20 to-gray-600/20',
      riskLevel: 'Medium',
      timeHorizon: 'Medium-term',
      features: [],
      minInvestment: 100
    }
  }
  return info
}

export function getStrategyIcon(type: StrategyType): string {
  return STRATEGY_INFO[type].icon
}

export function getStrategyColor(type: StrategyType): string {
  return STRATEGY_INFO[type].color
}

export function getStrategyDisplayName(type: StrategyType): string {
  return STRATEGY_INFO[type].name
}

export function formatStrategyStatus(status: string): string {
  return status.charAt(0).toUpperCase() + status.slice(1).toLowerCase()
}

export function getStrategyStatusColor(status: string): string {
  const normalizedStatus = status.toLowerCase()
  return STRATEGY_STATUS_COLORS[normalizedStatus as keyof typeof STRATEGY_STATUS_COLORS] || STRATEGY_STATUS_COLORS.draft
}

// Strategy type guards
export function isDCAStrategy(strategy: Strategy): strategy is DCAStrategy {
  return 'config' in strategy && strategy.config && 'base_amount' in strategy.config && 'frequency' in strategy.config
}

export function isGridTradingStrategy(strategy: Strategy): strategy is GridTradingStrategy {
  return 'config' in strategy && typeof strategy.config === 'object' && 'grid_count' in strategy.config
}

export function isSMACrossoverStrategy(strategy: Strategy): strategy is SMACrossoverStrategy {
  return 'config' in strategy && typeof strategy.config === 'object' && 'fast_period' in strategy.config
}


// Strategy performance helpers
export function calculateProfitLossPercentage(totalInvested: string, currentProfitLoss: string): number {
  const invested = parseFloat(totalInvested)
  const pnl = parseFloat(currentProfitLoss)
  
  if (invested === 0) return 0
  return (pnl / invested) * 100
}

export function formatCurrency(value: string | number, currency = 'USD'): string {
  const numValue = typeof value === 'string' ? parseFloat(value) : value
  
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(numValue)
}

export function formatPercentage(value: string | number): string {
  const numValue = typeof value === 'string' ? parseFloat(value) : value
  
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  }).format(numValue / 100)
}

// Default configurations for new strategies
export const DEFAULT_CONFIGS: Record<StrategyType, StrategyConfig> = {
  dca: {
    base_amount: 100,
    frequency: { Weekly: 1 },
    strategy_type: 'Simple',
    pause_on_high_volatility: false,
    pause_on_bear_market: false,
    filters: {},
    sentiment_config: {
      fear_greed_threshold: 25,
      bearish_multiplier: 1.5,
      bullish_multiplier: 0.7
    },
    volatility_config: {
      period: 20,
      low_threshold: 10,
      high_threshold: 30,
      low_volatility_multiplier: 0.8,
      high_volatility_multiplier: 1.5,
      normal_multiplier: 1.0
    },
    stop_loss_percentage: '10',
    take_profit_percentage: '20'
  },
  grid_trading: {
    grid_count: 10,
    range_percentage: '10',
    investment_amount: '1000',
    stop_loss_percentage: '5',
    take_profit_percentage: '15'
  },
  sma_crossover: {
    fast_period: 0, // User must configure
    slow_period: 0, // User must configure
    position_size_pct: 0, // User must configure
    enable_long: true,
    enable_short: false,
    use_market_orders: true,
    risk_settings: {
      stop_loss_pct: 0,
      take_profit_pct: 0,
      max_position_pct: 100,
      min_signal_interval: 0,
      trailing_stop: false,
      trailing_stop_pct: undefined
    },
    filters: {
      min_volume: undefined,
      max_spread_pct: undefined,
      rsi_overbought: undefined,
      rsi_oversold: undefined,
      macd_confirmation: false,
      min_sma_spread_pct: undefined
    },
    confirmation_indicators: {
      use_rsi: false,
      rsi_period: 14,
      use_macd: false,
      macd_fast: 12,
      macd_slow: 26,
      macd_signal: 9,
      use_volume: false,
      volume_period: 20,
      min_volume_multiplier: 1
    }
  }
}

// Strategy validation helpers
export function validateStrategyName(name: string): string | null {
  if (!name || name.trim().length === 0) {
    return 'Strategy name is required'
  }
  if (name.length < 3) {
    return 'Strategy name must be at least 3 characters'
  }
  if (name.length > 50) {
    return 'Strategy name must be less than 50 characters'
  }
  return null
}

export function validateAssetSymbol(symbol: string): string | null {
  if (!symbol || symbol.trim().length === 0) {
    return 'Asset symbol is required'
  }
  if (!/^[A-Z]{3,10}$/.test(symbol)) {
    return 'Asset symbol must be 3-10 uppercase letters'
  }
  return null
}

export function validateInvestmentAmount(amount: string | number): string | null {
  const numAmount = typeof amount === 'string' ? parseFloat(amount) : amount

  if (isNaN(numAmount) || numAmount <= 0) {
    return 'Investment amount must be a positive number'
  }
  if (numAmount < 10) {
    return 'Minimum investment amount is $10'
  }
  if (numAmount > 1000000) {
    return 'Maximum investment amount is $1,000,000'
  }
  return null
}

// Enhanced DCA-specific validation functions
export function validateDCAFrequency(frequencyType: string, frequencyValue: number): string | null {
  if (frequencyValue <= 0) {
    return 'Frequency value must be positive'
  }

  switch (frequencyType) {
    case 'hourly':
      if (frequencyValue > 24) {
        return 'Hourly frequency cannot exceed 24 hours'
      }
      break
    case 'daily':
      if (frequencyValue > 365) {
        return 'Daily frequency cannot exceed 365 days'
      }
      break
    case 'weekly':
      if (frequencyValue > 52) {
        return 'Weekly frequency cannot exceed 52 weeks'
      }
      break
    case 'monthly':
      if (frequencyValue > 12) {
        return 'Monthly frequency cannot exceed 12 months'
      }
      break
    case 'custom':
      if (frequencyValue > 525600) { // 1 year in minutes
        return 'Custom frequency cannot exceed 1 year in minutes'
      }
      break
  }

  return null
}

export function validateVolatilityConfig(config: {
  period: number
  lowThreshold: number
  highThreshold: number
  lowMultiplier: number
  highMultiplier: number
}): string | null {
  if (config.period < 2 || config.period > 100) {
    return 'Volatility period must be between 2 and 100'
  }
  if (config.lowThreshold <= 0 || config.lowThreshold >= 100) {
    return 'Low volatility threshold must be between 0 and 100'
  }
  if (config.highThreshold <= 0 || config.highThreshold >= 100) {
    return 'High volatility threshold must be between 0 and 100'
  }
  if (config.lowThreshold >= config.highThreshold) {
    return 'Low volatility threshold must be less than high threshold'
  }
  if (config.lowMultiplier < 0 || config.lowMultiplier > 5) {
    return 'Low volatility multiplier must be between 0 and 5'
  }
  if (config.highMultiplier < 0 || config.highMultiplier > 5) {
    return 'High volatility multiplier must be between 0 and 5'
  }
  return null
}

export function validateSentimentConfig(config: {
  fearGreedThreshold?: number
  bearishMultiplier: number
  bullishMultiplier: number
}): string | null {
  if (config.fearGreedThreshold !== undefined) {
    if (config.fearGreedThreshold < 0 || config.fearGreedThreshold > 100) {
      return 'Fear & Greed threshold must be between 0 and 100'
    }
  }
  if (config.bearishMultiplier < 0 || config.bearishMultiplier > 10) {
    return 'Bearish multiplier must be between 0 and 10'
  }
  if (config.bullishMultiplier < 0 || config.bullishMultiplier > 10) {
    return 'Bullish multiplier must be between 0 and 10'
  }
  return null
}

export function validateRiskManagement(config: {
  stopLossPercentage?: number
  takeProfitPercentage?: number
  maxPositionSize?: number
}): string | null {
  if (config.stopLossPercentage !== undefined) {
    if (config.stopLossPercentage <= 0 || config.stopLossPercentage > 100) {
      return 'Stop loss percentage must be between 0 and 100'
    }
  }
  if (config.takeProfitPercentage !== undefined) {
    if (config.takeProfitPercentage <= 0 || config.takeProfitPercentage > 1000) {
      return 'Take profit percentage must be between 0 and 1000'
    }
  }
  if (config.maxPositionSize !== undefined) {
    if (config.maxPositionSize <= 0) {
      return 'Maximum position size must be positive'
    }
  }
  return null
}
