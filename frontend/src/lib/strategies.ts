import { 
  StrategyType, 
  StrategyInfo, 
  Strategy, 
  StrategyConfig,
  DCAStrategy,
  GridTradingStrategy,
  SMACrossoverStrategy,
  RSIStrategy,
  MACDStrategy
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
  },
  rsi: {
    type: 'rsi',
    name: 'RSI Strategy',
    description: 'Trade based on Relative Strength Index overbought/oversold conditions',
    icon: '⚡',
    color: 'from-yellow-500/20 to-orange-500/20',
    riskLevel: 'High',
    timeHorizon: 'Short-term (1-8 weeks)',
    features: [
      'RSI-based entry/exit signals',
      'Overbought/oversold detection',
      'Configurable thresholds',
      'Signal strength analysis',
      'Market reversal timing'
    ],
    minInvestment: 200
  },
  macd: {
    type: 'macd',
    name: 'MACD Strategy',
    description: 'Use MACD indicator crossovers and divergences for trading signals',
    icon: '🌊',
    color: 'from-indigo-500/20 to-blue-500/20',
    riskLevel: 'High',
    timeHorizon: 'Short-term (2-12 weeks)',
    features: [
      'MACD line crossover signals',
      'Histogram divergence analysis',
      'Signal line confirmations',
      'Momentum trend detection',
      'Multi-timeframe analysis'
    ],
    minInvestment: 300
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
  return STRATEGY_INFO[type]
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
  return 'config' in strategy && 'base_tranche_size' in strategy
}

export function isGridTradingStrategy(strategy: Strategy): strategy is GridTradingStrategy {
  return 'config' in strategy && typeof strategy.config === 'object' && 'grid_count' in strategy.config
}

export function isSMACrossoverStrategy(strategy: Strategy): strategy is SMACrossoverStrategy {
  return 'config' in strategy && typeof strategy.config === 'object' && 'short_period' in strategy.config
}

export function isRSIStrategy(strategy: Strategy): strategy is RSIStrategy {
  return 'config' in strategy && typeof strategy.config === 'object' && 'rsi_period' in strategy.config
}

export function isMACDStrategy(strategy: Strategy): strategy is MACDStrategy {
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
export const DEFAULT_CONFIGS: Record<StrategyType, Partial<StrategyConfig>> = {
  dca: {
    // Will be handled by existing DCA logic
  },
  grid_trading: {
    grid_count: 10,
    investment_amount: '1000',
    stop_loss_percentage: '5',
    take_profit_percentage: '15'
  },
  sma_crossover: {
    short_period: 20,
    long_period: 50,
    investment_amount: '500',
    stop_loss_percentage: '5',
    take_profit_percentage: '10',
    confirmation_period: 2
  },
  rsi: {
    rsi_period: 14,
    oversold_threshold: 30,
    overbought_threshold: 70,
    investment_amount: '300',
    stop_loss_percentage: '3',
    take_profit_percentage: '8'
  },
  macd: {
    fast_period: 12,
    slow_period: 26,
    signal_period: 9,
    investment_amount: '400',
    stop_loss_percentage: '4',
    take_profit_percentage: '12'
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
