"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { 
  CreateDCAStrategyRequest 
} from "@/lib/api"
import { 
  validateStrategyName, 
  validateAssetSymbol, 
  validateInvestmentAmount 
} from "@/lib/strategies"
import { AlertTriangle, TrendingUp, Zap, Activity, BarChart3, Clock, Target } from "lucide-react"
import { cn } from "@/lib/utils"

interface DCAConfigProps {
  initialData?: Partial<CreateDCAStrategyRequest>
  onSubmit: (data: CreateDCAStrategyRequest) => Promise<void>
  onCancel: () => void
  onBacktest?: (config: any, name: string, assetSymbol: string) => void
  isLoading?: boolean
  className?: string
}

export function DCAConfig({
  initialData,
  onSubmit,
  onCancel,
  onBacktest,
  isLoading = false,
  className
}: DCAConfigProps) {
  const [formData, setFormData] = useState<CreateDCAStrategyRequest>({
    name: initialData?.name || '',
    asset_symbol: initialData?.asset_symbol || '',
    total_allocation: initialData?.total_allocation || 1000,
    base_tranche_percentage: initialData?.base_tranche_percentage || 5,
    strategy_type: initialData?.strategy_type || 'conservative',
    sentiment_multiplier: initialData?.sentiment_multiplier ?? true,
    volatility_adjustment: initialData?.volatility_adjustment ?? true,
    fear_greed_threshold_buy: initialData?.fear_greed_threshold_buy || 30,
    fear_greed_threshold_sell: initialData?.fear_greed_threshold_sell || 70,
    dca_interval_hours: initialData?.dca_interval_hours || 24,
    target_zones: initialData?.target_zones || [],
    stop_loss_percentage: initialData?.stop_loss_percentage,
    take_profit_percentage: initialData?.take_profit_percentage
  })
  
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [enableStopLoss, setEnableStopLoss] = useState(!!initialData?.stop_loss_percentage)
  const [enableTakeProfit, setEnableTakeProfit] = useState(!!initialData?.take_profit_percentage)

  const validateForm = () => {
    const newErrors: Record<string, string> = {}
    
    const nameError = validateStrategyName(formData.name)
    if (nameError) newErrors.name = nameError
    
    const symbolError = validateAssetSymbol(formData.asset_symbol)
    if (symbolError) newErrors.asset_symbol = symbolError
    
    const allocationError = validateInvestmentAmount(formData.total_allocation)
    if (allocationError) newErrors.total_allocation = allocationError
    
    // DCA-specific validations
    if (formData.base_tranche_percentage < 1 || formData.base_tranche_percentage > 20) {
      newErrors.base_tranche_percentage = 'Base tranche percentage must be between 1% and 20%'
    }
    
    if (formData.fear_greed_threshold_buy >= formData.fear_greed_threshold_sell) {
      newErrors.fear_greed_thresholds = 'Buy threshold must be less than sell threshold'
    }
    
    if (formData.dca_interval_hours < 1 || formData.dca_interval_hours > 168) {
      newErrors.dca_interval_hours = 'DCA interval must be between 1 and 168 hours (1 week)'
    }
    
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!validateForm()) return
    
    const submitData: CreateDCAStrategyRequest = {
      ...formData,
      stop_loss_percentage: enableStopLoss ? formData.stop_loss_percentage : undefined,
      take_profit_percentage: enableTakeProfit ? formData.take_profit_percentage : undefined
    }
    
    try {
      await onSubmit(submitData)
    } catch (error) {
      console.error('Failed to create DCA strategy:', error)
    }
  }

  const getStrategyTypeInfo = (type: string) => {
    switch (type) {
      case 'conservative':
        return { description: 'Lower frequency, stable investments', risk: 'Low', color: 'text-green-400' }
      case 'moderate':
        return { description: 'Balanced approach with some volatility adaptation', risk: 'Medium', color: 'text-yellow-400' }
      case 'aggressive':
        return { description: 'Higher frequency, market-responsive', risk: 'High', color: 'text-red-400' }
      default:
        return { description: 'Standard DCA approach', risk: 'Medium', color: 'text-blue-400' }
    }
  }

  const strategyTypeInfo = getStrategyTypeInfo(formData.strategy_type)

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
            DCA Strategy
          </CardTitle>
          <CardDescription className="text-white/60">
            Dollar Cost Averaging with advanced market sentiment and volatility adjustments
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
                  placeholder="My DCA Strategy"
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

            {/* Investment Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Target className="h-5 w-5" />
                <span>Investment Configuration</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="total_allocation" className="text-white/80">Total Allocation ($)</Label>
                  <Input
                    id="total_allocation"
                    type="number"
                    step="0.01"
                    value={formData.total_allocation}
                    onChange={(e) => setFormData({ 
                      ...formData, 
                      total_allocation: parseFloat(e.target.value) || 0
                    })}
                    placeholder="1000"
                    className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
                  />
                  {errors.total_allocation && (
                    <p className="text-red-400 text-sm">{errors.total_allocation}</p>
                  )}
                </div>

                <div className="space-y-2">
                  <Label className="text-white/80">Strategy Type</Label>
                  <Select 
                    value={formData.strategy_type} 
                    onValueChange={(value) => setFormData({ ...formData, strategy_type: value })}
                  >
                    <SelectTrigger className="bg-white/10 border-white/20 text-white">
                      <SelectValue placeholder="Select strategy type" />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 border-white/20">
                      <SelectItem value="conservative">Conservative</SelectItem>
                      <SelectItem value="moderate">Moderate</SelectItem>
                      <SelectItem value="aggressive">Aggressive</SelectItem>
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

            {/* DCA Parameters */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Activity className="h-5 w-5" />
                <span>DCA Parameters</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Base Tranche Percentage */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">Base Tranche Percentage</Label>
                    <span className="text-blue-400 font-mono">{formData.base_tranche_percentage}%</span>
                  </div>
                  <Slider
                    value={[formData.base_tranche_percentage]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      base_tranche_percentage: value
                    })}
                    min={1}
                    max={20}
                    step={0.5}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    Percentage of total allocation per purchase
                  </div>
                  {errors.base_tranche_percentage && (
                    <p className="text-red-400 text-sm">{errors.base_tranche_percentage}</p>
                  )}
                </div>

                {/* DCA Interval */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label className="text-white/80">DCA Interval (Hours)</Label>
                    <span className="text-cyan-400 font-mono">{formData.dca_interval_hours}h</span>
                  </div>
                  <Slider
                    value={[formData.dca_interval_hours]}
                    onValueChange={([value]: number[]) => setFormData({ 
                      ...formData, 
                      dca_interval_hours: value
                    })}
                    min={1}
                    max={168}
                    step={1}
                    className="w-full"
                  />
                  <div className="text-sm text-white/60">
                    {formData.dca_interval_hours === 1 && 'Every hour'}
                    {formData.dca_interval_hours > 1 && formData.dca_interval_hours < 24 && `Every ${formData.dca_interval_hours} hours`}
                    {formData.dca_interval_hours === 24 && 'Daily'}
                    {formData.dca_interval_hours === 168 && 'Weekly'}
                    {formData.dca_interval_hours > 24 && formData.dca_interval_hours !== 168 && `Every ${(formData.dca_interval_hours / 24).toFixed(1)} days`}
                  </div>
                  {errors.dca_interval_hours && (
                    <p className="text-red-400 text-sm">{errors.dca_interval_hours}</p>
                  )}
                </div>
              </div>
            </div>

            {/* Market Sentiment Configuration */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Clock className="h-5 w-5" />
                <span>Market Intelligence</span>
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Sentiment Multiplier */}
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

                {/* Volatility Adjustment */}
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

              {/* Fear & Greed Thresholds */}
              {formData.sentiment_multiplier && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label className="text-white/80">Fear Threshold (Buy More)</Label>
                      <span className="text-green-400 font-mono">{formData.fear_greed_threshold_buy}</span>
                    </div>
                    <Slider
                      value={[formData.fear_greed_threshold_buy]}
                      onValueChange={([value]: number[]) => setFormData({ 
                        ...formData, 
                        fear_greed_threshold_buy: value
                      })}
                      min={0}
                      max={50}
                      step={1}
                      className="w-full"
                    />
                    <div className="text-sm text-green-400/70">
                      Increase purchases when fear is high
                    </div>
                  </div>

                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label className="text-white/80">Greed Threshold (Buy Less)</Label>
                      <span className="text-red-400 font-mono">{formData.fear_greed_threshold_sell}</span>
                    </div>
                    <Slider
                      value={[formData.fear_greed_threshold_sell]}
                      onValueChange={([value]: number[]) => setFormData({ 
                        ...formData, 
                        fear_greed_threshold_sell: value
                      })}
                      min={50}
                      max={100}
                      step={1}
                      className="w-full"
                    />
                    <div className="text-sm text-red-400/70">
                      Reduce purchases when greed is high
                    </div>
                  </div>
                </div>
              )}

              {errors.fear_greed_thresholds && (
                <p className="text-red-400 text-sm flex items-center space-x-1">
                  <AlertTriangle className="h-4 w-4" />
                  <span>{errors.fear_greed_thresholds}</span>
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
                      checked={enableTakeProfit}
                      onCheckedChange={setEnableTakeProfit}
                    />
                  </div>
                  {enableTakeProfit && (
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
            </div>

            {/* Strategy Tips */}
            <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
              <h4 className="font-medium text-blue-400 mb-2">💡 DCA Strategy Tips</h4>
              <ul className="text-sm text-white/70 space-y-1">
                <li>• DCA works best for long-term investment horizons (6+ months)</li>
                <li>• Enable sentiment multiplier to buy more during market fear</li>
                <li>• Volatility adjustment helps capitalize on market dips</li>
                <li>• Conservative approach: larger intervals, smaller percentages</li>
                <li>• Consider your risk tolerance when setting stop loss levels</li>
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
                  onClick={() => onBacktest(formData, formData.name, formData.asset_symbol)}
                  className="border-blue-500/30 text-blue-400 hover:bg-blue-500/10"
                  disabled={isLoading || !validateForm()}
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Backtest First
                </Button>
              )}
              
              <Button
                type="submit"
                className="bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-500 hover:to-cyan-500 text-white flex-1"
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
