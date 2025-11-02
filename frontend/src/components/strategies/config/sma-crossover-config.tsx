"use client"

import { useState, useMemo } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
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
  SMACrossoverConfig as SMACrossoverConfigType,
  CreateSMACrossoverStrategyRequest
} from "@/lib/api"
import {
  validateStrategyName,
  validateAssetSymbol
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Activity, LineChart, BarChart3, Clock, Settings, Shield } from "lucide-react"
import { cn } from "@/lib/utils"

interface SMAConfigProps {
  initialData?: Partial<CreateSMACrossoverStrategyRequest>
  onSubmit: (data: CreateSMACrossoverStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (
    config: SMACrossoverConfigType,
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

interface SMAFormData extends CreateSMACrossoverStrategyRequest {
  backtest_start_date?: string
  backtest_end_date?: string
  backtest_interval?: '1m' | '5m' | '15m' | '30m' | '1h' | '4h' | '1d' | '1w'
  backtest_initial_capital?: number
  asset_type?: 'crypto' | 'stock'
}

export function SMAConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: SMAConfigProps) {
  const [formData, setFormData] = useState<SMAFormData>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      fast_period: initialData?.config?.fast_period || 10,
      slow_period: initialData?.config?.slow_period || 20,
      position_size_pct: initialData?.config?.position_size_pct || 10,
      enable_long: initialData?.config?.enable_long ?? true,
      enable_short: initialData?.config?.enable_short ?? false,
      use_market_orders: initialData?.config?.use_market_orders ?? true,
      risk_settings: {
        stop_loss_pct: initialData?.config?.risk_settings?.stop_loss_pct || 2.5,
        take_profit_pct: initialData?.config?.risk_settings?.take_profit_pct || 5.0,
        max_position_pct: initialData?.config?.risk_settings?.max_position_pct || 100,
        min_signal_interval: initialData?.config?.risk_settings?.min_signal_interval || 60,
        trailing_stop: initialData?.config?.risk_settings?.trailing_stop ?? false,
        trailing_stop_pct: initialData?.config?.risk_settings?.trailing_stop_pct || 1.0
      },
      filters: {
        min_volume: initialData?.config?.filters?.min_volume || undefined,
        max_spread_pct: initialData?.config?.filters?.max_spread_pct || undefined,
        rsi_overbought: initialData?.config?.filters?.rsi_overbought || undefined,
        rsi_oversold: initialData?.config?.filters?.rsi_oversold || undefined,
        macd_confirmation: initialData?.config?.filters?.macd_confirmation ?? false,
        min_sma_spread_pct: initialData?.config?.filters?.min_sma_spread_pct || undefined
      },
      confirmation_indicators: {
        use_rsi: initialData?.config?.confirmation_indicators?.use_rsi ?? false,
        rsi_period: initialData?.config?.confirmation_indicators?.rsi_period || 14,
        use_macd: initialData?.config?.confirmation_indicators?.use_macd ?? false,
        macd_fast: initialData?.config?.confirmation_indicators?.macd_fast || 12,
        macd_slow: initialData?.config?.confirmation_indicators?.macd_slow || 26,
        macd_signal: initialData?.config?.confirmation_indicators?.macd_signal || 9,
        use_volume: initialData?.config?.confirmation_indicators?.use_volume ?? false,
        volume_period: initialData?.config?.confirmation_indicators?.volume_period || 20,
        min_volume_multiplier: initialData?.config?.confirmation_indicators?.min_volume_multiplier || 1.5
      }
    },
    // Backtest Configuration
    backtest_start_date: new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    backtest_end_date: new Date().toISOString().split('T')[0],
    backtest_interval: '1d',
    backtest_initial_capital: 10000,
    asset_type: 'crypto'
  })

  const [errors, setErrors] = useState<Record<string, string>>({})
  const [activeTab, setActiveTab] = useState("basic")

  const validateForm = () => {
    const newErrors: Record<string, string> = {}
    
    // Basic validations
    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError
    
    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError
    
    // SMA period validations
    if (formData.config.fast_period <= 0) {
      newErrors.fast_period = 'Fast period must be greater than 0'
    }
    
    if (formData.config.slow_period <= 0) {
      newErrors.slow_period = 'Slow period must be greater than 0'
    }
    
    if (formData.config.fast_period >= formData.config.slow_period) {
      newErrors.periods = 'Fast period must be less than slow period'
    }
    
    if (formData.config.fast_period > 100) {
      newErrors.fast_period = 'Fast period should not exceed 100'
    }
    
    if (formData.config.slow_period > 200) {
      newErrors.slow_period = 'Slow period should not exceed 200'
    }
    
    // Position size validation
    if (formData.config.position_size_pct <= 0 || formData.config.position_size_pct > 100) {
      newErrors.position_size_pct = 'Position size must be between 0.1% and 100%'
    }
    
    // Risk settings validations
    if (formData.config.risk_settings.max_position_pct <= 0 || formData.config.risk_settings.max_position_pct > 100) {
      newErrors.max_position_pct = 'Max position percentage must be between 0.1% and 100%'
    }
    
    if (formData.config.risk_settings.stop_loss_pct < 0 || formData.config.risk_settings.stop_loss_pct > 50) {
      newErrors.stop_loss_pct = 'Stop loss must be between 0% and 50%'
    }
    
    if (formData.config.risk_settings.take_profit_pct < 0) {
      newErrors.take_profit_pct = 'Take profit must be 0% or greater'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) return
    
    const submitData: CreateSMACrossoverStrategyRequest = {
      name: formData.name,
      asset_symbol: formData.asset_symbol,
      config: {
        ...formData.config,
        // Clean up optional fields with undefined values
        risk_settings: {
          ...formData.config.risk_settings,
          trailing_stop_pct: formData.config.risk_settings.trailing_stop ? formData.config.risk_settings.trailing_stop_pct : undefined
        },
        filters: {
          ...formData.config.filters,
          min_volume: formData.config.filters.min_volume || undefined,
          max_spread_pct: formData.config.filters.max_spread_pct || undefined,
          rsi_overbought: formData.config.filters.rsi_overbought || undefined,
          rsi_oversold: formData.config.filters.rsi_oversold || undefined,
          min_sma_spread_pct: formData.config.filters.min_sma_spread_pct || undefined
        }
      }
    }
    
    try {
      await onSubmit(submitData)
    } catch (error) {
      console.error('Failed to create SMA crossover strategy:', error)
    }
  }

  const getSignalStrength = () => {
    const difference = formData.config.slow_period - formData.config.fast_period
    if (difference < 15) return { strength: 'High Frequency', color: 'text-red-400', description: 'More signals, higher noise' }
    if (difference < 30) return { strength: 'Balanced', color: 'text-yellow-400', description: 'Good balance of signals and reliability' }
    return { strength: 'Low Frequency', color: 'text-green-400', description: 'Fewer but more reliable signals' }
  }

  const signalStrength = getSignalStrength()

  const isFormValid = useMemo(() => {
    const nameError = validateStrategyName(formData.name)
    if (nameError) return false

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) return false

    if (formData.config.fast_period <= 0 || formData.config.slow_period <= 0) return false
    if (formData.config.fast_period >= formData.config.slow_period) return false
    if (formData.config.fast_period > 100 || formData.config.slow_period > 200) return false
    if (formData.config.position_size_pct <= 0 || formData.config.position_size_pct > 100) return false

    return true
  }, [formData])

  return (
    <Card className={cn(
      "bg-gradient-to-br from-purple-500/20 to-pink-500/20 backdrop-blur-xl border border-white/10",
      className
    )}>
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
      
      <div className="relative z-10">
        <CardHeader className="text-center space-y-2">
          <div className="text-4xl mx-auto">📈</div>
          <CardTitle className="text-2xl font-bold text-white/90">
            SMA Crossover Strategy Builder
          </CardTitle>
          <CardDescription className="text-white/60">
            Configure every aspect of your moving average crossover strategy
          </CardDescription>
        </CardHeader>

        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-6">
            <Tabs value={activeTab} onValueChange={setActiveTab}>
              <TabsList className="grid w-full grid-cols-5 bg-white/5">
                <TabsTrigger value="basic" className="data-[state=active]:bg-purple-500/20 text-xs">
                  Basic
                </TabsTrigger>
                <TabsTrigger value="risk" className="data-[state=active]:bg-purple-500/20 text-xs">
                  Risk
                </TabsTrigger>
                <TabsTrigger value="filters" className="data-[state=active]:bg-purple-500/20 text-xs">
                  Filters
                </TabsTrigger>
                <TabsTrigger value="indicators" className="data-[state=active]:bg-purple-500/20 text-xs">
                  Indicators
                </TabsTrigger>
                <TabsTrigger value="backtest" className="data-[state=active]:bg-purple-500/20 text-xs">
                  Backtest
                </TabsTrigger>
              </TabsList>

              {/* Basic Configuration */}
              <TabsContent value="basic" className="space-y-6 mt-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                    <Settings className="h-5 w-5" />
                    <span>Basic Configuration</span>
                  </h3>

                    {/* Strategy Name and Asset */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="name" className="text-white/90 font-medium">Strategy Name</Label>
                      <Input
                        id="name"
                        value={formData.name}
                        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        placeholder="My SMA Crossover Strategy"
                        className="bg-white/10 border-white/30 text-white placeholder:text-white/60 focus:border-white/50"
                      />
                      {errors.name && (
                        <p className="text-red-400 text-sm flex items-center space-x-1">
                          <AlertTriangle className="h-4 w-4" />
                          <span>{errors.name}</span>
                        </p>
                      )}
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="asset_type" className="text-white/90 font-medium">Asset Type</Label>
                      <Select
                        value={formData.asset_type || 'crypto'}
                        onValueChange={(value: 'crypto' | 'stock') => setFormData({ ...formData, asset_type: value })}
                      >
                        <SelectTrigger className="bg-white/10 border-white/30 text-white hover:bg-white/15 transition-colors">
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
                      <Label htmlFor="asset_symbol" className="text-white/90 font-medium">Asset Symbol</Label>
                      <Input
                        id="asset_symbol"
                        value={formData.asset_symbol}
                        onChange={(e) => setFormData({ ...formData, asset_symbol: e.target.value.toUpperCase() })}
                        placeholder={formData.asset_type === 'stock' ? 'AAPL' : 'BTC'}
                        className="bg-white/10 border-white/30 text-white placeholder:text-white/60 focus:border-white/50"
                      />
                      {errors.asset_symbol && (
                        <p className="text-red-400 text-sm flex items-center space-x-1">
                          <AlertTriangle className="h-4 w-4" />
                          <span>{errors.asset_symbol}</span>
                        </p>
                      )}
                    </div>
                  </div>

                  {/* SMA Periods */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/90 font-medium">Fast Period</Label>
                        <span className="text-purple-300 font-mono bg-purple-500/20 px-2 py-1 rounded">{formData.config.fast_period}</span>
                      </div>
                      <Input
                        type="number"
                        min="1"
                        max="100"
                        value={formData.config.fast_period || ''}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { ...formData.config, fast_period: parseInt(e.target.value) || 0 }
                        })}
                        placeholder="Enter fast period (e.g., 10)"
                        className="bg-white/10 border-white/30 text-white placeholder:text-white/60 focus:border-white/50"
                      />
                      <p className="text-xs text-white/70">Shorter period = more responsive to price changes</p>
                      {errors.fast_period && (
                        <p className="text-red-400 text-sm">{errors.fast_period}</p>
                      )}
                    </div>

                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-white/90 font-medium">Slow Period</Label>
                        <span className="text-pink-300 font-mono bg-pink-500/20 px-2 py-1 rounded">{formData.config.slow_period}</span>
                      </div>
                      <Input
                        type="number"
                        min="2"
                        max="200"
                        value={formData.config.slow_period || ''}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { ...formData.config, slow_period: parseInt(e.target.value) || 0 }
                        })}
                        placeholder="Enter slow period (e.g., 20)"
                        className="bg-white/10 border-white/30 text-white placeholder:text-white/60 focus:border-white/50"
                      />
                      <p className="text-xs text-white/70">Longer period = smoother trend signals</p>
                      {errors.slow_period && (
                        <p className="text-red-400 text-sm">{errors.slow_period}</p>
                      )}
                    </div>
                  </div>

                  {/* Position Size */}
                  <div className="space-y-3">
                    <Label className="text-white/90 font-medium">Position Size (%)</Label>
                    <Input
                      type="number"
                      min="0.1"
                      max="100"
                      step="0.1"
                      value={formData.config.position_size_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { ...formData.config, position_size_pct: parseFloat(e.target.value) || 0 }
                      })}
                      placeholder="Enter position size percentage (e.g., 10)"
                      className="bg-white/10 border-white/30 text-white placeholder:text-white/60 focus:border-white/50"
                    />
                    <p className="text-xs text-white/70">Percentage of available balance to use per trade</p>
                    {errors.position_size_pct && (
                      <p className="text-red-400 text-sm">{errors.position_size_pct}</p>
                    )}
                  </div>

                  {/* Trading Direction */}
                  <div className="space-y-3">
                    <Label className="text-white/90 font-medium">Trading Direction</Label>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                        <Switch
                          checked={formData.config.enable_long}
                          onCheckedChange={(checked) => setFormData({ 
                            ...formData, 
                            config: { ...formData.config, enable_long: checked }
                          })}
                          className="data-[state=checked]:bg-green-500 data-[state=unchecked]:bg-gray-600"
                        />
                        <div>
                          <span className="text-white font-medium">Long Positions</span>
                          <p className="text-xs text-white/60">Buy when fast {">"} slow SMA</p>
                        </div>
                      </div>
                      <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                        <Switch
                          checked={formData.config.enable_short}
                          onCheckedChange={(checked) => setFormData({ 
                            ...formData, 
                            config: { ...formData.config, enable_short: checked }
                          })}
                          className="data-[state=checked]:bg-red-500 data-[state=unchecked]:bg-gray-600"
                        />
                        <div>
                          <span className="text-white font-medium">Short Positions</span>
                          <p className="text-xs text-white/60">Sell when fast {"<"} slow SMA</p>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Order Type */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.use_market_orders}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { ...formData.config, use_market_orders: checked }
                        })}
                        className="data-[state=checked]:bg-blue-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Use Market Orders</Label>
                        <p className="text-xs text-white/60">
                          Market orders execute immediately at current price. Limit orders may get better prices but might not fill.
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Signal Analysis */}
                  {formData.config.fast_period > 0 && formData.config.slow_period > 0 && (
                    <div className="bg-white/10 border border-white/20 rounded-lg p-4 space-y-3">
                      <h4 className="font-medium text-white/90 flex items-center space-x-2">
                        <Activity className="h-4 w-4" />
                        <span>Signal Analysis</span>
                      </h4>
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div className="bg-white/5 rounded p-2">
                          <span className="text-white/80">Period Difference:</span>
                          <div className="text-white font-mono text-lg">{formData.config.slow_period - formData.config.fast_period}</div>
                        </div>
                        <div className="bg-white/5 rounded p-2">
                          <span className="text-white/80">Signal Frequency:</span>
                          <div className={cn("font-semibold text-lg", signalStrength.color)}>{signalStrength.strength}</div>
                        </div>
                      </div>
                      <p className="text-sm text-white/80 bg-white/5 rounded p-2">{signalStrength.description}</p>
                    </div>
                  )}

                  {errors.periods && (
                    <p className="text-red-400 text-sm flex items-center space-x-1">
                      <AlertTriangle className="h-4 w-4" />
                      <span>{errors.periods}</span>
                    </p>
                  )}
                </div>
              </TabsContent>

              {/* Risk Management */}
              <TabsContent value="risk" className="space-y-6 mt-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                    <Shield className="h-5 w-5" />
                    <span>Risk Management</span>
                  </h3>

                  {/* Stop Loss */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Stop Loss (%)</Label>
                    <Input
                      type="number"
                      min="0"
                      max="50"
                      step="0.1"
                      value={formData.config.risk_settings.stop_loss_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          risk_settings: { 
                            ...formData.config.risk_settings, 
                            stop_loss_pct: parseFloat(e.target.value) || 0 
                          }
                        }
                      })}
                      placeholder="2.5"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Maximum loss before position is closed (0.1 - 50%).</p>
                    {errors.stop_loss_pct && (
                      <p className="text-red-400 text-sm">{errors.stop_loss_pct}</p>
                    )}
                  </div>

                  {/* Take Profit */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Take Profit (%)</Label>
                    <Input
                      type="number"
                      min="0"
                      step="0.1"
                      value={formData.config.risk_settings.take_profit_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          risk_settings: { 
                            ...formData.config.risk_settings, 
                            take_profit_pct: parseFloat(e.target.value) || 0 
                          }
                        }
                      })}
                      placeholder="5.0"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Profit target before position is closed (min 0.1%).</p>
                    {errors.take_profit_pct && (
                      <p className="text-red-400 text-sm">{errors.take_profit_pct}</p>
                    )}
                  </div>

                  {/* Max Position Size */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Max Position Size (%)</Label>
                    <Input
                      type="number"
                      min="0.1"
                      max="100"
                      step="0.1"
                      value={formData.config.risk_settings.max_position_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          risk_settings: { 
                            ...formData.config.risk_settings, 
                            max_position_pct: parseFloat(e.target.value) || 100 
                          }
                        }
                      })}
                      placeholder="100"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Maximum % of portfolio for this strategy (0.1 - 100%).</p>
                    {errors.max_position_pct && (
                      <p className="text-red-400 text-sm">{errors.max_position_pct}</p>
                    )}
                  </div>

                  {/* Signal Interval */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Min Signal Interval (minutes)</Label>
                    <Input
                      type="number"
                      min="0"
                      value={formData.config.risk_settings.min_signal_interval || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          risk_settings: { 
                            ...formData.config.risk_settings, 
                            min_signal_interval: parseInt(e.target.value) || 0 
                          }
                        }
                      })}
                      placeholder="60"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Minimum minutes between signals (0 = no limit).</p>
                  </div>

                  {/* Trailing Stop */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.risk_settings.trailing_stop}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            risk_settings: { 
                              ...formData.config.risk_settings, 
                              trailing_stop: checked 
                            }
                          }
                        })}
                        className="data-[state=checked]:bg-purple-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Enable Trailing Stop</Label>
                        <p className="text-xs text-white/60">Dynamically adjust stop loss as price moves favorably</p>
                      </div>
                    </div>
                    {formData.config.risk_settings.trailing_stop && (
                      <div className="space-y-2">
                        <Input
                          type="number"
                          min="0.1"
                          max="10"
                          step="0.1"
                          value={formData.config.risk_settings.trailing_stop_pct || ''}
                          onChange={(e) => setFormData({ 
                            ...formData, 
                            config: { 
                              ...formData.config, 
                              risk_settings: { 
                                ...formData.config.risk_settings, 
                                trailing_stop_pct: parseFloat(e.target.value) || undefined 
                              }
                            }
                          })}
                          placeholder="1.0"
                          className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                        />
                        <p className="text-xs text-white/60">Distance below peak to trigger stop (0.1 - 10%).</p>
                      </div>
                    )}
                  </div>
                </div>
              </TabsContent>

              {/* Filters */}
              <TabsContent value="filters" className="space-y-6 mt-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                    <Activity className="h-5 w-5" />
                    <span>Signal Filters</span>
                  </h3>

                  {/* Volume Filter */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Minimum Volume</Label>
                    <Input
                      type="number"
                      min="0"
                      value={formData.config.filters.min_volume || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          filters: { 
                            ...formData.config.filters, 
                            min_volume: parseFloat(e.target.value) || undefined 
                          }
                        }
                      })}
                      placeholder="Optional"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Only trade when volume exceeds threshold (optional).</p>
                  </div>

                  {/* Spread Filter */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Max Spread (%)</Label>
                    <Input
                      type="number"
                      min="0"
                      max="10"
                      step="0.01"
                      value={formData.config.filters.max_spread_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          filters: { 
                            ...formData.config.filters, 
                            max_spread_pct: parseFloat(e.target.value) || undefined 
                          }
                        }
                      })}
                      placeholder="0.5"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Skip trades when spread exceeds this % (optional).</p>
                  </div>

                  {/* RSI Filters */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-3">
                      <Label className="text-white/80">RSI Overbought Level</Label>
                      <Input
                        type="number"
                        min="50"
                        max="100"
                        value={formData.config.filters.rsi_overbought || ''}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            filters: { 
                              ...formData.config.filters, 
                              rsi_overbought: parseFloat(e.target.value) || undefined 
                            }
                          }
                        })}
                        placeholder="70"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">Skip buys when RSI above this (50-100, optional).</p>
                    </div>

                    <div className="space-y-3">
                      <Label className="text-white/80">RSI Oversold Level</Label>
                      <Input
                        type="number"
                        min="0"
                        max="50"
                        value={formData.config.filters.rsi_oversold || ''}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            filters: { 
                              ...formData.config.filters, 
                              rsi_oversold: parseFloat(e.target.value) || undefined 
                            }
                          }
                        })}
                        placeholder="30"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">Skip sells when RSI below this (0-50, optional).</p>
                    </div>
                  </div>

                  {/* MACD Confirmation */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.filters.macd_confirmation}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            filters: { 
                              ...formData.config.filters, 
                              macd_confirmation: checked 
                            }
                          }
                        })}
                        className="data-[state=checked]:bg-indigo-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Require MACD Confirmation</Label>
                        <p className="text-xs text-white/60">Only trade when MACD also indicates the same direction.</p>
                      </div>
                    </div>
                  </div>

                  {/* SMA Spread Filter */}
                  <div className="space-y-3">
                    <Label className="text-white/80">Min SMA Spread (%)</Label>
                    <Input
                      type="number"
                      min="0"
                      max="10"
                      step="0.01"
                      value={formData.config.filters.min_sma_spread_pct || ''}
                      onChange={(e) => setFormData({ 
                        ...formData, 
                        config: { 
                          ...formData.config, 
                          filters: { 
                            ...formData.config.filters, 
                            min_sma_spread_pct: parseFloat(e.target.value) || undefined 
                          }
                        }
                      })}
                      placeholder="0.2"
                      className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                    />
                    <p className="text-xs text-white/60">Only trade when SMAs separated by this % (optional).</p>
                  </div>
                </div>
              </TabsContent>

              {/* Confirmation Indicators */}
              <TabsContent value="indicators" className="space-y-6 mt-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                    <LineChart className="h-5 w-5" />
                    <span>Confirmation Indicators</span>
                  </h3>

                  {/* RSI Confirmation */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.confirmation_indicators.use_rsi}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            confirmation_indicators: { 
                              ...formData.config.confirmation_indicators, 
                              use_rsi: checked 
                            }
                          }
                        })}
                        className="data-[state=checked]:bg-orange-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Use RSI Confirmation</Label>
                        <p className="text-xs text-white/60">Add RSI momentum confirmation to signals</p>
                      </div>
                    </div>
                    {formData.config.confirmation_indicators.use_rsi && (
                      <div className="space-y-2">
                        <Input
                          type="number"
                          min="2"
                          max="100"
                          value={formData.config.confirmation_indicators.rsi_period || ''}
                          onChange={(e) => setFormData({ 
                            ...formData, 
                            config: { 
                              ...formData.config, 
                              confirmation_indicators: { 
                                ...formData.config.confirmation_indicators, 
                                rsi_period: parseInt(e.target.value) || 14 
                              }
                            }
                          })}
                          placeholder="14"
                          className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                        />
                        <p className="text-xs text-white/60">RSI calculation period (2-100).</p>
                      </div>
                    )}
                  </div>

                  {/* MACD Confirmation */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.confirmation_indicators.use_macd}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            confirmation_indicators: { 
                              ...formData.config.confirmation_indicators, 
                              use_macd: checked 
                            }
                          }
                        })}
                        className="data-[state=checked]:bg-cyan-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Use MACD Confirmation</Label>
                        <p className="text-xs text-white/60">Add MACD convergence/divergence confirmation</p>
                      </div>
                    </div>
                    {formData.config.confirmation_indicators.use_macd && (
                      <div className="grid grid-cols-3 gap-2">
                        <div>
                          <Input
                            type="number"
                            min="2"
                            max="50"
                            value={formData.config.confirmation_indicators.macd_fast || ''}
                            onChange={(e) => setFormData({ 
                              ...formData, 
                              config: { 
                                ...formData.config, 
                                confirmation_indicators: { 
                                  ...formData.config.confirmation_indicators, 
                                  macd_fast: parseInt(e.target.value) || 12 
                                }
                              }
                            })}
                            placeholder="12"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                        </div>
                        <div>
                          <Input
                            type="number"
                            min="5"
                            max="100"
                            value={formData.config.confirmation_indicators.macd_slow || ''}
                            onChange={(e) => setFormData({
                              ...formData,
                              config: {
                                ...formData.config,
                                confirmation_indicators: {
                                  ...formData.config.confirmation_indicators,
                                  macd_slow: parseInt(e.target.value) || 26
                                }
                              }
                            })}
                            placeholder="26"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                        </div>
                        <div>
                          <Input
                            type="number"
                            min="2"
                            max="50"
                            value={formData.config.confirmation_indicators.macd_signal || ''}
                            onChange={(e) => setFormData({
                              ...formData,
                              config: {
                                ...formData.config,
                                confirmation_indicators: {
                                  ...formData.config.confirmation_indicators,
                                  macd_signal: parseInt(e.target.value) || 9
                                }
                              }
                            })}
                            placeholder="9"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Volume Confirmation */}
                  <div className="space-y-3">
                    <div className="flex items-center space-x-3 p-3 bg-white/10 rounded-lg border border-white/20">
                      <Switch
                        checked={formData.config.confirmation_indicators.use_volume}
                        onCheckedChange={(checked) => setFormData({ 
                          ...formData, 
                          config: { 
                            ...formData.config, 
                            confirmation_indicators: { 
                              ...formData.config.confirmation_indicators, 
                              use_volume: checked 
                            }
                          }
                        })}
                        className="data-[state=checked]:bg-teal-500 data-[state=unchecked]:bg-gray-600"
                      />
                      <div>
                        <Label className="text-white font-medium">Use Volume Confirmation</Label>
                        <p className="text-xs text-white/60">Require volume spike to confirm signals</p>
                      </div>
                    </div>
                    {formData.config.confirmation_indicators.use_volume && (
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <Input
                            type="number"
                            min="5"
                            max="100"
                            value={formData.config.confirmation_indicators.volume_period || ''}
                            onChange={(e) => setFormData({ 
                              ...formData, 
                              config: { 
                                ...formData.config, 
                                confirmation_indicators: { 
                                  ...formData.config.confirmation_indicators, 
                                  volume_period: parseInt(e.target.value) || 20 
                                }
                              }
                            })}
                            placeholder="20"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                        </div>
                        <div>
                          <Input
                            type="number"
                            min="1"
                            max="10"
                            step="0.1"
                            value={formData.config.confirmation_indicators.min_volume_multiplier || ''}
                            onChange={(e) => setFormData({
                              ...formData,
                              config: {
                                ...formData.config,
                                confirmation_indicators: {
                                  ...formData.config.confirmation_indicators,
                                  min_volume_multiplier: parseFloat(e.target.value) || 1.5
                                }
                              }
                            })}
                            placeholder="1.5"
                            className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                          />
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="backtest" className="space-y-6 mt-6">
                {/* Backtest Configuration */}
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                      <Clock className="h-5 w-5" />
                      <span>Backtest Configuration</span>
                    </h3>
                  </div>

                  {/* Quick date range presets */}
                  <div className="flex flex-wrap gap-2">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const today = new Date().toISOString().split('T')[0]
                        const start = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                        setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                      }}
                      className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                    >
                      1 Month
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const today = new Date().toISOString().split('T')[0]
                        const start = new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
                        setFormData({ ...formData, backtest_start_date: start, backtest_end_date: today })
                      }}
                      className="bg-white/5 border-white/20 text-white/80 hover:bg-white/10 text-xs"
                    >
                      3 Months
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
                    </div>

                    {/* Interval Selection */}
                    <div className="space-y-2">
                      <Label className="text-white/80">Candlestick Interval</Label>
                      <Select
                        value={formData.backtest_interval}
                        onValueChange={(value) => setFormData({ ...formData, backtest_interval: value as SMAFormData['backtest_interval'] })}
                      >
                        <SelectTrigger className="bg-white/10 border-white/20 text-white hover:bg-white/20">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="bg-slate-900 border-white/30 backdrop-blur-xl">
                          <SelectItem value="1m" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">1 Minute</SelectItem>
                          <SelectItem value="5m" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">5 Minutes</SelectItem>
                          <SelectItem value="15m" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">15 Minutes</SelectItem>
                          <SelectItem value="30m" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">30 Minutes</SelectItem>
                          <SelectItem value="1h" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">1 Hour</SelectItem>
                          <SelectItem value="4h" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">4 Hours</SelectItem>
                          <SelectItem value="1d" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">1 Day</SelectItem>
                          <SelectItem value="1w" className="text-white hover:bg-purple-500/20 focus:bg-purple-500/20 cursor-pointer">1 Week</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    {/* Initial Capital */}
                    <div className="space-y-2">
                      <Label className="text-white/80">Initial Capital ($)</Label>
                      <Input
                        type="number"
                        value={formData.backtest_initial_capital}
                        onChange={(e) => setFormData({ ...formData, backtest_initial_capital: parseFloat(e.target.value) })}
                        min="100"
                        step="100"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                    </div>
                  </div>

                  {/* Summary Box */}
                  <div className="bg-white/5 border border-white/10 rounded-lg p-4">
                    <h4 className="text-sm font-medium text-white/90 mb-3">Backtest Summary</h4>
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
                    </div>
                  </div>
                </div>
              </TabsContent>
            </Tabs>

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
                  onClick={() => {
                    const backtestParams = {
                      start_date: formData.backtest_start_date || new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
                      end_date: formData.backtest_end_date || new Date().toISOString().split('T')[0],
                      interval: formData.backtest_interval || '1d',
                      initial_capital: formData.backtest_initial_capital || 10000,
                      asset_type: formData.asset_type || 'crypto'
                    }
                    onBacktest(formData.config, formData.name, formData.asset_symbol, backtestParams)
                  }}
                  className="border-blue-500/30 text-blue-400 hover:bg-blue-500/10"
                  disabled={isLoading || !isFormValid}
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Backtest First
                </Button>
              )}
              
              <Button
                type="submit"
                className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white flex-1"
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
                    Create SMA Strategy
                  </>
                )}
              </Button>
            </div>

            {/* Strategy Tips */}
            <div className="bg-purple-500/10 border border-purple-500/20 rounded-lg p-4 mt-6">
              <h4 className="font-medium text-purple-400 mb-2">💡 SMA Crossover Strategy Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• Works best in trending markets, less effective in sideways markets</li>
                <li>• Popular combinations: 10/20, 20/50, 50/200</li>
                <li>• Shorter periods = more signals but higher noise</li>
                <li>• Use filters and confirmations to reduce false signals</li>
                <li>• Consider volume and RSI for additional confirmation</li>
                <li>• Backtest thoroughly before deploying with real funds</li>
              </ul>
            </div>
          </CardContent>
        </form>
      </div>
    </Card>
  )
}
