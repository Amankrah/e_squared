"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { 
  SMACrossoverConfig as SMACrossoverConfigType,
  CreateSMACrossoverStrategyRequest 
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount 
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, TrendingDown, Activity, LineChart, BarChart3 } from "lucide-react"
import { cn } from "@/lib/utils"

interface SMAConfigProps {
  initialData?: Partial<CreateSMACrossoverStrategyRequest>
  onSubmit: (data: CreateSMACrossoverStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: SMACrossoverConfigType, name: string, assetSymbol: string) => void
  isLoading?: boolean
  className?: string
}

export function SMAConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: SMAConfigProps) {
  const [formData, setFormData] = useState<CreateSMACrossoverStrategyRequest>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      short_period: initialData?.config?.short_period || 20,
      long_period: initialData?.config?.long_period || 50,
      investment_amount: initialData?.config?.investment_amount || '500',
      stop_loss_percentage: initialData?.config?.stop_loss_percentage || '5',
      take_profit_percentage: initialData?.config?.take_profit_percentage || '10',
      confirmation_period: initialData?.config?.confirmation_period || 2
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
    
    // SMA-specific validations
    if (formData.config.short_period >= formData.config.long_period) {
      newErrors.periods = 'Short period must be less than long period'
    }
    
    if (formData.config.short_period < 5 || formData.config.short_period > 100) {
      newErrors.short_period = 'Short period must be between 5 and 100'
    }
    
    if (formData.config.long_period < 10 || formData.config.long_period > 200) {
      newErrors.long_period = 'Long period must be between 10 and 200'
    }
    
    if (formData.config.confirmation_period && (formData.config.confirmation_period < 1 || formData.config.confirmation_period > 10)) {
      newErrors.confirmation_period = 'Confirmation period must be between 1 and 10'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) return
    
    const submitData: CreateSMACrossoverStrategyRequest = {
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
      console.error('Failed to create SMA crossover strategy:', error)
    }
  }

  const getSignalStrength = () => {
    const difference = formData.config.long_period - formData.config.short_period
    if (difference < 15) return { strength: 'High Frequency', color: 'text-red-400', description: 'More signals, higher noise' }
    if (difference < 30) return { strength: 'Balanced', color: 'text-yellow-400', description: 'Good balance of signals and reliability' }
    return { strength: 'Low Frequency', color: 'text-green-400', description: 'Fewer but more reliable signals' }
  }

  const signalStrength = getSignalStrength()

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
            SMA Crossover Strategy
          </CardTitle>
          <CardDescription className="text-white/60">
            Trade when short-term moving average crosses above or below long-term moving average
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
                  placeholder="My SMA Crossover Strategy"
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
                placeholder="500"
                className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
              />
              {errors.investment_amount && (
                <p className="text-red-400 text-sm">{errors.investment_amount}</p>
              )}
            </div>

            {/* SMA Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <LineChart className="h-5 w-5" />
                <span>Moving Average Settings</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Short Period */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Short Period (Fast SMA)</Label>
                    <span className="text-purple-400 font-mono">{formData.config.short_period} periods</span>
                  </div>
                  <Slider
                    value={[formData.config.short_period]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, short_period: value }
                    })}
                    min={5}
                    max={100}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Responds quickly to price changes
                  </div>
                  {errors.short_period && (
                    <p className="text-red-400 text-sm">{errors.short_period}</p>
                  )}
                </div>

                {/* Long Period */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Long Period (Slow SMA)</Label>
                    <span className="text-pink-400 font-mono">{formData.config.long_period} periods</span>
                  </div>
                  <Slider
                    value={[formData.config.long_period]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, long_period: value }
                    })}
                    min={10}
                    max={200}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Smooths out price noise, shows trend
                  </div>
                  {errors.long_period && (
                    <p className="text-red-400 text-sm">{errors.long_period}</p>
                  )}
                </div>
              </div>

              {/* Signal Analysis */}
              <div className="bg-white/5 rounded-lg p-4 space-y-3">
                <h4 className="font-medium text-white/80">Signal Analysis</h4>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-white/60">Period Difference: </span>
                    <span className="text-white/90 font-mono">{formData.config.long_period - formData.config.short_period}</span>
                  </div>
                  <div>
                    <span className="text-white/60">Signal Frequency: </span>
                    <span className={cn("font-medium", signalStrength.color)}>{signalStrength.strength}</span>
                  </div>
                </div>
                <p className="text-xs text-white/60">{signalStrength.description}</p>
              </div>

              {/* Trading Signals Explanation */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3">
                  <div className="flex items-center space-x-2 mb-2">
                    <TrendingUp className="h-4 w-4 text-green-400" />
                    <span className="font-medium text-green-400">Buy Signal</span>
                  </div>
                  <p className="text-white/70">
                    When short SMA ({formData.config.short_period}) crosses above long SMA ({formData.config.long_period})
                  </p>
                </div>

                <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3">
                  <div className="flex items-center space-x-2 mb-2">
                    <TrendingDown className="h-4 w-4 text-red-400" />
                    <span className="font-medium text-red-400">Sell Signal</span>
                  </div>
                  <p className="text-white/70">
                    When short SMA ({formData.config.short_period}) crosses below long SMA ({formData.config.long_period})
                  </p>
                </div>
              </div>

              {errors.periods && (
                <p className="text-red-400 text-sm flex items-center space-x-1">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{errors.periods}</span>
                </p>
              )}
            </div>

            {/* Advanced Settings */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Activity className="h-5 w-5" />
                <span>Advanced Settings</span>
              </h3>

              {/* Confirmation Period */}
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label className="text-white/80">Confirmation Period</Label>
                  <span className="text-white/60 font-mono">{formData.config.confirmation_period} periods</span>
                </div>
                <Slider
                  value={[formData.config.confirmation_period || 2]}
                  onValueChange={([value]: number[]) => setFormData({ 
                    ...formData, 
                    config: { ...formData.config, confirmation_period: value }
                  })}
                  min={1}
                  max={10}
                  step={1}
                  className="w-full"
                />
                <div className="text-sm text-white/60">
                  Wait for this many periods after crossover before executing trade
                </div>
                {errors.confirmation_period && (
                  <p className="text-red-400 text-sm">{errors.confirmation_period}</p>
                )}
              </div>
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
                        placeholder="10"
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
            <div className="bg-purple-500/10 border border-purple-500/20 rounded-lg p-4">
              <h4 className="font-medium text-purple-400 mb-2">💡 SMA Crossover Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• Works best in trending markets, less effective in sideways markets</li>
                <li>• Popular combinations: 20/50, 50/200, 9/21</li>
                <li>• Longer periods = fewer false signals but slower response</li>
                <li>• Use confirmation period to reduce whipsaws</li>
                <li>• Consider volume confirmation for stronger signals</li>
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
                  disabled={isLoading || !validateForm()}
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Backtest First
                </Button>
              )}
              
              <Button
                type="submit"
                className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white flex-1"
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
                    Create SMA Strategy
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
