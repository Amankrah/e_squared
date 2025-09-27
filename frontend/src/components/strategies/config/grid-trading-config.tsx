"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { 
  GridTradingConfig,
  CreateGridTradingStrategyRequest 
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount,
  formatCurrency 
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, Target, BarChart3 } from "lucide-react"
import { cn } from "@/lib/utils"

interface GridTradingConfigProps {
  initialData?: Partial<CreateGridTradingStrategyRequest>
  onSubmit: (data: CreateGridTradingStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: GridTradingConfig, name: string, assetSymbol: string) => void
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
  const [formData, setFormData] = useState<CreateGridTradingStrategyRequest>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      grid_count: initialData?.config?.grid_count || 10,
      lower_price: initialData?.config?.lower_price || '',
      upper_price: initialData?.config?.upper_price || '',
      investment_amount: initialData?.config?.investment_amount || '1000',
      stop_loss_percentage: initialData?.config?.stop_loss_percentage || '5',
      take_profit_percentage: initialData?.config?.take_profit_percentage || '15',
      rebalance_threshold: initialData?.config?.rebalance_threshold || '2'
    }
  })
  
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [enableStopLoss, setEnableStopLoss] = useState(!!initialData?.config?.stop_loss_percentage)
  const [enableTakeProfit, setEnableTakeProfit] = useState(!!initialData?.config?.take_profit_percentage)

  const validateForm = () => {
    const newErrors: Record<string, string> = {}
    
    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError
    
    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError
    
    const amountError = validateInvestmentAmount(formData.config.investment_amount)
    if (amountError) newErrors.investment_amount = amountError
    
    // Grid-specific validations
    if (!formData.config.lower_price || parseFloat(formData.config.lower_price) <= 0) {
      newErrors.lower_price = 'Lower price must be greater than 0'
    }
    
    if (!formData.config.upper_price || parseFloat(formData.config.upper_price) <= 0) {
      newErrors.upper_price = 'Upper price must be greater than 0'
    }
    
    if (formData.config.lower_price && formData.config.upper_price) {
      if (parseFloat(formData.config.lower_price) >= parseFloat(formData.config.upper_price)) {
        newErrors.price_range = 'Lower price must be less than upper price'
      }
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
    
    const submitData: CreateGridTradingStrategyRequest = {
      ...formData,
      config: {
        ...formData.config,
        stop_loss_percentage: enableStopLoss ? formData.config.stop_loss_percentage : undefined,
        take_profit_percentage: enableTakeProfit ? formData.config.take_profit_percentage : undefined
      }
    }
    
    try {
      await onSubmit(submitData)
    } catch (error) {
      console.error('Failed to create grid trading strategy:', error)
    }
  }

  const calculateGridSpacing = () => {
    if (!formData.config.lower_price || !formData.config.upper_price) return 0
    const range = parseFloat(formData.config.upper_price) - parseFloat(formData.config.lower_price)
    return range / (formData.config.grid_count - 1)
  }

  const calculateGridValue = () => {
    const spacing = calculateGridSpacing()
    const investmentPerGrid = parseFloat(formData.config.investment_amount) / formData.config.grid_count
    return { spacing, investmentPerGrid }
  }

  const { spacing, investmentPerGrid } = calculateGridValue()

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

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="lower_price" className="text-white/80">Lower Price ($)</Label>
                  <Input
                    id="lower_price"
                    type="number"
                    step="0.01"
                    value={formData.config.lower_price}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, lower_price: e.target.value }
                    })}
                    placeholder="35000"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.lower_price && (
                    <p className="text-red-400 text-sm">{errors.lower_price}</p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label htmlFor="upper_price" className="text-white/80">Upper Price ($)</Label>
                  <Input
                    id="upper_price"
                    type="number"
                    step="0.01"
                    value={formData.config.upper_price}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, upper_price: e.target.value }
                    })}
                    placeholder="50000"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.upper_price && (
                    <p className="text-red-400 text-sm">{errors.upper_price}</p>
                  )}
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
              {spacing > 0 && (
                <div className="bg-white/5 rounded-lg p-4 space-y-2">
                  <h4 className="font-medium text-white/80">Grid Analysis</h4>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-white/60">Grid Spacing: </span>
                      <span className="text-white/90 font-mono">${spacing.toFixed(2)}</span>
                    </div>
                    <div>
                      <span className="text-white/60">Investment per Grid: </span>
                      <span className="text-white/90 font-mono">${investmentPerGrid.toFixed(2)}</span>
                    </div>
                  </div>
                </div>
              )}

              {errors.price_range && (
                <p className="text-red-400 text-sm flex items-center space-x-1">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{errors.price_range}</span>
                </p>
              )}
            </div>

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
                  onClick={() => onBacktest(formData.config, formData.name, formData.asset_symbol)}
                  className="border-blue-500/30 text-blue-400 hover:bg-blue-500/10"
                  disabled={isLoading || !validateForm()}
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
