"use client"

import { useState, useMemo, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import {
  type GridTradingConfig,
  CreateGridTradingStrategyRequest
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, Target, BarChart3, Calendar } from "lucide-react"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

interface GridTradingConfigProps {
  initialData?: Partial<CreateGridTradingStrategyRequest>
  onSubmit: (data: CreateGridTradingStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (
    config: GridTradingConfig,
    name: string,
    assetSymbol: string,
    backtestParams: {
      start_date: string
      end_date: string
      interval: string
      initial_capital: number
    }
  ) => void
  isLoading?: boolean
  className?: string
}

export function GridTradingConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: GridTradingConfigProps) {
  const [formData, setFormData] = useState<CreateGridTradingStrategyRequest & {
    backtest_start_date?: string
    backtest_end_date?: string
    backtest_interval?: '1m' | '5m' | '15m' | '30m' | '1h' | '4h' | '1d' | '1w'
    backtest_initial_capital?: number
  }>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      grid_count: initialData?.config?.grid_count || 10,
      range_percentage: initialData?.config?.range_percentage || '10',
      investment_amount: initialData?.config?.investment_amount || '1000',
      stop_loss_percentage: initialData?.config?.stop_loss_percentage || '5',
      take_profit_percentage: initialData?.config?.take_profit_percentage || '15',
      rebalance_threshold: initialData?.config?.rebalance_threshold || '2'
    },
    // Backtest Configuration - Default to 11 months
    backtest_start_date: new Date(Date.now() - 335 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
    backtest_end_date: new Date().toISOString().split('T')[0],
    backtest_interval: '1d',
    backtest_initial_capital: 10000
  })

  const [errors, setErrors] = useState<Record<string, string>>({})
  const [enableStopLoss, setEnableStopLoss] = useState(!!initialData?.config?.stop_loss_percentage)
  const [enableTakeProfit, setEnableTakeProfit] = useState(!!initialData?.config?.take_profit_percentage)
  const [currentPrice, setCurrentPrice] = useState<number | null>(null)
  const [priceLoading, setPriceLoading] = useState(false)

  // Fetch current price when asset symbol changes
  useEffect(() => {
    const fetchPrice = async () => {
      if (!formData.asset_symbol) {
        setCurrentPrice(null)
        return
      }

      setPriceLoading(true)
      try {
        const response = await fetch(`http://localhost:8080/api/v1/market-data/${formData.asset_symbol}/current`)
        if (response.ok) {
          const data = await response.json()
          setCurrentPrice(data.price)
        } else {
          setCurrentPrice(null)
        }
      } catch (error) {
        console.error('Failed to fetch current price:', error)
        setCurrentPrice(null)
      } finally {
        setPriceLoading(false)
      }
    }

    const debounce = setTimeout(fetchPrice, 500)
    return () => clearTimeout(debounce)
  }, [formData.asset_symbol])

  const validateForm = () => {
    const newErrors: Record<string, string> = {}

    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError

    const amountError = validateInvestmentAmount(formData.config.investment_amount)
    if (amountError) newErrors.investment_amount = amountError

    // Grid-specific validations
    const rangePercentage = parseFloat(formData.config.range_percentage || '0')
    if (!formData.config.range_percentage || rangePercentage <= 0) {
      newErrors.range_percentage = 'Range percentage must be greater than 0'
    }
    if (rangePercentage > 50) {
      newErrors.range_percentage = 'Range percentage should not exceed 50% for safety'
    }

    if (formData.config.grid_count < 3 || formData.config.grid_count > 50) {
      newErrors.grid_count = 'Grid count must be between 3 and 50'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!validateForm()) return

    const investmentAmount = parseFloat(formData.config.investment_amount)
    const gridCount = formData.config.grid_count
    const rangePercentage = parseFloat(formData.config.range_percentage)

    // Use percentage-based bounds that will center around current market price
    // This is the proper way to set up grid trading!

    // Calculate spacing: total range / (levels - 1)
    // Total range is 2x rangePercentage (±X% means X% up + X% down)
    const totalRangePercentage = rangePercentage * 2
    const spacingPercentage = totalRangePercentage / (gridCount - 1)

    const backendConfig = {
      grid_levels: gridCount,
      total_investment: investmentAmount,
      spacing: {
        mode: "Standard",
        fixed_spacing: spacingPercentage,
        arithmetic_increment: null,
        geometric_multiplier: null,
        dynamic_base_pct: null,
        volatility_factor: null
      },
      bounds: {
        // Use percentage-based bounds - grid will center around current price
        upper_bound: rangePercentage,
        lower_bound: rangePercentage,
        bounds_type: "PercentageFromCenter",
        auto_adjust: true,
        use_support_resistance: false
      },
      risk_settings: {
        max_inventory: investmentAmount * 0.5, // 50% of investment for inventory
        stop_loss_pct: enableStopLoss ? parseFloat(formData.config.stop_loss_percentage!) : null,
        take_profit_pct: enableTakeProfit ? parseFloat(formData.config.take_profit_percentage!) : null,
        max_drawdown_pct: enableStopLoss ? parseFloat(formData.config.stop_loss_percentage!) : 15,
        max_time_in_position: null,
        dynamic_adjustment: true,
        volatility_pause_threshold: null
      },
      min_order_size: 10,
      max_order_size: null,
      enable_rebalancing: true,
      rebalancing_interval: null, // Only rebalance when price exits bounds
      take_profit_threshold: enableTakeProfit ? parseFloat(formData.config.take_profit_percentage!) : null,
      stop_loss_threshold: enableStopLoss ? parseFloat(formData.config.stop_loss_percentage!) : null,
      market_making: {
        enabled: false,
        spread_pct: 0.2,
        inventory_target: 0,
        max_inventory_deviation: investmentAmount * 0.3,
        inventory_adjustment: true,
        inventory_skew_factor: 0.1
      }
    }

    const submitData: CreateGridTradingStrategyRequest = {
      name: formData.name,
      asset_symbol: formData.asset_symbol,
      config: backendConfig as unknown as GridTradingConfig
    }

    try {
      await onSubmit(submitData)
    } catch (error) {
      console.error('Failed to create grid trading strategy:', error)
    }
  }

  const calculateGridBounds = () => {
    if (!currentPrice || !formData.config.range_percentage) {
      return { upperBound: null, lowerBound: null, spacing: null }
    }

    const rangePercentage = parseFloat(formData.config.range_percentage) / 100
    const upperBound = currentPrice * (1 + rangePercentage)
    const lowerBound = currentPrice * (1 - rangePercentage)
    const range = upperBound - lowerBound
    const spacing = range / (formData.config.grid_count - 1)

    return { upperBound, lowerBound, spacing }
  }

  const { upperBound, lowerBound, spacing } = calculateGridBounds()
  const investmentPerGrid = parseFloat(formData.config.investment_amount) / formData.config.grid_count

  const isFormValid = useMemo(() => {
    const nameError = validateStrategyName(formData.name)
    if (nameError) return false

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) return false

    const amountError = validateInvestmentAmount(formData.config.investment_amount)
    if (amountError) return false

    const rangePercentage = parseFloat(formData.config.range_percentage || '0')
    if (!formData.config.range_percentage || rangePercentage <= 0) return false
    if (rangePercentage > 50) return false

    if (formData.config.grid_count < 3 || formData.config.grid_count > 50) return false

    return true
  }, [formData])

  return (
    <Card className={cn(
      "bg-gradient-to-br from-green-500/20 to-emerald-500/20 backdrop-blur-xl border border-white/10",
      className
    )}>
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
      
      <div className="relative z-10">
        <CardHeader className="text-center space-y-2">
          <div className="text-4xl mx-auto">🎯</div>
          <CardTitle className="text-2xl font-bold text-white/90">
            Grid Trading Strategy
          </CardTitle>
          <CardDescription className="text-white/60">
            Set up automated buy and sell orders in a grid pattern to profit from market volatility
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
                  placeholder="My Grid Trading Strategy"
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

            {/* Grid Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Target className="h-5 w-5" />
                <span>Grid Configuration</span>
              </h3>

              {/* Current Price Display */}
              {currentPrice && (
                <div className="bg-blue-500/20 border border-blue-500/30 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <span className="text-white/80">Current {formData.asset_symbol} Price:</span>
                    <span className="text-2xl font-bold text-white">${currentPrice.toLocaleString()}</span>
                  </div>
                  <p className="text-xs text-white/60 mt-2">
                    Grid will be centered around this price
                  </p>
                </div>
              )}

              {priceLoading && (
                <div className="bg-white/5 rounded-lg p-4 text-center text-white/60">
                  Loading current price...
                </div>
              )}

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="range_percentage" className="text-white/80">Grid Range (%)</Label>
                  <Input
                    id="range_percentage"
                    type="number"
                    step="0.5"
                    value={formData.config.range_percentage}
                    onChange={(e) => setFormData({
                      ...formData,
                      config: { ...formData.config, range_percentage: e.target.value }
                    })}
                    placeholder="10"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.range_percentage && (
                    <p className="text-red-400 text-sm">{errors.range_percentage}</p>
                  )}
                  <p className="text-xs text-white/60">
                    Grid will span ±{formData.config.range_percentage || 0}% from current price
                  </p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="investment_amount" className="text-white/80">Total Investment ($)</Label>
                  <Input
                    id="investment_amount"
                    type="number"
                    step="0.01"
                    value={formData.config.investment_amount}
                    onChange={(e) => setFormData({
                      ...formData,
                      config: { ...formData.config, investment_amount: e.target.value }
                    })}
                    placeholder="1000"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.investment_amount && (
                    <p className="text-red-400 text-sm">{errors.investment_amount}</p>
                  )}
                </div>
              </div>

              {/* Grid Count Slider */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label className="text-white/80">Grid Count</Label>
                  <span className="text-white/60 font-mono">{formData.config.grid_count} grids</span>
                </div>
                <Slider
                  value={[formData.config.grid_count]}
                  onValueChange={([value]) => setFormData({ 
                    ...formData, 
                    config: { ...formData.config, grid_count: value }
                  })}
                  min={3}
                  max={50}
                  step={1}
                  className="w-full"
                />
                <div className="text-sm text-white/60">
                  More grids = smaller spacing between orders, potentially more frequent trades
                </div>
              </div>

              {/* Grid Analysis */}
              {currentPrice && upperBound && lowerBound && spacing && (
                <div className="bg-white/5 rounded-lg p-4 space-y-3">
                  <h4 className="font-medium text-white/80">Grid Preview</h4>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-white/60">Upper Bound: </span>
                      <span className="text-white/90 font-mono">${upperBound.toFixed(2)}</span>
                    </div>
                    <div>
                      <span className="text-white/60">Lower Bound: </span>
                      <span className="text-white/90 font-mono">${lowerBound.toFixed(2)}</span>
                    </div>
                    <div>
                      <span className="text-white/60">Grid Spacing: </span>
                      <span className="text-white/90 font-mono">${spacing.toFixed(2)}</span>
                    </div>
                    <div>
                      <span className="text-white/60">Investment per Grid: </span>
                      <span className="text-white/90 font-mono">${investmentPerGrid.toFixed(2)}</span>
                    </div>
                  </div>
                  <div className="pt-2 border-t border-white/10">
                    <p className="text-xs text-white/60">
                      Buy orders will be placed below ${currentPrice.toFixed(2)}, sell orders above
                    </p>
                  </div>
                </div>
              )}
            </div>

            {/* Backtest Configuration */}
            {onBacktest && (
              <div className="space-y-4">
                <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                  <Calendar className="h-5 w-5" />
                  <span>Backtest Configuration</span>
                </h3>

                <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
                  <p className="text-sm text-blue-300 mb-4">
                    Configure the historical period and parameters for backtesting this strategy
                  </p>

                  <div className="flex gap-2 mb-4">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
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
                      size="sm"
                      variant="outline"
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
                      size="sm"
                      variant="outline"
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

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                    <div className="space-y-2">
                      <Label className="text-white/80">Start Date</Label>
                      <Input
                        type="date"
                        value={formData.backtest_start_date}
                        onChange={(e) => setFormData({ ...formData, backtest_start_date: e.target.value })}
                        className="bg-white/10 border-white/20 text-white"
                      />
                      <p className="text-xs text-white/60">Beginning of backtest period</p>
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
                      <p className="text-xs text-white/60">End of backtest period (today or earlier)</p>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label className="text-white/80">Candlestick Interval</Label>
                      <Select
                        value={formData.backtest_interval}
                        onValueChange={(value) => setFormData({ ...formData, backtest_interval: value as typeof formData.backtest_interval })}
                      >
                        <SelectTrigger className="bg-white/10 border-white/20 text-white hover:bg-white/20">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="bg-gray-900 border-white/20">
                          <SelectItem value="1m">1 minute</SelectItem>
                          <SelectItem value="5m">5 minutes</SelectItem>
                          <SelectItem value="15m">15 minutes</SelectItem>
                          <SelectItem value="30m">30 minutes</SelectItem>
                          <SelectItem value="1h">1 hour</SelectItem>
                          <SelectItem value="4h">4 hours</SelectItem>
                          <SelectItem value="1d">1 day</SelectItem>
                          <SelectItem value="1w">1 week</SelectItem>
                        </SelectContent>
                      </Select>
                      <p className="text-xs text-white/60">How often to check for grid fills</p>
                    </div>

                    <div className="space-y-2">
                      <Label className="text-white/80">Initial Capital ($)</Label>
                      <Input
                        type="number"
                        value={formData.backtest_initial_capital}
                        onChange={(e) => setFormData({ ...formData, backtest_initial_capital: parseFloat(e.target.value) })}
                        className="bg-white/10 border-white/20 text-white"
                      />
                      <p className="text-xs text-white/60">Starting balance for backtest</p>
                    </div>
                  </div>

                  <div className="mt-4 p-3 bg-white/5 rounded">
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div className="text-white/70">Duration:</div>
                      <div className="text-white font-semibold">
                        {formData.backtest_start_date && formData.backtest_end_date
                          ? (() => {
                              const days = Math.ceil((new Date(formData.backtest_end_date).getTime() - new Date(formData.backtest_start_date).getTime()) / (1000 * 60 * 60 * 24))
                              return days > 365 ? `${days} days (~${(days / 365).toFixed(1)} years)` : `${days} days`
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
              </div>
            )}

            {/* Risk Management */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Zap className="h-5 w-5" />
                <span>Risk Management</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Stop Loss */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Stop Loss</Label>
                    <Switch
                      checked={enableStopLoss}
                      onCheckedChange={setEnableStopLoss}
                    />
                  </div>
                  {enableStopLoss && (
                    <div className="space-y-2">
                      <Input
                        type="number"
                        step="0.1"
                        value={formData.config.stop_loss_percentage}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { ...formData.config, stop_loss_percentage: e.target.value }
                        })}
                        placeholder="5"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">
                        Exit all positions if total loss exceeds this percentage
                      </p>
                    </div>
                  )}
                </div>

                {/* Take Profit */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Take Profit</Label>
                    <Switch
                      checked={enableTakeProfit}
                      onCheckedChange={setEnableTakeProfit}
                    />
                  </div>
                  {enableTakeProfit && (
                    <div className="space-y-2">
                      <Input
                        type="number"
                        step="0.1"
                        value={formData.config.take_profit_percentage}
                        onChange={(e) => setFormData({ 
                          ...formData, 
                          config: { ...formData.config, take_profit_percentage: e.target.value }
                        })}
                        placeholder="15"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">
                        Close strategy when total profit reaches this percentage
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Rebalance Threshold */}
              <div className="space-y-2">
                <Label htmlFor="rebalance_threshold" className="text-white/80">
                  Rebalance Threshold (%)
                </Label>
                <Input
                  id="rebalance_threshold"
                  type="number"
                  step="0.1"
                  value={formData.config.rebalance_threshold}
                  onChange={(e) => setFormData({ 
                    ...formData, 
                    config: { ...formData.config, rebalance_threshold: e.target.value }
                  })}
                  placeholder="2"
                  className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                />
                <p className="text-xs text-white/60">
                  Rebalance grid when price moves beyond this percentage from center
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
                  onClick={() => {
                    const backtestParams = {
                      start_date: formData.backtest_start_date || new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
                      end_date: formData.backtest_end_date || new Date().toISOString().split('T')[0],
                      interval: formData.backtest_interval || '1d',
                      initial_capital: formData.backtest_initial_capital || 10000
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
                className="bg-gradient-to-r from-green-600 to-emerald-600 hover:from-green-500 hover:to-emerald-500 text-white flex-1"
                disabled={isLoading}
              >
                {isLoading ? (
                  <div className="flex items-center space-x-2">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>Creating Strategy...</span>
                  </div>
                ) : (
                  <>
                    <TrendingUp className="mr-2 h-4 w-4" />
                    Create Grid Strategy
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
