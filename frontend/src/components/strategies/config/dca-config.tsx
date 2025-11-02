"use client"

import { useState, useCallback, useMemo, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  CreateDCAStrategyRequest,
  type DCAConfig,
  type DCAFrequency
} from "@/lib/api"
import {
  validateStrategyName,
  validateAssetSymbol,
  validateInvestmentAmount
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, BarChart3, Clock, Target } from "lucide-react"
import { cn } from "@/lib/utils"

// Frontend DCA form data interface for easier form handling
interface DCAFormData {
  name: string
  asset_symbol: string
  base_amount: number
  strategy_type: 'Simple' | 'RSIBased' | 'VolatilityBased' | 'Dynamic' | 'DipBuying' | 'SentimentBased'
  frequency_type: 'hourly' | 'daily' | 'weekly' | 'monthly' | 'custom'
  frequency_value: number

  // Simple options
  sentiment_multiplier: boolean
  volatility_adjustment: boolean

  // Advanced RSI options
  enable_rsi: boolean
  rsi_period: number
  rsi_oversold: number
  rsi_overbought: number
  rsi_oversold_multiplier: number
  rsi_overbought_multiplier: number

  // Advanced Volatility options
  volatility_period: number
  volatility_low_threshold: number
  volatility_high_threshold: number
  low_volatility_multiplier: number
  high_volatility_multiplier: number

  // Advanced Sentiment options
  fear_greed_threshold: number
  bearish_multiplier: number
  bullish_multiplier: number

  // Advanced Dip Buying options
  dip_threshold_percentage: number
  dip_multiplier: number
  max_dip_purchases: number

  // Risk Management
  enable_stop_loss: boolean
  stop_loss_percentage?: number
  enable_take_profit: boolean
  take_profit_percentage?: number
  max_position_size?: number

  // Filters
  allowed_hours?: number[]
  allowed_weekdays?: number[]
  max_executions_per_day?: number

  // Backtest Configuration
  backtest_start_date?: string
  backtest_end_date?: string
  backtest_interval?: '1m' | '5m' | '15m' | '30m' | '1h' | '4h' | '1d' | '1w'
  backtest_initial_capital?: number
  asset_type?: 'crypto' | 'stock'
}

