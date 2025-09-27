"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { 
  MACDConfig as MACDConfigType,
  CreateMACDStrategyRequest 
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount 
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, BarChart3, LineChart, TrendingDown } from "lucide-react"
import { cn } from "@/lib/utils"

interface MACDConfigProps {
  initialData?: Partial<CreateMACDStrategyRequest>
  onSubmit: (data: CreateMACDStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: MACDConfigType, name: string, assetSymbol: string) => void
  isLoading?: boolean
  className?: string
}

export function MACDConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: MACDConfigProps) {
  const [formData, setFormData] = useState<CreateMACDStrategyRequest>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    config: {
      fast_period: initialData?.config?.fast_period || 12,
      slow_period: initialData?.config?.slow_period || 26,
      signal_period: initialData?.config?.signal_period || 9,
      investment_amount: initialData?.config?.investment_amount || '400',
      stop_loss_percentage: initialData?.config?.stop_loss_percentage || '4',
      take_profit_percentage: initialData?.config?.take_profit_percentage || '12'
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
    
    // MACD-specific validations
    if (formData.config.fast_period >= formData.config.slow_period) {
      newErrors.periods = 'Fast period must be less than slow period'
    }
    
    if (formData.config.fast_period < 5 || formData.config.fast_period > 50) {
      newErrors.fast_period = 'Fast period must be between 5 and 50'
    }
    
    if (formData.config.slow_period < 10 || formData.config.slow_period > 100) {
      newErrors.slow_period = 'Slow period must be between 10 and 100'
    }
    
    if (formData.config.signal_period < 3 || formData.config.signal_period > 30) {
      newErrors.signal_period = 'Signal period must be between 3 and 30'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) return
    
    const submitData: CreateMACDStrategyRequest = {
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
      console.error('Failed to create MACD strategy:', error)
    }
  }

  const getSensitivityLevel = () => {
    const fastSlow = formData.config.slow_period - formData.config.fast_period
    const signalSpeed = formData.config.signal_period
    
    if (fastSlow < 10 || signalSpeed < 7) {
      return { level: 'High Sensitivity', color: 'text-red-400', description: 'More signals, higher noise' }
    }
    if (fastSlow < 20 || signalSpeed < 12) {
      return { level: 'Balanced', color: 'text-yellow-400', description: 'Good balance of signals and reliability' }
    }
    return { level: 'Low Sensitivity', color: 'text-green-400', description: 'Fewer but more reliable signals' }
  }

  const sensitivity = getSensitivityLevel()

  return (
    <Card className={cn(
      "bg-gradient-to-br from-indigo-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10",
      className
    )}>
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
      
      <div className="relative z-10">
        <CardHeader className="text-center space-y-2">
          <div className="text-4xl mx-auto">🌊</div>
          <CardTitle className="text-2xl font-bold text-white/90">
            MACD Strategy
          </CardTitle>
          <CardDescription className="text-white/60">
            Use MACD line crossovers and histogram divergences for precise market timing
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
                  placeholder="My MACD Strategy"
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
                placeholder="400"
                className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
              />
              {errors.investment_amount && (
                <p className="text-red-400 text-sm">{errors.investment_amount}</p>
              )}
            </div>

            {/* MACD Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <BarChart3 className="h-5 w-5" />
                <span>MACD Indicator Settings</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {/* Fast Period */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Fast Period</Label>
                    <span className="text-blue-400 font-mono">{formData.config.fast_period}</span>
                  </div>
                  <Slider
                    value={[formData.config.fast_period]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, fast_period: value }
                    })}
                    min={5}
                    max={50}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Fast EMA (usually 12)
                  </div>
                  {errors.fast_period && (
                    <p className="text-red-400 text-sm">{errors.fast_period}</p>
                  )}
                </div>

                {/* Slow Period */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Slow Period</Label>
                    <span className="text-indigo-400 font-mono">{formData.config.slow_period}</span>
                  </div>
                  <Slider
                    value={[formData.config.slow_period]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, slow_period: value }
                    })}
                    min={10}
                    max={100}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Slow EMA (usually 26)
                  </div>
                  {errors.slow_period && (
                    <p className="text-red-400 text-sm">{errors.slow_period}</p>
                  )}
                </div>

                {/* Signal Period */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Signal Period</Label>
                    <span className="text-cyan-400 font-mono">{formData.config.signal_period}</span>
                  </div>
                  <Slider
                    value={[formData.config.signal_period]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      config: { ...formData.config, signal_period: value }
                    })}
                    min={3}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Signal line EMA (usually 9)
                  </div>
                  {errors.signal_period && (
                    <p className="text-red-400 text-sm">{errors.signal_period}</p>
                  )}
                </div>
              </div>

              {/* MACD Formula Explanation */}
              <div className="bg-white/5 rounded-lg p-4 space-y-3">
                <h4 className="font-medium text-white/80">MACD Calculation</h4>
                <div className="space-y-2 text-sm font-mono">
                  <div className="text-white/70">
                    <span className="text-blue-400">MACD Line</span> = EMA({formData.config.fast_period}) - EMA({formData.config.slow_period})
                  </div>
                  <div className="text-white/70">
                    <span className="text-cyan-400">Signal Line</span> = EMA({formData.config.signal_period}) of MACD Line
                  </div>
                  <div className="text-white/70">
                    <span className="text-yellow-400">Histogram</span> = MACD Line - Signal Line
                  </div>
                </div>
                <div className="text-xs text-white/60">
                  <span className={cn("font-medium", sensitivity.color)}>{sensitivity.level}</span>
                  : {sensitivity.description}
                </div>
              </div>

              {errors.periods && (
                <p className="text-red-400 text-sm flex items-center space-x-1">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{errors.periods}</span>
                </p>
              )}
            </div>

            {/* Trading Signals */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <LineChart className="h-5 w-5" />
                <span>Trading Signals</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Buy Signals */}
                <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-4 space-y-2">
                  <div className="flex items-center space-x-2">
                    <TrendingUp className="h-5 w-5 text-green-400" />
                    <span className="font-medium text-green-400">Buy Signals</span>
                  </div>
                  <ul className="text-sm text-white/70 space-y-1">
                    <li>• MACD line crosses above signal line</li>
                    <li>• MACD histogram turns positive</li>
                    <li>• MACD line crosses above zero line</li>
                  </ul>
                </div>

                {/* Sell Signals */}
                <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4 space-y-2">
                  <div className="flex items-center space-x-2">
                    <TrendingDown className="h-5 w-5 text-red-400" />
                    <span className="font-medium text-red-400">Sell Signals</span>
                  </div>
                  <ul className="text-sm text-white/70 space-y-1">
                    <li>• MACD line crosses below signal line</li>
                    <li>• MACD histogram turns negative</li>
                    <li>• MACD line crosses below zero line</li>
                  </ul>
                </div>
              </div>

              {/* Divergence Analysis */}
              <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4">
                <div className="flex items-center space-x-2 mb-2">
                  <BarChart3 className="h-5 w-5 text-yellow-400" />
                  <span className="font-medium text-yellow-400">Divergence Signals</span>
                </div>
                <ul className="text-sm text-white/70 space-y-1">
                  <li>• <strong>Bullish Divergence:</strong> Price makes lower lows, MACD makes higher lows</li>
                  <li>• <strong>Bearish Divergence:</strong> Price makes higher highs, MACD makes lower highs</li>
                  <li>• <strong>Hidden Divergence:</strong> Trend continuation signals</li>
                </ul>
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
                        placeholder="4"
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
                        placeholder="12"
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
            <div className="bg-indigo-500/10 border border-indigo-500/20 rounded-lg p-4">
              <h4 className="font-medium text-indigo-400 mb-2">💡 MACD Strategy Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• MACD works best in trending markets, less effective in sideways markets</li>
                <li>• Standard settings (12,26,9) work well for most timeframes</li>
                <li>• Use histogram for early signal detection</li>
                <li>• Look for divergences to spot trend reversals</li>
                <li>• Combine with support/resistance levels for better entries</li>
                <li>• Wait for MACD to cross zero line for stronger trend confirmation</li>
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
                className="bg-gradient-to-r from-indigo-600 to-blue-600 hover:from-indigo-500 hover:to-blue-500 text-white flex-1"
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
                    Create MACD Strategy
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
