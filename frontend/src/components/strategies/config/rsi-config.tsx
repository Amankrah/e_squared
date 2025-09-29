"use client"

import { useState, useMemo } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { 
  RSIConfig as RSIConfigType,
  CreateRSIStrategyRequest 
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount 
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, Activity, BarChart3 } from "lucide-react"
import { cn } from "@/lib/utils"

interface RSIConfigProps {
  initialData?: Partial<CreateRSIStrategyRequest>
  onSubmit: (data: CreateRSIStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: RSIConfigType, name: string, assetSymbol: string) => void
  isLoading?: boolean
  className?: string
}

export function RSIConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: RSIConfigProps) {
  const [formData, setFormData] = useState<CreateRSIStrategyRequest>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      rsi_period: initialData?.config?.rsi_period || 14,
      oversold_threshold: initialData?.config?.oversold_threshold || 30,
      overbought_threshold: initialData?.config?.overbought_threshold || 70,
      investment_amount: initialData?.config?.investment_amount || '300',
      stop_loss_percentage: initialData?.config?.stop_loss_percentage || '3',
      take_profit_percentage: initialData?.config?.take_profit_percentage || '8'
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
    
    // RSI-specific validations
    if (formData.config.rsi_period < 5 || formData.config.rsi_period > 50) {
      newErrors.rsi_period = 'RSI period must be between 5 and 50'
    }
    
    if (formData.config.oversold_threshold >= formData.config.overbought_threshold) {
      newErrors.thresholds = 'Oversold threshold must be less than overbought threshold'
    }
    
    if (formData.config.oversold_threshold < 10 || formData.config.oversold_threshold > 40) {
      newErrors.oversold_threshold = 'Oversold threshold should be between 10 and 40'
    }
    
    if (formData.config.overbought_threshold < 60 || formData.config.overbought_threshold > 90) {
      newErrors.overbought_threshold = 'Overbought threshold should be between 60 and 90'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) return
    
    const submitData: CreateRSIStrategyRequest = {
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
      console.error('Failed to create RSI strategy:', error)
    }
  }

  const getRSIZoneColor = (value: number) => {
    if (value <= formData.config.oversold_threshold) return 'text-green-400'
    if (value >= formData.config.overbought_threshold) return 'text-red-400'
    return 'text-white/60'
  }

  const getRSIZoneLabel = (value: number) => {
    if (value <= formData.config.oversold_threshold) return 'Oversold (Buy Zone)'
    if (value >= formData.config.overbought_threshold) return 'Overbought (Sell Zone)'
    return 'Neutral Zone'
  }

  const isFormValid = useMemo(() => {
    const nameError = validateStrategyName(formData.name)
    if (nameError) return false

    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) return false

    const amountError = validateInvestmentAmount(formData.config.investment_amount)
    if (amountError) return false

    if (formData.config.rsi_period < 5 || formData.config.rsi_period > 50) return false
    if (formData.config.oversold_threshold >= formData.config.overbought_threshold) return false
    if (formData.config.oversold_threshold < 10 || formData.config.oversold_threshold > 40) return false
    if (formData.config.overbought_threshold < 60 || formData.config.overbought_threshold > 90) return false

    return true
  }, [formData])

  return (
    <Card className={cn(
      "bg-gradient-to-br from-yellow-500/20 to-orange-500/20 backdrop-blur-xl border border-white/10",
      className
    )}>
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
      
      <div className="relative z-10">
        <CardHeader className="text-center space-y-2">
          <div className="text-4xl mx-auto">⚡</div>
          <CardTitle className="text-2xl font-bold text-white/90">
            RSI Strategy
          </CardTitle>
          <CardDescription className="text-white/60">
            Trade based on Relative Strength Index overbought and oversold conditions
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
                  placeholder="My RSI Strategy"
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

            {/* Investment Amount */}
            <div className="space-y-2">
              <Label htmlFor="investment_amount" className="text-white/80">Investment Amount ($)</Label>
              <Input
                id="investment_amount"
                type="number"
                step="0.01"
                value={formData.config.investment_amount}
                onChange={(e) => setFormData({ 
                  ...formData, 
                  config: { ...formData.config, investment_amount: e.target.value }
                })}
                placeholder="300"
                className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
              />
              {errors.investment_amount && (
                <p className="text-red-400 text-sm">{errors.investment_amount}</p>
              )}
            </div>

            {/* RSI Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Activity className="h-5 w-5" />
                <span>RSI Indicator Settings</span>
              </h3>

              {/* RSI Period */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label className="text-white/80">RSI Period</Label>
                  <span className="text-white/60 font-mono">{formData.config.rsi_period} periods</span>
                </div>
                <Slider
                  value={[formData.config.rsi_period]}
                  onValueChange={([value]) => setFormData({ 
                    ...formData, 
                    config: { ...formData.config, rsi_period: value }
                  })}
                  min={5}
                  max={50}
                  step={1}
                  className="w-full"
                />
                <div className="text-sm text-white/60">
                  Higher periods = smoother RSI, less sensitive to price changes
                </div>
                {errors.rsi_period && (
                  <p className="text-red-400 text-sm">{errors.rsi_period}</p>
                )}
              </div>

              {/* RSI Thresholds */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Oversold Threshold */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Oversold Threshold</Label>
                    <span className="text-green-400 font-mono">{formData.config.oversold_threshold}</span>
                  </div>
                  <Slider
                    value={[formData.config.oversold_threshold]}
                    onValueChange={([value]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, oversold_threshold: value }
                    })}
                    min={10}
                    max={40}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-green-400/70">
                    Buy signals when RSI drops below this level
                  </div>
                  {errors.oversold_threshold && (
                    <p className="text-red-400 text-sm">{errors.oversold_threshold}</p>
                  )}
                </div>

                {/* Overbought Threshold */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Overbought Threshold</Label>
                    <span className="text-red-400 font-mono">{formData.config.overbought_threshold}</span>
                  </div>
                  <Slider
                    value={[formData.config.overbought_threshold]}
                    onValueChange={([value]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, overbought_threshold: value }
                    })}
                    min={60}
                    max={90}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-red-400/70">
                    Sell signals when RSI rises above this level
                  </div>
                  {errors.overbought_threshold && (
                    <p className="text-red-400 text-sm">{errors.overbought_threshold}</p>
                  )}
                </div>
              </div>

              {/* RSI Zones Visualization */}
              <div className="bg-white/5 rounded-lg p-4 space-y-3">
                <h4 className="font-medium text-white/80">RSI Zones</h4>
                <div className="space-y-2">
                  <div className="flex justify-between items-center text-sm">
                    <span>0 - {formData.config.oversold_threshold}</span>
                    <span className="text-green-400 font-medium">Oversold (Buy Zone)</span>
                  </div>
                  <div className="flex justify-between items-center text-sm">
                    <span>{formData.config.oversold_threshold} - {formData.config.overbought_threshold}</span>
                    <span className="text-white/60 font-medium">Neutral Zone</span>
                  </div>
                  <div className="flex justify-between items-center text-sm">
                    <span>{formData.config.overbought_threshold} - 100</span>
                    <span className="text-red-400 font-medium">Overbought (Sell Zone)</span>
                  </div>
                </div>
              </div>

              {errors.thresholds && (
                <p className="text-red-400 text-sm flex items-center space-x-1">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{errors.thresholds}</span>
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
                        placeholder="3"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">
                        Exit position if loss exceeds this percentage
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
                        placeholder="8"
                        className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                      <p className="text-xs text-white/60">
                        Close position when profit reaches this percentage
                      </p>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Strategy Tips */}
            <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4">
              <h4 className="font-medium text-yellow-400 mb-2">💡 RSI Strategy Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• RSI works best in ranging markets, less effective in strong trends</li>
                <li>• Consider combining with other indicators for better signal confirmation</li>
                <li>• Lower thresholds (20/80) = fewer but stronger signals</li>
                <li>• Higher thresholds (35/65) = more frequent but weaker signals</li>
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
                  onClick={() => onBacktest(formData.config, formData.name, formData.asset_symbol)}
                  className="border-blue-500/30 text-blue-400 hover:bg-blue-500/10"
                  disabled={isLoading || !isFormValid}
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Backtest First
                </Button>
              )}
              
              <Button
                type="submit"
                className="bg-gradient-to-r from-yellow-600 to-orange-600 hover:from-yellow-500 hover:to-orange-500 text-white flex-1"
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
                    Create RSI Strategy
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
