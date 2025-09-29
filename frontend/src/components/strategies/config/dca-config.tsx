"use client"

import { useState, useCallback, useMemo } from "react"
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
}

interface DCAConfigProps {
  initialData?: Partial<CreateDCAStrategyRequest>
  onSubmit: (data: CreateDCAStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: DCAConfig, name: string, assetSymbol: string) => void
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
      base_amount: config?.base_amount || 1000,
      strategy_type: config?.strategy_type || 'Simple',
      frequency_type: config?.frequency?.Daily ? 'daily' :
                     config?.frequency?.Weekly ? 'weekly' :
                     config?.frequency?.Hourly ? 'hourly' :
                     config?.frequency?.Monthly ? 'monthly' : 'custom',
      frequency_value: config?.frequency?.Daily || config?.frequency?.Weekly ||
                      config?.frequency?.Hourly || config?.frequency?.Monthly ||
                      config?.frequency?.Custom || 1,

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
      max_executions_per_day: config?.filters?.max_executions_per_day
    }
  }

  const [formData, setFormData] = useState<DCAFormData>(initializeFormData)
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [activeTab, setActiveTab] = useState("simple")

  const validateForm = useCallback(() => {
    const newErrors: Record<string, string> = {}

    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError

    const amountError = validateInvestmentAmount(formData.base_amount)
    if (amountError) newErrors.base_amount = amountError

    // Frequency validations
    if (formData.frequency_value <= 0) {
      newErrors.frequency_value = 'Frequency value must be positive'
    }

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
    // Build frequency object
    let frequency: DCAFrequency
    switch (formData.frequency_type) {
      case 'hourly':
        frequency = { Hourly: formData.frequency_value }
        break
      case 'daily':
        frequency = { Daily: formData.frequency_value }
        break
      case 'weekly':
        frequency = { Weekly: formData.frequency_value }
        break
      case 'monthly':
        frequency = { Monthly: formData.frequency_value }
        break
      case 'custom':
        frequency = { Custom: formData.frequency_value }
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

    if (formData.frequency_value <= 0) return false

    if (formData.enable_rsi && formData.rsi_oversold >= formData.rsi_overbought) return false
    if (formData.volatility_adjustment && formData.volatility_low_threshold >= formData.volatility_high_threshold) return false

    return true
  }, [formData])

  // Memoize backtest handler to prevent re-renders
  const handleBacktest = useCallback(() => {
    if (onBacktest && isFormValid) {
      const backtestConfig = convertToBackendConfig()
      // Add risk management parameters for backtesting
      const backtestConfigWithRisk = {
        ...backtestConfig,
        stop_loss_percentage: formData.enable_stop_loss ? formData.stop_loss_percentage : undefined,
        take_profit_percentage: formData.enable_take_profit ? formData.take_profit_percentage : undefined
      }
      onBacktest(backtestConfigWithRisk, formData.name, formData.asset_symbol)
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
                <Label htmlFor="asset_symbol" className="text-white/80">Asset Symbol</Label>
                <Input
                  id="asset_symbol"
                  value={formData.asset_symbol}
                  onChange={(e) => setFormData({ ...formData, asset_symbol: e.target.value.toUpperCase() })}
                  placeholder="BTC"
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

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
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
                </div>

                <div className="space-y-2">
                  <Label className="text-white/80">Strategy Type</Label>
                  <Select
                    value={formData.strategy_type}
                    onValueChange={(value) => setFormData({ ...formData, strategy_type: value as DCAFormData['strategy_type'] })}
                  >
                    <SelectTrigger className="bg-white/10 border-white/20 text-white">
                      <SelectValue placeholder="Select strategy type" />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 border-white/20">
                      <SelectItem value="Simple">Simple</SelectItem>
                      <SelectItem value="VolatilityBased">Volatility Based</SelectItem>
                      <SelectItem value="RSIBased">RSI Based</SelectItem>
                      <SelectItem value="SentimentBased">Sentiment Based</SelectItem>
                      <SelectItem value="Dynamic">Dynamic</SelectItem>
                      <SelectItem value="DipBuying">Dip Buying</SelectItem>
                    </SelectContent>
                  </Select>
                  <div className="text-xs text-white/60">
                    <span className={cn("font-medium", strategyTypeInfo.color)}>
                      {strategyTypeInfo.risk} Risk
                    </span>
                    : {strategyTypeInfo.description}
                  </div>
                </div>

                <div className="space-y-2">
                  <Label className="text-white/80">Frequency</Label>
                  <div className="grid grid-cols-2 gap-2">
                    <Select
                      value={formData.frequency_type}
                      onValueChange={(value) => setFormData({ ...formData, frequency_type: value as DCAFormData['frequency_type'] })}
                    >
                      <SelectTrigger className="bg-white/10 border-white/20 text-white">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-black/90 border-white/20">
                        <SelectItem value="hourly">Hourly</SelectItem>
                        <SelectItem value="daily">Daily</SelectItem>
                        <SelectItem value="weekly">Weekly</SelectItem>
                        <SelectItem value="monthly">Monthly</SelectItem>
                        <SelectItem value="custom">Custom</SelectItem>
                      </SelectContent>
                    </Select>
                    <Input
                      type="number"
                      min="1"
                      value={formData.frequency_value}
                      onChange={(e) => setFormData({
                        ...formData,
                        frequency_value: parseInt(e.target.value) || 1
                      })}
                      className="bg-white/10 border-white/20 text-white"
                    />
                  </div>
                </div>
              </div>
            </div>


            {/* Configuration Tabs */}
            <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-white/5">
                <TabsTrigger value="simple">Simple</TabsTrigger>
                <TabsTrigger value="advanced">Advanced</TabsTrigger>
                <TabsTrigger value="risk">Risk Mgmt</TabsTrigger>
                <TabsTrigger value="filters">Filters</TabsTrigger>
              </TabsList>

              <TabsContent value="simple" className="space-y-4">
                {/* Tab Description */}
                <div className="p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                  <p className="text-sm text-blue-200">
                    <strong>Simple Configuration:</strong> Basic market intelligence features to enhance your DCA strategy with sentiment and volatility adjustments.
                  </p>
                </div>

                {/* Simple Market Intelligence */}
                <div className="space-y-4">
                  <h4 className="text-md font-semibold text-white/90 flex items-center space-x-2">
                    <Clock className="h-4 w-4" />
                    <span>Market Intelligence</span>
                  </h4>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/80">Sentiment Multiplier</Label>
                        <Switch
                          checked={formData.sentiment_multiplier}
                          onCheckedChange={(checked) => setFormData({ ...formData, sentiment_multiplier: checked })}
                        />
                      </div>
                      <p className="text-xs text-white/60">
                        Adjust purchase amounts based on market sentiment (Fear & Greed Index)
                      </p>
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/80">Volatility Adjustment</Label>
                        <Switch
                          checked={formData.volatility_adjustment}
                          onCheckedChange={(checked) => setFormData({ ...formData, volatility_adjustment: checked })}
                        />
                      </div>
                      <p className="text-xs text-white/60">
                        Increase purchases during high volatility periods
                      </p>
                    </div>
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="advanced" className="space-y-6">
                {/* Tab Description */}
                <div className="p-3 bg-purple-500/10 border border-purple-500/20 rounded-lg">
                  <p className="text-sm text-purple-200">
                    <strong>Advanced Configuration:</strong> Advanced options appear based on your selected strategy type. Fine-tune RSI, volatility, sentiment, and dip buying parameters for sophisticated market analysis.
                  </p>
                </div>

                {/* Advanced RSI Configuration */}
                {(formData.strategy_type === 'RSIBased' || formData.strategy_type === 'Dynamic' || formData.enable_rsi) && (
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <h4 className="text-md font-semibold text-white/90">RSI Configuration</h4>
                      <Switch
                        checked={formData.enable_rsi}
                        onCheckedChange={(checked) => setFormData({ ...formData, enable_rsi: checked })}
                      />
                    </div>

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
                {(formData.strategy_type === 'VolatilityBased' || formData.strategy_type === 'Dynamic' || formData.volatility_adjustment) && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90">Volatility Configuration</h4>
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
                {(formData.strategy_type === 'SentimentBased' || formData.strategy_type === 'Dynamic' || formData.sentiment_multiplier) && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90">Sentiment Configuration</h4>
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
                {(formData.strategy_type === 'DipBuying' || formData.strategy_type === 'Dynamic') && (
                  <div className="space-y-4">
                    <h4 className="text-md font-semibold text-white/90">Dip Buying Configuration</h4>
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
                <div className="p-3 bg-green-500/10 border border-green-500/20 rounded-lg">
                  <p className="text-sm text-green-200">
                    <strong>Trading Filters:</strong> Control when your DCA strategy executes with time-based restrictions and execution limits.
                  </p>
                </div>

                {/* Trading Filters */}
                <div className="space-y-4">
                  <h4 className="text-md font-semibold text-white/90">Trading Filters</h4>

                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label className="text-white/80">Max Executions Per Day</Label>
                      <Input
                        type="number"
                        min="1"
                        max="24"
                        value={formData.max_executions_per_day || ''}
                        onChange={(e) => setFormData({
                          ...formData,
                          max_executions_per_day: parseInt(e.target.value) || undefined
                        })}
                        placeholder="3"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                    </div>

                    <div className="text-sm text-white/70">
                      <p>Additional filters like allowed hours and weekdays can be configured here.</p>
                      <p className="text-xs text-white/50 mt-1">
                        These advanced filters help control when the strategy can execute trades.
                      </p>
                    </div>
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

            {/* Strategy Tips */}
            <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
              <h4 className="font-medium text-blue-400 mb-2">💡 DCA Strategy Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• Use Simple strategy for conservative, predictable DCA</li>
                <li>• RSI-based strategy buys more when assets are oversold</li>
                <li>• Volatility-based strategy increases purchases during market turbulence</li>
                <li>• Dynamic strategy combines multiple indicators for optimal timing</li>
                <li>• Always enable risk management for safer long-term investing</li>
              </ul>
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