interface DCAConfigProps {
  initialData?: Partial<CreateDCAStrategyRequest>
  onSubmit: (data: CreateDCAStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (
    config: DCAConfig,
    name: string,
    assetSymbol: string,
    backtestParams: {
      start_date: string
      end_date: string
      interval: string
      initial_capital: number
      asset_type?: 'crypto' | 'stock'
    }
  ) => void
  isLoading?: boolean
  className?: string
}

export function DCAConfigComponent({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: DCAConfigProps) {
  // Initialize form data from backend DCA config if available
  const initializeFormData = (): DCAFormData => {
    const config = initialData?.config
    return {
      name: initialData?.name || '',
      asset_symbol: initialData?.asset_symbol || '',
      base_amount: config?.base_amount || 100,
      strategy_type: config?.strategy_type || 'Simple',
      // Note: Frequency is now auto-derived from backtest_interval
      frequency_type: 'daily',
      frequency_value: 1,

      // Simple options
      sentiment_multiplier: !!config?.sentiment_config,
      volatility_adjustment: !!config?.volatility_config,

      // Advanced RSI options
      enable_rsi: !!config?.rsi_config,
      rsi_period: config?.rsi_config?.period || 14,
      rsi_oversold: config?.rsi_config?.oversold_threshold || 30,
      rsi_overbought: config?.rsi_config?.overbought_threshold || 70,
      rsi_oversold_multiplier: config?.rsi_config?.oversold_multiplier || 2.0,
      rsi_overbought_multiplier: config?.rsi_config?.overbought_multiplier || 0.5,

      // Advanced Volatility options
      volatility_period: config?.volatility_config?.period || 20,
      volatility_low_threshold: config?.volatility_config?.low_threshold || 10,
      volatility_high_threshold: config?.volatility_config?.high_threshold || 30,
      low_volatility_multiplier: config?.volatility_config?.low_volatility_multiplier || 0.8,
      high_volatility_multiplier: config?.volatility_config?.high_volatility_multiplier || 1.5,

      // Advanced Sentiment options
      fear_greed_threshold: config?.sentiment_config?.fear_greed_threshold || 25,
      bearish_multiplier: config?.sentiment_config?.bearish_multiplier || 1.5,
      bullish_multiplier: config?.sentiment_config?.bullish_multiplier || 0.7,

      // Dip Buying options
      dip_threshold_percentage: 5,
      dip_multiplier: 2.0,
      max_dip_purchases: 3,

      // Risk Management
      enable_stop_loss: false,
      stop_loss_percentage: undefined,
      enable_take_profit: false,
      take_profit_percentage: undefined,
      max_position_size: config?.max_position_size,

      // Filters
      allowed_hours: config?.filters?.allowed_hours,
      allowed_weekdays: config?.filters?.allowed_weekdays,
      max_executions_per_day: config?.filters?.max_executions_per_day,

      // Backtest Configuration - Default to 11 months (335 days to stay under 365 day limit)
      backtest_start_date: new Date(Date.now() - 335 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      backtest_end_date: new Date().toISOString().split('T')[0],
      backtest_interval: '1d',
      backtest_initial_capital: 10000,
      asset_type: 'crypto'
    }
  }

  const [formData, setFormData] = useState<DCAFormData>(initializeFormData)
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [activeTab, setActiveTab] = useState("simple")

  // Determine which tabs are relevant based on strategy type
  const relevantTabs = useMemo(() => {
    const base = { simple: true, risk: true, filters: true, advanced: false }

    // Advanced tab only for complex strategies
    if (['RSIBased', 'VolatilityBased', 'SentimentBased', 'Dynamic', 'DipBuying'].includes(formData.strategy_type)) {
      base.advanced = true
    }

    return base
  }, [formData.strategy_type])

  // Auto-enable features and switch tabs based on strategy type
  useEffect(() => {
    if (formData.strategy_type === 'RSIBased') {
      setFormData(prev => ({ ...prev, enable_rsi: true }))
      if (activeTab === 'simple') setActiveTab('advanced')
    }
    if (formData.strategy_type === 'VolatilityBased') {
      setFormData(prev => ({ ...prev, volatility_adjustment: true }))
      if (activeTab === 'simple') setActiveTab('advanced')
    }
    if (formData.strategy_type === 'SentimentBased') {
      setFormData(prev => ({ ...prev, sentiment_multiplier: true }))
      if (activeTab === 'simple') setActiveTab('advanced')
    }
    if (formData.strategy_type === 'Dynamic' || formData.strategy_type === 'DipBuying') {
      if (activeTab === 'simple') setActiveTab('advanced')
    }
    if (formData.strategy_type === 'Simple') {
      if (activeTab === 'advanced') setActiveTab('simple')
    }
  }, [formData.strategy_type, activeTab])

  const validateForm = useCallback(() => {
    const newErrors: Record<string, string> = {}

    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError

    const amountError = validateInvestmentAmount(formData.base_amount)
    if (amountError) newErrors.base_amount = amountError

    // Note: Frequency is now auto-derived from backtest_interval, no validation needed

    // Advanced RSI validations
    if (formData.enable_rsi) {
      if (formData.rsi_period < 2 || formData.rsi_period > 100) {
        newErrors.rsi_period = 'RSI period must be between 2 and 100'
      }
      if (formData.rsi_oversold >= formData.rsi_overbought) {
        newErrors.rsi_thresholds = 'Oversold threshold must be less than overbought threshold'
      }
    }

    // Advanced Volatility validations
    if (formData.volatility_adjustment) {
      if (formData.volatility_period < 2 || formData.volatility_period > 100) {
        newErrors.volatility_period = 'Volatility period must be between 2 and 100'
      }
      if (formData.volatility_low_threshold >= formData.volatility_high_threshold) {
        newErrors.volatility_thresholds = 'Low volatility threshold must be less than high threshold'
      }
    }

    // Risk management validations
    if (formData.enable_stop_loss && (!formData.stop_loss_percentage || formData.stop_loss_percentage <= 0)) {
      newErrors.stop_loss_percentage = 'Stop loss percentage must be positive when enabled'
    }

    if (formData.enable_take_profit && (!formData.take_profit_percentage || formData.take_profit_percentage <= 0)) {
      newErrors.take_profit_percentage = 'Take profit percentage must be positive when enabled'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }, [formData])

  // Convert form data to backend DCA config format
  const convertToBackendConfig = useCallback((): DCAConfig => {
    // Build frequency object based on backtest interval
    // This ensures DCA frequency matches the data granularity
    let frequency: DCAFrequency
    const interval = formData.backtest_interval || '1d'

    switch (interval) {
      case '1m':
      case '5m':
      case '15m':
      case '30m':
        frequency = { Hourly: 1 } // For minute intervals, DCA every hour
        break
      case '1h':
      case '4h':
        frequency = { Hourly: 4 } // For hourly intervals, DCA every 4 hours
        break
      case '1d':
        frequency = { Daily: 1 } // For daily candles, DCA daily
        break
      case '1w':
        frequency = { Weekly: 1 } // For weekly candles, DCA weekly
        break
      default:
        frequency = { Daily: 1 }
    }

    const config: DCAConfig = {
      base_amount: formData.base_amount,
      frequency,
      strategy_type: formData.strategy_type,
      pause_on_high_volatility: false,
      pause_on_bear_market: false,
      filters: {
        allowed_hours: formData.allowed_hours,
        allowed_weekdays: formData.allowed_weekdays,
        max_executions_per_day: formData.max_executions_per_day
      }
    }

    // Add optional configurations
    if (formData.sentiment_multiplier) {
      config.sentiment_config = {
        fear_greed_threshold: formData.fear_greed_threshold,
        bearish_multiplier: formData.bearish_multiplier,
        bullish_multiplier: formData.bullish_multiplier
      }
    }

    if (formData.volatility_adjustment) {
      config.volatility_config = {
        period: formData.volatility_period,
        low_threshold: formData.volatility_low_threshold,
        high_threshold: formData.volatility_high_threshold,
        low_volatility_multiplier: formData.low_volatility_multiplier,
        high_volatility_multiplier: formData.high_volatility_multiplier,
        normal_multiplier: 1.0
      }
    }

    if (formData.enable_rsi) {
      config.rsi_config = {
        period: formData.rsi_period,
        oversold_threshold: formData.rsi_oversold,
        overbought_threshold: formData.rsi_overbought,
        oversold_multiplier: formData.rsi_oversold_multiplier,
        overbought_multiplier: formData.rsi_overbought_multiplier,
        normal_multiplier: 1.0
      }
    }

    // Dip Buying strategy - create dip levels array
    if (formData.strategy_type === 'DipBuying') {
      config.dip_levels = [
        {
          price_drop_percentage: formData.dip_threshold_percentage,
          amount_multiplier: formData.dip_multiplier,
          max_triggers: formData.max_dip_purchases
        },
        {
          price_drop_percentage: formData.dip_threshold_percentage * 2,
          amount_multiplier: formData.dip_multiplier * 1.5,
          max_triggers: Math.max(1, Math.floor(formData.max_dip_purchases / 2))
        },
        {
          price_drop_percentage: formData.dip_threshold_percentage * 4,
          amount_multiplier: formData.dip_multiplier * 3,
          max_triggers: 1
        }
      ]
      config.reference_period_days = 30
    }

    // Dynamic strategy - create dynamic factors
    if (formData.strategy_type === 'Dynamic') {
      config.dynamic_factors = {
        rsi_weight: formData.enable_rsi ? 0.4 : 0,
        volatility_weight: formData.volatility_adjustment ? 0.3 : 0,
        sentiment_weight: formData.sentiment_multiplier ? 0.2 : 0,
        trend_weight: 0.1,
        max_multiplier: 3.0,
        min_multiplier: 0.3
      }
      // Dynamic strategy requires RSI and Volatility configs
      if (!config.rsi_config) {
        config.rsi_config = {
          period: 14,
          oversold_threshold: 30,
          overbought_threshold: 70,
          oversold_multiplier: 2.0,
          overbought_multiplier: 0.5,
          normal_multiplier: 1.0
        }
      }
      if (!config.volatility_config) {
        config.volatility_config = {
          period: 20,
          low_threshold: 10,
          high_threshold: 30,
          low_volatility_multiplier: 0.8,
          high_volatility_multiplier: 1.5,
          normal_multiplier: 1.0
        }
      }
    }

    // Risk management
    if (formData.max_position_size) {
      config.max_position_size = formData.max_position_size
    }

    return config
  }, [formData])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!validateForm()) return

    const dcaConfig = convertToBackendConfig()
    const submitData: CreateDCAStrategyRequest = {
      name: formData.name,
      asset_symbol: formData.asset_symbol,
      config: dcaConfig
    }

    try {
      await onSubmit(submitData)
    } catch (error) {
      console.error('Failed to create DCA strategy:', error)
    }
  }

  const strategyTypeInfo = useMemo(() => {
    switch (formData.strategy_type) {
      case 'Simple':
        return { description: 'Basic DCA with fixed amounts', risk: 'Low', color: 'text-green-400' }
      case 'VolatilityBased':
        return { description: 'Adjusts based on market volatility', risk: 'Medium', color: 'text-yellow-400' }
      case 'RSIBased':
        return { description: 'Uses RSI for timing and amounts', risk: 'Medium', color: 'text-orange-400' }
      case 'Dynamic':
        return { description: 'Combines multiple factors dynamically', risk: 'High', color: 'text-red-400' }
      case 'SentimentBased':
        return { description: 'Adjusts based on market sentiment', risk: 'Medium', color: 'text-blue-400' }
      case 'DipBuying':
        return { description: 'Increases purchases during price dips', risk: 'High', color: 'text-purple-400' }
      default:
        return { description: 'Standard DCA approach', risk: 'Medium', color: 'text-blue-400' }
    }
  }, [formData.strategy_type])

  // Memoize form validation result to prevent unnecessary re-calculations
  const isFormValid = useMemo(() => {
    const nameError = validateStrategyName(formData.name)
    const symbolError = validateAssetSymbol(formData.asset_symbol)
    const amountError = validateInvestmentAmount(formData.base_amount)

    if (nameError || symbolError || amountError) return false

    // Frequency is auto-derived, no validation needed

    if (formData.enable_rsi && formData.rsi_oversold >= formData.rsi_overbought) return false
    if (formData.volatility_adjustment && formData.volatility_low_threshold >= formData.volatility_high_threshold) return false

    return true
  }, [formData])

  // Memoize backtest handler to prevent re-renders
  const handleBacktest = useCallback(() => {
    if (onBacktest && isFormValid) {
      const backtestConfig = convertToBackendConfig()
      console.log('DCA Backtest Config:', JSON.stringify(backtestConfig, null, 2))
      console.log('FormData strategy_type:', formData.strategy_type)
      console.log('FormData enable_rsi:', formData.enable_rsi)
      // Add risk management parameters for backtesting
      const backtestConfigWithRisk = {
        ...backtestConfig,
        stop_loss_percentage: formData.enable_stop_loss ? formData.stop_loss_percentage : undefined,
        take_profit_percentage: formData.enable_take_profit ? formData.take_profit_percentage : undefined
      }

      // Pass backtest parameters
      const backtestParams = {
        start_date: formData.backtest_start_date || new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
        end_date: formData.backtest_end_date || new Date().toISOString().split('T')[0],
        interval: formData.backtest_interval || '1d',
        initial_capital: formData.backtest_initial_capital || 10000,
        asset_type: formData.asset_type || 'crypto'
      }

      onBacktest(backtestConfigWithRisk, formData.name, formData.asset_symbol, backtestParams)
    }
  }, [onBacktest, formData, isFormValid, convertToBackendConfig])

  return (
    <Card className={cn(
      "bg-gradient-to-br from-blue-500/20 to-cyan-500/20 backdrop-blur-xl border border-white/10",
      className
    )}>
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />

      <div className="relative z-10">
        <CardHeader className="text-center space-y-2">
          <div className="text-4xl mx-auto">📊</div>
          <CardTitle className="text-2xl font-bold text-white/90">
            DCA Strategy Configuration
          </CardTitle>
          <CardDescription className="text-white/60">
            Advanced Dollar Cost Averaging with comprehensive market analysis and risk management
          </CardDescription>
        </CardHeader>

        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-6">
            {/* Basic Information */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="name" className="text-white/80">Strategy Name</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="My Enhanced DCA Strategy"
                  className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                />
                {errors.name && (
                  <p className="text-red-400 text-sm flex items-center space-x-1">
                    <AlertTriangle className="h-4 w-4" />
                    <span>{errors.name}</span>
                  </p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="asset_type" className="text-white/80 font-medium">Asset Type</Label>
                <Select
                  value={formData.asset_type || 'crypto'}
                  onValueChange={(value: 'crypto' | 'stock') => setFormData({ ...formData, asset_type: value })}
                >
                  <SelectTrigger className="bg-white/10 border-white/20 text-white hover:bg-white/15 transition-colors">
                    <SelectValue placeholder="Select asset type" />
                  </SelectTrigger>
                  <SelectContent className="bg-slate-900 border-white/20">
                    <SelectItem value="crypto" className="text-white hover:bg-white/10 cursor-pointer">
                      <div className="flex items-center gap-2">
                        <span className="text-orange-400">₿</span>
                        <span>Cryptocurrency</span>
                      </div>
                    </SelectItem>
                    <SelectItem value="stock" className="text-white hover:bg-white/10 cursor-pointer">
                      <div className="flex items-center gap-2">
                        <span className="text-blue-400">📈</span>
                        <span>Stock Market</span>
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
                {formData.asset_type === 'stock' && (
                  <p className="text-amber-400/80 text-xs flex items-center gap-1">
                    <span>ℹ️</span>
                    <span>Stocks support daily intervals only (1d, 1w, 1M)</span>
                  </p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="asset_symbol" className="text-white/80">Asset Symbol</Label>
                <Input
                  id="asset_symbol"
                  value={formData.asset_symbol}
                  onChange={(e) => setFormData({ ...formData, asset_symbol: e.target.value.toUpperCase() })}
                  placeholder={formData.asset_type === 'stock' ? 'AAPL' : 'BTC'}
                  className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                />
                {errors.asset_symbol && (
                  <p className="text-red-400 text-sm flex items-center space-x-1">
                    <AlertTriangle className="h-4 w-4" />
                    <span>{errors.asset_symbol}</span>
                  </p>
                )}
              </div>
            </div>

            {/* Core Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Target className="h-5 w-5" />
                <span>Core Configuration</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="base_amount" className="text-white/80">Base Amount ($)</Label>
                  <Input
                    id="base_amount"
                    type="number"
                    step="0.01"
                    value={formData.base_amount}
                    onChange={(e) => setFormData({
                      ...formData,
                      base_amount: parseFloat(e.target.value) || 0
                    })}
                    placeholder="1000"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.base_amount && (
                    <p className="text-red-400 text-sm">{errors.base_amount}</p>
                  )}
                  <p className="text-xs text-white/50">
                    Amount to invest per DCA execution
                  </p>
                </div>

                <div className="space-y-2">
                  <Label className="text-white/80">Strategy Type</Label>
                  <Select
                    value={formData.strategy_type}
                    onValueChange={(value) => setFormData({ ...formData, strategy_type: value as DCAFormData['strategy_type'] })}
                  >
                    <SelectTrigger className="bg-white/10 border-white/20 text-white hover:bg-white/20">
                      <SelectValue placeholder="Select strategy type" />
                    </SelectTrigger>
                    <SelectContent className="bg-slate-900 border-white/30 backdrop-blur-xl">
                      <SelectItem value="Simple" className="text-white hover:bg-green-500/20 focus:bg-green-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>Simple</span>
                          <span className="text-xs text-green-400 ml-2">Low Risk</span>
                        </div>
                      </SelectItem>
                      <SelectItem value="RSIBased" className="text-white hover:bg-orange-500/20 focus:bg-orange-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>RSI Based</span>
                          <span className="text-xs text-orange-400 ml-2">Medium Risk</span>
                        </div>
                      </SelectItem>
                      <SelectItem value="VolatilityBased" className="text-white hover:bg-yellow-500/20 focus:bg-yellow-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>Volatility Based</span>
                          <span className="text-xs text-yellow-400 ml-2">Medium Risk</span>
                        </div>
                      </SelectItem>
                      <SelectItem value="SentimentBased" className="text-white hover:bg-blue-500/20 focus:bg-blue-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>Sentiment Based</span>
                          <span className="text-xs text-blue-400 ml-2">Medium Risk</span>
                        </div>
                      </SelectItem>
                      <SelectItem value="Dynamic" className="text-white hover:bg-red-500/20 focus:bg-red-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>Dynamic</span>
                          <span className="text-xs text-red-400 ml-2">High Risk</span>
                        </div>
                      </SelectItem>
                      <SelectItem value="DipBuying" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">
                        <div className="flex items-center justify-between w-full">
                          <span>Dip Buying</span>
                          <span className="text-xs text-purple-400 ml-2">High Risk</span>
                        </div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                  <div className="text-xs text-white/60">
                    <span className={cn("font-medium", strategyTypeInfo.color)}>
                      {strategyTypeInfo.risk} Risk
                    </span>
                    : {strategyTypeInfo.description}
                  </div>
                </div>
              </div>
            </div>


            {/* Strategy Type Info Banner */}
            <div className={cn(
              "p-4 rounded-lg border-2 transition-all",
              strategyTypeInfo.color === 'text-green-400' && "bg-green-500/10 border-green-500/30",
              strategyTypeInfo.color === 'text-yellow-400' && "bg-yellow-500/10 border-yellow-500/30",
              strategyTypeInfo.color === 'text-orange-400' && "bg-orange-500/10 border-orange-500/30",
              strategyTypeInfo.color === 'text-red-400' && "bg-red-500/10 border-red-500/30",
              strategyTypeInfo.color === 'text-blue-400' && "bg-blue-500/10 border-blue-500/30",
              strategyTypeInfo.color === 'text-purple-400' && "bg-purple-500/10 border-purple-500/30"
            )}>
              <div className="flex items-center justify-between">
                <div>
                  <span className="text-white/70 text-sm">Selected Strategy: </span>
                  <span className={cn("font-bold text-lg", strategyTypeInfo.color)}>
                    {formData.strategy_type}
                  </span>
                </div>
                <div className={cn("px-3 py-1 rounded-full text-xs font-semibold", strategyTypeInfo.color)}>
                  {strategyTypeInfo.risk} Risk
                </div>
              </div>
              <p className="text-white/60 text-sm mt-2">{strategyTypeInfo.description}</p>
            </div>

            {/* Configuration Tabs */}
            <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
              <TabsList className={cn(
                "grid w-full bg-white/5",
                relevantTabs.advanced ? "grid-cols-4" : "grid-cols-3"
              )}>
                <TabsTrigger value="simple" className="data-[state=active]:bg-blue-500/20">
                  Overview
                </TabsTrigger>
                {relevantTabs.advanced && (
                  <TabsTrigger value="advanced" className="data-[state=active]:bg-purple-500/20">
                    {formData.strategy_type === 'RSIBased' ? 'RSI Config' :
                     formData.strategy_type === 'VolatilityBased' ? 'Volatility Config' :
                     formData.strategy_type === 'SentimentBased' ? 'Sentiment Config' :
                     formData.strategy_type === 'DipBuying' ? 'Dip Buying' :
                     formData.strategy_type === 'Dynamic' ? 'Dynamic Factors' : 'Advanced'}
                  </TabsTrigger>
                )}
                <TabsTrigger value="risk" className="data-[state=active]:bg-orange-500/20">
                  Risk & Limits
                </TabsTrigger>
                <TabsTrigger value="filters" className="data-[state=active]:bg-cyan-500/20">
                  Backtest Setup
                </TabsTrigger>
              </TabsList>

              <TabsContent value="simple" className="space-y-4">
                {/* Tab Description */}
                <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                  <p className="text-sm text-blue-200">
                    {formData.strategy_type === 'Simple' ? (
                      <><strong>Simple Strategy:</strong> Pure Dollar Cost Averaging - invest a fixed amount at regular intervals. No complex market analysis needed.</>
                    ) : (
                      <><strong>Quick Options:</strong> Toggle basic features for your {formData.strategy_type} strategy. Configure advanced parameters in the Advanced tab.</>
                    )}
                  </p>
                </div>

                {/* Show summary for Simple strategy */}
                {formData.strategy_type === 'Simple' && (
                  <div className="p-4 bg-white/5 rounded-lg border border-white/10">
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="text-white/90 font-semibold">Strategy Summary</h4>
                      <span className="text-green-400 text-sm">✓ Ready to Test</span>
                    </div>
                    <div className="space-y-2 text-sm text-white/70">
                      <div className="flex justify-between">
                        <span>Investment per cycle:</span>
                        <span className="text-white font-semibold">${formData.base_amount}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>DCA Frequency:</span>
                        <span className="text-white font-semibold">
                          {formData.backtest_interval === '1w' ? 'Weekly' :
                           formData.backtest_interval === '1d' ? 'Daily' :
                           formData.backtest_interval === '1h' || formData.backtest_interval === '4h' ? 'Every 4 hours' :
                           'Hourly'} (synced with interval)
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span>Data interval:</span>
                        <span className="text-white font-semibold">
                          {formData.backtest_interval}
                        </span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Quick toggles for non-Simple strategies */}
                {formData.strategy_type !== 'Simple' && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90 flex items-center space-x-2">
                      <Zap className="h-4 w-4" />
                      <span>Quick Toggles</span>
                    </h4>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      {formData.strategy_type !== 'SentimentBased' && (
                        <div className="p-4 bg-white/5 rounded-lg border border-white/10 space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-white/90 font-medium">Sentiment Analysis</Label>
                            <Switch
                              checked={formData.sentiment_multiplier}
                              onCheckedChange={(checked) => setFormData({ ...formData, sentiment_multiplier: checked })}
                            />
                          </div>
                          <p className="text-xs text-white/60">
                            Adjust based on Fear & Greed Index
                          </p>
                        </div>
                      )}

                      {formData.strategy_type !== 'VolatilityBased' && (
                        <div className="p-4 bg-white/5 rounded-lg border border-white/10 space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-white/90 font-medium">Volatility Adjustment</Label>
                            <Switch
                              checked={formData.volatility_adjustment}
                              onCheckedChange={(checked) => setFormData({ ...formData, volatility_adjustment: checked })}
                            />
                          </div>
                          <p className="text-xs text-white/60">
                            Buy more during high volatility
                          </p>
                        </div>
                      )}

                      {formData.strategy_type !== 'RSIBased' && (
                        <div className="p-4 bg-white/5 rounded-lg border border-white/10 space-y-2">
                          <div className="flex items-center justify-between">
                            <Label className="text-white/90 font-medium">RSI Timing</Label>
                            <Switch
                              checked={formData.enable_rsi}
                              onCheckedChange={(checked) => setFormData({ ...formData, enable_rsi: checked })}
                            />
                          </div>
                          <p className="text-xs text-white/60">
                            Use RSI for market timing
                          </p>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </TabsContent>

              <TabsContent value="advanced" className="space-y-6">
                {/* Tab Description */}
                <div className="p-3 bg-purple-500/10 border border-purple-500/20 rounded-lg">
                  <p className="text-sm text-purple-200">
                    <strong>{formData.strategy_type} Configuration:</strong> {
                      formData.strategy_type === 'RSIBased' ? 'Configure RSI thresholds and multipliers to time your DCA purchases based on overbought/oversold conditions.' :
                      formData.strategy_type === 'VolatilityBased' ? 'Set volatility thresholds to increase purchases during market turbulence.' :
                      formData.strategy_type === 'SentimentBased' ? 'Adjust purchase amounts based on Fear & Greed Index and market sentiment.' :
                      formData.strategy_type === 'DipBuying' ? 'Configure price drop levels and multipliers for aggressive dip buying.' :
                      formData.strategy_type === 'Dynamic' ? 'Combine multiple factors with custom weights for optimal DCA timing.' :
                      'Fine-tune advanced parameters for your strategy.'
                    }
                  </p>
                </div>

                {/* Advanced RSI Configuration */}
                {(formData.strategy_type === 'RSIBased' || formData.strategy_type === 'Dynamic') && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90 flex items-center gap-2">
                      <BarChart3 className="h-5 w-5 text-orange-400" />
                      RSI Parameters
                    </h4>

                    {formData.enable_rsi && (
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label>RSI Period: {formData.rsi_period}</Label>
                          <Slider
                            value={[formData.rsi_period]}
                            onValueChange={([value]) => setFormData({ ...formData, rsi_period: value })}
                            min={2}
                            max={100}
                            step={1}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Oversold Threshold: {formData.rsi_oversold}</Label>
                          <Slider
                            value={[formData.rsi_oversold]}
                            onValueChange={([value]) => setFormData({ ...formData, rsi_oversold: value })}
                            min={10}
                            max={50}
                            step={1}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Overbought Threshold: {formData.rsi_overbought}</Label>
                          <Slider
                            value={[formData.rsi_overbought]}
                            onValueChange={([value]) => setFormData({ ...formData, rsi_overbought: value })}
                            min={50}
                            max={90}
                            step={1}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Oversold Multiplier: {formData.rsi_oversold_multiplier}x</Label>
                          <Slider
                            value={[formData.rsi_oversold_multiplier]}
                            onValueChange={([value]) => setFormData({ ...formData, rsi_oversold_multiplier: value })}
                            min={1}
                            max={5}
                            step={0.1}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                )}

                {/* Advanced Volatility Configuration */}
                {(formData.strategy_type === 'VolatilityBased' || formData.strategy_type === 'Dynamic') && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90 flex items-center gap-2">
                      <TrendingUp className="h-5 w-5 text-yellow-400" />
                      Volatility Parameters
                    </h4>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label>Volatility Period: {formData.volatility_period}</Label>
                        <Slider
                          value={[formData.volatility_period]}
                          onValueChange={([value]) => setFormData({ ...formData, volatility_period: value })}
                          min={5}
                          max={100}
                          step={1}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Low Volatility Threshold: {formData.volatility_low_threshold}%</Label>
                        <Slider
                          value={[formData.volatility_low_threshold]}
                          onValueChange={([value]) => setFormData({ ...formData, volatility_low_threshold: value })}
                          min={1}
                          max={20}
                          step={0.5}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>High Volatility Threshold: {formData.volatility_high_threshold}%</Label>
                        <Slider
                          value={[formData.volatility_high_threshold]}
                          onValueChange={([value]) => setFormData({ ...formData, volatility_high_threshold: value })}
                          min={20}
                          max={100}
                          step={1}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>High Vol Multiplier: {formData.high_volatility_multiplier}x</Label>
                        <Slider
                          value={[formData.high_volatility_multiplier]}
                          onValueChange={([value]) => setFormData({ ...formData, high_volatility_multiplier: value })}
                          min={1}
                          max={3}
                          step={0.1}
                        />
                      </div>
                    </div>
                  </div>
                )}

                {/* Advanced Sentiment Configuration */}
                {(formData.strategy_type === 'SentimentBased' || formData.strategy_type === 'Dynamic') && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90 flex items-center gap-2">
                      <Zap className="h-5 w-5 text-blue-400" />
                      Sentiment Parameters
                    </h4>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <Label>Fear & Greed Threshold: {formData.fear_greed_threshold}</Label>
                        <Slider
                          value={[formData.fear_greed_threshold]}
                          onValueChange={([value]) => setFormData({ ...formData, fear_greed_threshold: value })}
                          min={0}
                          max={50}
                          step={1}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Bearish Multiplier: {formData.bearish_multiplier}x</Label>
                        <Slider
                          value={[formData.bearish_multiplier]}
                          onValueChange={([value]) => setFormData({ ...formData, bearish_multiplier: value })}
                          min={1}
                          max={3}
                          step={0.1}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Bullish Multiplier: {formData.bullish_multiplier}x</Label>
                        <Slider
                          value={[formData.bullish_multiplier]}
                          onValueChange={([value]) => setFormData({ ...formData, bullish_multiplier: value })}
                          min={0.1}
                          max={1}
                          step={0.05}
                        />
                      </div>
                    </div>
                  </div>
                )}

                {/* Advanced Dip Buying Configuration */}
                {formData.strategy_type === 'DipBuying' && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90 flex items-center gap-2">
                      <Target className="h-5 w-5 text-purple-400" />
                      Dip Buying Parameters
                    </h4>
                    <div className="p-3 bg-purple-500/10 border border-purple-500/20 rounded-lg text-sm text-purple-200">
                      <p>💡 The system will automatically create 3 dip levels based on your threshold:</p>
                      <ul className="list-disc list-inside mt-2 space-y-1 text-xs">
                        <li>Level 1: {formData.dip_threshold_percentage}% drop → {formData.dip_multiplier}x multiplier</li>
                        <li>Level 2: {formData.dip_threshold_percentage * 2}% drop → {(formData.dip_multiplier * 1.5).toFixed(1)}x multiplier</li>
                        <li>Level 3: {formData.dip_threshold_percentage * 4}% drop → {(formData.dip_multiplier * 3).toFixed(1)}x multiplier</li>
                      </ul>
                    </div>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <Label>Dip Threshold: {formData.dip_threshold_percentage}%</Label>
                        <Slider
                          value={[formData.dip_threshold_percentage]}
                          onValueChange={([value]) => setFormData({ ...formData, dip_threshold_percentage: value })}
                          min={1}
                          max={20}
                          step={0.5}
                        />
                        <p className="text-xs text-white/60">Price drop % that triggers increased buying</p>
                      </div>
                      <div className="space-y-2">
                        <Label>Dip Multiplier: {formData.dip_multiplier}x</Label>
                        <Slider
                          value={[formData.dip_multiplier]}
                          onValueChange={([value]) => setFormData({ ...formData, dip_multiplier: value })}
                          min={1}
                          max={5}
                          step={0.1}
                        />
                        <p className="text-xs text-white/60">Purchase amount multiplier during dips</p>
                      </div>
                      <div className="space-y-2">
                        <Label>Max Dip Purchases: {formData.max_dip_purchases}</Label>
                        <Slider
                          value={[formData.max_dip_purchases]}
                          onValueChange={([value]) => setFormData({ ...formData, max_dip_purchases: value })}
                          min={1}
                          max={10}
                          step={1}
                        />
                        <p className="text-xs text-white/60">Maximum consecutive dip purchases</p>
                      </div>
                    </div>
                  </div>
                )}
              </TabsContent>

              <TabsContent value="risk" className="space-y-4">
                {/* Tab Description */}
                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
                  <p className="text-sm text-red-200">
                    <strong>Risk Management:</strong> Set stop-loss, take-profit, and position size limits to protect your investment and lock in gains.
                  </p>
                </div>

                {/* Risk Management */}
                <div className="space-y-4">
                  <h4 className="text-md font-semibold text-white/90 flex items-center space-x-2">
                    <Zap className="h-4 w-4" />
                    <span>Risk Management</span>
                  </h4>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    {/* Stop Loss */}
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/80">Stop Loss</Label>
                        <Switch
                          checked={formData.enable_stop_loss}
                          onCheckedChange={(checked) => setFormData({ ...formData, enable_stop_loss: checked })}
                        />
                      </div>
                      {formData.enable_stop_loss && (
                        <div className="space-y-2">
                          <Input
                            type="number"
                            step="0.1"
                            value={formData.stop_loss_percentage || ''}
                            onChange={(e) => setFormData({
                              ...formData,
                              stop_loss_percentage: parseFloat(e.target.value) || undefined
                            })}
                            placeholder="10"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                          <p className="text-xs text-white/60">
                            Stop all purchases if total loss exceeds this percentage
                          </p>
                        </div>
                      )}
                    </div>

                    {/* Take Profit */}
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/80">Take Profit</Label>
                        <Switch
                          checked={formData.enable_take_profit}
                          onCheckedChange={(checked) => setFormData({ ...formData, enable_take_profit: checked })}
                        />
                      </div>
                      {formData.enable_take_profit && (
                        <div className="space-y-2">
                          <Input
                            type="number"
                            step="0.1"
                            value={formData.take_profit_percentage || ''}
                            onChange={(e) => setFormData({
                              ...formData,
                              take_profit_percentage: parseFloat(e.target.value) || undefined
                            })}
                            placeholder="50"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                          <p className="text-xs text-white/60">
                            Stop strategy when total profit reaches this percentage
                          </p>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Max Position Size */}
                  <div className="space-y-2">
                    <Label htmlFor="max_position_size" className="text-white/80">Max Position Size ($)</Label>
                    <Input
                      id="max_position_size"
                      type="number"
                      step="100"
                      value={formData.max_position_size || ''}
                      onChange={(e) => setFormData({
                        ...formData,
                        max_position_size: parseFloat(e.target.value) || undefined
                      })}
                      placeholder="10000"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">
                      Maximum total value to invest in this strategy
                    </p>
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="filters" className="space-y-4">
                {/* Tab Description */}
                <div className="p-3 bg-cyan-500/10 border border-cyan-500/20 rounded-lg">
                  <p className="text-sm text-cyan-200">
                    <strong>Backtest Configuration:</strong> Set the date range, interval (kline), and initial capital for your backtest simulation.
                  </p>
                  <p className="text-xs text-cyan-300 mt-2">
                    💡 <strong>Note:</strong> The candlestick interval also determines DCA execution frequency (e.g., 1d interval = daily DCA, 1w = weekly DCA)
                  </p>
                </div>

                {/* Backtest Settings */}
                <div className="space-y-4">
                  <h4 className="text-md font-semibold text-white/90 flex items-center gap-2">
                    <BarChart3 className="h-5 w-5 text-cyan-400" />
                    Backtest Parameters
                  </h4>

                  {/* Quick Period Presets */}
                  <div className="space-y-2">
                    <Label className="text-white/80">Quick Presets</Label>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const today = new Date().toISOString().split('T')[0]
                          const start = new Date(Date.now() - 180 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                          setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                        }}
                        className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                      >
                        6 Months
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const today = new Date().toISOString().split('T')[0]
                          const start = new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                          setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                        }}
                        className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                      >
                        1 Year
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const today = new Date().toISOString().split('T')[0]
                          const start = new Date(Date.now() - 730 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                          setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                        }}
                        className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                      >
                        2 Years
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => {
                          const today = new Date().toISOString().split('T')[0]
                          const start = new Date(Date.now() - 1095 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                          setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                        }}
                        className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                      >
                        3 Years
                      </Button>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {/* Date Range */}
                    <div className="space-y-2">
                      <Label className="text-white/80">Start Date</Label>
                      <Input
                        type="date"
                        value={formData.backtest_start_date}
                        onChange={(e) => setFormData({ ...formData, backtest_start_date: e.target.value })}
                        className="bg-white/10 border-white/20 text-white"
                      />
                      <p className="text-xs text-white/60">Beginning of backtest period (up to 5 years ago)</p>
                    </div>

                    <div className="space-y-2">
                      <Label className="text-white/80">End Date</Label>
                      <Input
                        type="date"
                        value={formData.backtest_end_date}
                        onChange={(e) => setFormData({ ...formData, backtest_end_date: e.target.value })}
                        max={new Date().toISOString().split('T')[0]}
                        className="bg-white/10 border-white/20 text-white"
                      />
                      <p className="text-xs text-white/60">End of backtest period</p>
                    </div>

                    {/* Kline Interval */}
                    <div className="space-y-2">
                      <Label className="text-white/80">Candlestick Interval (Kline)</Label>
                      <Select
                        value={formData.backtest_interval}
                        onValueChange={(value) => setFormData({ ...formData, backtest_interval: value as DCAFormData['backtest_interval'] })}
                      >
                        <SelectTrigger className="bg-white/10 border-white/20 text-white hover:bg-white/20">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="bg-slate-900 border-white/30 backdrop-blur-xl">
                          <SelectItem value="1m" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">1 Minute</SelectItem>
                          <SelectItem value="5m" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">5 Minutes</SelectItem>
                          <SelectItem value="15m" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">15 Minutes</SelectItem>
                          <SelectItem value="30m" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">30 Minutes</SelectItem>
                          <SelectItem value="1h" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">1 Hour</SelectItem>
                          <SelectItem value="4h" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">4 Hours</SelectItem>
                          <SelectItem value="1d" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">1 Day (Recommended)</SelectItem>
                          <SelectItem value="1w" className="text-white hover:bg-cyan-500/20 focus:bg-cyan-500/20 cursor-pointer">1 Week</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-white/60">
                        Smaller intervals = more data points, longer processing
                      </p>
                    </div>

                    {/* Initial Capital */}
                    <div className="space-y-2">
                      <Label className="text-white/80">Initial Capital ($)</Label>
                      <Input
                        type="number"
                        step="100"
                        min="100"
                        value={formData.backtest_initial_capital}
                        onChange={(e) => setFormData({
                          ...formData,
                          backtest_initial_capital: parseFloat(e.target.value) || 10000
                        })}
                        placeholder="10000"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">Starting portfolio value</p>
                    </div>
                  </div>

                  {/* Backtest Summary */}
                  <div className="p-4 bg-white/5 rounded-lg border border-white/10">
                    <h5 className="text-white/90 font-semibold mb-2">Backtest Summary</h5>
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div className="text-white/70">Duration:</div>
                      <div className="text-white font-semibold">
                        {formData.backtest_start_date && formData.backtest_end_date
                          ? (() => {
                              const days = Math.ceil((new Date(formData.backtest_end_date).getTime() - new Date(formData.backtest_start_date).getTime()) / (1000 * 60 * 60 * 24))
                              const years = (days / 365).toFixed(1)
                              return days > 365 ? `${days} days (~${years} years)` : `${days} days`
                            })()
                          : '0 days'}
                      </div>
                      <div className="text-white/70">Data interval:</div>
                      <div className="text-white font-semibold">{formData.backtest_interval}</div>
                      <div className="text-white/70">Starting capital:</div>
                      <div className="text-white font-semibold">${formData.backtest_initial_capital?.toLocaleString()}</div>
                      <div className="text-white/70">Max duration:</div>
                      <div className="text-white/60 text-xs">5 years (1,825 days)</div>
                    </div>

                    {/* Warning for long periods with small intervals */}
                    {formData.backtest_start_date && formData.backtest_end_date &&
                     Math.ceil((new Date(formData.backtest_end_date).getTime() - new Date(formData.backtest_start_date).getTime()) / (1000 * 60 * 60 * 24)) > 730 &&
                     ['1m', '5m', '15m', '30m', '1h'].includes(formData.backtest_interval || '') && (
                      <div className="mt-3 p-2 bg-yellow-500/10 border border-yellow-500/20 rounded text-xs text-yellow-300">
                        ⚠️ Long backtest period with small interval may take longer to process
                      </div>
                    )}
                  </div>
                </div>
              </TabsContent>
            </Tabs>

            {/* Validation Errors */}
            {Object.keys(errors).length > 0 && (
              <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                <div className="flex items-center space-x-2 mb-2">
                  <AlertTriangle className="h-5 w-5 text-red-400" />
                  <span className="font-medium text-red-400">Configuration Errors</span>
                </div>
                <ul className="text-sm text-red-300 space-y-1">
                  {Object.entries(errors).map(([field, error]) => (
                    <li key={field}>• {error}</li>
                  ))}
                </ul>
              </div>
            )}

            {/* Strategy-Specific Tips */}
            <div className={cn(
              "rounded-lg p-4 border-2",
              formData.strategy_type === 'Simple' && "bg-green-500/10 border-green-500/20",
              formData.strategy_type === 'RSIBased' && "bg-orange-500/10 border-orange-500/20",
              formData.strategy_type === 'VolatilityBased' && "bg-yellow-500/10 border-yellow-500/20",
              formData.strategy_type === 'SentimentBased' && "bg-blue-500/10 border-blue-500/20",
              formData.strategy_type === 'Dynamic' && "bg-red-500/10 border-red-500/20",
              formData.strategy_type === 'DipBuying' && "bg-purple-500/10 border-purple-500/20"
            )}>
              <h4 className="font-medium text-white/90 mb-3 flex items-center gap-2">
                💡 {formData.strategy_type} Strategy Tips
              </h4>
              <div className="text-sm text-white/70 space-y-2">
                {formData.strategy_type === 'Simple' && (
                  <>
                    <p>• <strong>Best for:</strong> Long-term investors who want predictable, automatic investing</p>
                    <p>• <strong>Recommended frequency:</strong> Weekly or bi-weekly for most investors</p>
                    <p>• <strong>Pro tip:</strong> Start with $50-$200 per cycle to test the strategy</p>
                  </>
                )}
                {formData.strategy_type === 'RSIBased' && (
                  <>
                    <p>• <strong>Best for:</strong> Timing market entries during oversold conditions</p>
                    <p>• <strong>Recommended:</strong> Set oversold threshold around 30, overbought around 70</p>
                    <p>• <strong>Pro tip:</strong> Higher multipliers (2-3x) during oversold periods can maximize dip buying</p>
                  </>
                )}
                {formData.strategy_type === 'VolatilityBased' && (
                  <>
                    <p>• <strong>Best for:</strong> Taking advantage of market turbulence and fear</p>
                    <p>• <strong>Recommended:</strong> Set high volatility threshold around 30-40%</p>
                    <p>• <strong>Pro tip:</strong> Use higher multipliers (1.5-2x) during high volatility to buy the fear</p>
                  </>
                )}
                {formData.strategy_type === 'SentimentBased' && (
                  <>
                    <p>• <strong>Best for:</strong> Contrarian investors who buy when others are fearful</p>
                    <p>• <strong>Recommended:</strong> Fear threshold around 20-30 for extreme fear buying</p>
                    <p>• <strong>Pro tip:</strong> Higher bearish multipliers help accumulate during market panic</p>
                  </>
                )}
                {formData.strategy_type === 'Dynamic' && (
                  <>
                    <p>• <strong>Best for:</strong> Advanced investors wanting multi-factor optimization</p>
                    <p>• <strong>Recommended:</strong> Enable at least RSI and Volatility for best results</p>
                    <p>• <strong>Pro tip:</strong> This strategy automatically adjusts based on all enabled factors</p>
                  </>
                )}
                {formData.strategy_type === 'DipBuying' && (
                  <>
                    <p>• <strong>Best for:</strong> Aggressive accumulation during price drops</p>
                    <p>• <strong>Recommended:</strong> Set initial dip threshold around 5-10%</p>
                    <p>• <strong>Pro tip:</strong> The strategy creates 3 levels automatically - larger dips = bigger buys</p>
                  </>
                )}
                <p className="text-yellow-400/80 mt-3">
                  ⚠️ Always test your strategy with backtesting before deploying with real capital!
                </p>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-col sm:flex-row gap-3 pt-6">
              <Button
                type="button"
                variant="outline"
                onClick={onCancel}
                className="border-white/20 text-white/80 hover:bg-white/10"
                disabled={isLoading}
              >
                Cancel
              </Button>

              {onBacktest && (
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleBacktest}
                  className="border-blue-500/30 text-blue-400 hover:bg-blue-500/10"
                  disabled={isLoading || !isFormValid}
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Backtest Configuration
                </Button>
              )}

              <Button
                type="submit"
                className="bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-500 hover:to-cyan-500 text-white flex-1"
                disabled={isLoading || !isFormValid}
              >
                {isLoading ? (
                  <div className="flex items-center space-x-2">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>Creating Strategy...</span>
                  </div>
                ) : (
                  <>
                    <TrendingUp className="mr-2 h-4 w-4" />
                    Create DCA Strategy
                  </>
                )}
              </Button>
            </div>
          </CardContent>
        </form>
      </div>
    </Card>
  )
}

// Export with the expected name for compatibility
export { DCAConfigComponent as DCAConfig }