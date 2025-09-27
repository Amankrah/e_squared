"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { 
  BacktestRequest, 
  BacktestResult, 
  StrategyType, 
  StrategyConfig,
  apiClient
} from "@/lib/api"
import { 
  getStrategyInfo, 
  getStrategyDisplayName, 
  formatCurrency, 
  formatPercentage 
} from "@/lib/strategies"
import { 
  Play, 
  TrendingUp, 
  TrendingDown, 
  BarChart3, 
  Calendar, 
  DollarSign,
  AlertTriangle,
  RefreshCw,
  LineChart
} from "lucide-react"
import { BacktestResults } from "./backtest-results"
import { cn } from "@/lib/utils"

interface BacktestingInterfaceProps {
  strategyType: StrategyType
  strategyConfig: StrategyConfig
  assetSymbol: string
  onClose?: () => void
  className?: string
}

export function BacktestingInterface({
  strategyType,
  strategyConfig,
  assetSymbol,
  onClose,
  className
}: BacktestingInterfaceProps) {
  const [backtestConfig, setBacktestConfig] = useState({
    start_date: new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], // 1 year ago
    end_date: new Date().toISOString().split('T')[0], // Today
    initial_capital: 10000
  })
  
  const [isRunning, setIsRunning] = useState(false)
  const [results, setResults] = useState<BacktestResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  
  const strategyInfo = getStrategyInfo(strategyType)

  const validateConfig = () => {
    const errors = []
    
    const startDate = new Date(backtestConfig.start_date)
    const endDate = new Date(backtestConfig.end_date)
    const today = new Date()
    
    if (startDate >= endDate) {
      errors.push('Start date must be before end date')
    }
    
    if (endDate >= today) {
      errors.push('End date must be in the past')
    }
    
    if (backtestConfig.initial_capital < 100) {
      errors.push('Initial capital must be at least $100')
    }
    
    if (backtestConfig.initial_capital > 1000000) {
      errors.push('Initial capital must be less than $1,000,000')
    }
    
    // Check if we have enough historical data (at least 30 days)
    const daysDiff = (endDate.getTime() - startDate.getTime()) / (1000 * 60 * 60 * 24)
    if (daysDiff < 30) {
      errors.push('Backtest period must be at least 30 days')
    }
    
    return errors
  }

  const handleRunBacktest = async () => {
    const errors = validateConfig()
    if (errors.length > 0) {
      setError(errors.join(', '))
      return
    }
    
    setIsRunning(true)
    setError(null)
    setResults(null)
    
    try {
      const request: BacktestRequest = {
        strategy_type: strategyType,
        asset_symbol: assetSymbol,
        start_date: backtestConfig.start_date,
        end_date: backtestConfig.end_date,
        initial_capital: backtestConfig.initial_capital,
        config: strategyConfig
      }
      
      const result = await apiClient.runBacktest(request)
      setResults(result)
    } catch (err) {
      console.error('Backtest failed:', err)
      setError('Backtest failed. Please check your configuration and try again.')
    } finally {
      setIsRunning(false)
    }
  }

  const getDateRangePreset = (preset: string) => {
    const end = new Date().toISOString().split('T')[0]
    let start: string
    
    switch (preset) {
      case '1m':
        start = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        break
      case '3m':
        start = new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        break
      case '6m':
        start = new Date(Date.now() - 180 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        break
      case '1y':
        start = new Date(Date.now() - 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        break
      case '2y':
        start = new Date(Date.now() - 2 * 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]
        break
      default:
        return
    }
    
    setBacktestConfig({ ...backtestConfig, start_date: start, end_date: end })
  }

  const calculateDuration = () => {
    const start = new Date(backtestConfig.start_date)
    const end = new Date(backtestConfig.end_date)
    const days = Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24))
    
    if (days < 30) return `${days} days`
    if (days < 365) return `${Math.ceil(days / 30)} months`
    return `${(days / 365).toFixed(1)} years`
  }

  return (
    <div className={cn("space-y-6", className)}>
      {/* Configuration Panel */}
      <Card className={cn(
        "bg-gradient-to-br backdrop-blur-xl border border-white/10",
        strategyInfo.color
      )}>
        <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
        
        <div className="relative z-10">
          <CardHeader className="text-center space-y-2">
            <div className="flex items-center justify-center space-x-3">
              <div className="text-3xl">{strategyInfo.icon}</div>
              <div className="text-center">
                <CardTitle className="text-xl font-bold text-white/90">
                  Backtest {getStrategyDisplayName(strategyType)}
                </CardTitle>
                <CardDescription className="text-white/60">
                  Test your strategy with historical data for {assetSymbol}
                </CardDescription>
              </div>
            </div>
          </CardHeader>

          <CardContent className="space-y-6">
            {/* Date Range Selection */}
            <div className="space-y-4">
              <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                <Calendar className="h-5 w-5" />
                <span>Backtest Period</span>
              </h3>
              
              {/* Preset Buttons */}
              <div className="flex flex-wrap gap-2">
                {[
                  { label: '1 Month', value: '1m' },
                  { label: '3 Months', value: '3m' },
                  { label: '6 Months', value: '6m' },
                  { label: '1 Year', value: '1y' },
                  { label: '2 Years', value: '2y' }
                ].map((preset) => (
                  <Button
                    key={preset.value}
                    variant="outline"
                    size="sm"
                    onClick={() => getDateRangePreset(preset.value)}
                    className="border-white/20 text-white/80 hover:bg-white/10"
                  >
                    {preset.label}
                  </Button>
                ))}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="start_date" className="text-white/80">Start Date</Label>
                  <Input
                    id="start_date"
                    type="date"
                    value={backtestConfig.start_date}
                    onChange={(e) => setBacktestConfig({ 
                      ...backtestConfig, 
                      start_date: e.target.value 
                    })}
                    className="bg-white/10 border-white/20 text-white"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="end_date" className="text-white/80">End Date</Label>
                  <Input
                    id="end_date"
                    type="date"
                    value={backtestConfig.end_date}
                    onChange={(e) => setBacktestConfig({ 
                      ...backtestConfig, 
                      end_date: e.target.value 
                    })}
                    className="bg-white/10 border-white/20 text-white"
                  />
                </div>
              </div>

              <div className="bg-white/5 rounded-lg p-3">
                <p className="text-sm text-white/70">
                  <strong>Duration:</strong> {calculateDuration()}
                </p>
              </div>
            </div>

            {/* Initial Capital */}
            <div className="space-y-2">
              <Label htmlFor="initial_capital" className="text-white/80 flex items-center space-x-2">
                <DollarSign className="h-4 w-4" />
                <span>Initial Capital ($)</span>
              </Label>
              <Input
                id="initial_capital"
                type="number"
                step="100"
                value={backtestConfig.initial_capital}
                onChange={(e) => setBacktestConfig({ 
                  ...backtestConfig, 
                  initial_capital: parseInt(e.target.value) || 0
                })}
                className="bg-white/10 border-white/20 text-white placeholder:text-white/50"
              />
              <p className="text-xs text-white/60">
                The amount of virtual capital to use for backtesting
              </p>
            </div>

            {/* Error Display */}
            {error && (
              <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                <div className="flex items-center space-x-2">
                  <AlertTriangle className="h-5 w-5 text-red-400" />
                  <span className="font-medium text-red-400">Backtest Error</span>
                </div>
                <p className="text-sm text-red-300 mt-1">{error}</p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex flex-col sm:flex-row gap-3 pt-4">
              {onClose && (
                <Button
                  type="button"
                  variant="outline"
                  onClick={onClose}
                  className="border-white/20 text-white/80 hover:bg-white/10"
                  disabled={isRunning}
                >
                  Cancel
                </Button>
              )}
              
              <Button
                onClick={handleRunBacktest}
                disabled={isRunning}
                className="bg-gradient-to-r from-green-600 to-emerald-600 hover:from-green-500 hover:to-emerald-500 text-white flex-1"
              >
                {isRunning ? (
                  <div className="flex items-center space-x-2">
                    <RefreshCw className="h-4 w-4 animate-spin" />
                    <span>Running Backtest...</span>
                  </div>
                ) : (
                  <>
                    <Play className="mr-2 h-4 w-4" />
                    Run Backtest
                  </>
                )}
              </Button>
            </div>
          </CardContent>
        </div>
      </Card>

      {/* Results Panel */}
      {results && (
        <BacktestResults 
          results={results} 
          strategyInfo={strategyInfo}
          className="animate-in fade-in duration-500"
        />
      )}
    </div>
  )
}
