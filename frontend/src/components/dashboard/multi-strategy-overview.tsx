"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { 
  TrendingUp, 
  TrendingDown, 
  Activity, 
  Target, 
  BarChart3,
  ArrowRight,
  PlusCircle,
  RefreshCw
} from "lucide-react"
import { 
  apiClient, 
  Strategy, 
  StrategyType
} from "@/lib/api"
import { 
  getStrategyInfo, 
  formatCurrency, 
  formatPercentage, 
  getStrategyStatusColor,
  STRATEGY_INFO
} from "@/lib/strategies"
import { useAuth } from "@/contexts/auth-context"
import Link from "next/link"
import { cn } from "@/lib/utils"

interface MultiStrategyOverviewProps {
  className?: string
}

export function MultiStrategyOverview({ className }: MultiStrategyOverviewProps) {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [allStrategies, setAllStrategies] = useState<{
    dca: Strategy[]
    gridTrading: Strategy[]
    smaCrossover: Strategy[]
    rsi: Strategy[]
    macd: Strategy[]
  }>({
    dca: [],
    gridTrading: [],
    smaCrossover: [],
    rsi: [],
    macd: []
  })
  const [loading, setLoading] = useState(false)
  const [summary, setSummary] = useState({
    totalStrategies: 0,
    activeStrategies: 0,
    totalInvested: '0',
    totalProfitLoss: '0',
    bestPerformer: null as { strategy: Strategy; type: StrategyType } | null,
    worstPerformer: null as { strategy: Strategy; type: StrategyType } | null
  })

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadStrategies()
    }
  }, [isAuthenticated, authLoading])

  const loadStrategies = async () => {
    if (!isAuthenticated) return
    
    setLoading(true)
    try {
      const results = await apiClient.getAllStrategies()
      
      setAllStrategies({
        dca: results.dca.strategies || [],
        gridTrading: results.gridTrading.strategies || [],
        smaCrossover: results.smaCrossover.strategies || [],
        rsi: results.rsi.strategies || [],
        macd: results.macd.strategies || []
      })

      // Calculate enhanced summary statistics
      const allStrategyArrays = [
        ...results.dca.strategies.map(s => ({ strategy: s, type: 'dca' as StrategyType })),
        ...results.gridTrading.strategies.map(s => ({ strategy: s, type: 'grid_trading' as StrategyType })),
        ...results.smaCrossover.strategies.map(s => ({ strategy: s, type: 'sma_crossover' as StrategyType })),
        ...results.rsi.strategies.map(s => ({ strategy: s, type: 'rsi' as StrategyType })),
        ...results.macd.strategies.map(s => ({ strategy: s, type: 'macd' as StrategyType }))
      ]
      
      const totalStrategies = allStrategyArrays.length
      const activeStrategies = allStrategyArrays.filter(({ strategy }) => 
        strategy.status?.toLowerCase() === 'active'
      ).length
      
      const totalInvested = allStrategyArrays.reduce((sum, { strategy }) => 
        sum + parseFloat(strategy.total_invested || '0'), 0
      )
      
      const totalProfitLoss = allStrategyArrays.reduce((sum, { strategy }) => 
        sum + parseFloat(strategy.current_profit_loss || '0'), 0
      )

      // Find best and worst performers
      let bestPerformer = null
      let worstPerformer = null
      let bestPnL = -Infinity
      let worstPnL = Infinity

      allStrategyArrays.forEach(({ strategy, type }) => {
        const pnl = parseFloat(strategy.current_profit_loss || '0')
        if (pnl > bestPnL) {
          bestPnL = pnl
          bestPerformer = { strategy, type }
        }
        if (pnl < worstPnL) {
          worstPnL = pnl
          bestPerformer = { strategy, type }
        }
      })

      setSummary({
        totalStrategies,
        activeStrategies,
        totalInvested: totalInvested.toString(),
        totalProfitLoss: totalProfitLoss.toString(),
        bestPerformer,
        worstPerformer
      })

    } catch (error) {
      console.error('Failed to load strategies:', error)
    } finally {
      setLoading(false)
    }
  }

  // Get strategy type breakdown
  const getStrategyBreakdown = () => {
    return Object.entries(allStrategies).map(([key, strategies]) => {
      const type = key === 'gridTrading' ? 'grid_trading' : 
                   key === 'smaCrossover' ? 'sma_crossover' : key as StrategyType
      const info = getStrategyInfo(type)
      return {
        type,
        info,
        count: strategies.length,
        activeCount: strategies.filter(s => s.status?.toLowerCase() === 'active').length
      }
    }).filter(item => item.count > 0)
  }

  const strategyBreakdown = getStrategyBreakdown()

  if (authLoading) {
    return (
      <Card className={cn("bg-white/5 backdrop-blur-xl border border-white/10", className)}>
        <CardContent className="p-6">
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-white/10 rounded w-1/2"></div>
            <div className="h-8 bg-white/10 rounded"></div>
            <div className="h-4 bg-white/10 rounded w-3/4"></div>
          </div>
        </CardContent>
      </Card>
    )
  }

  if (!isAuthenticated) {
    return (
      <Card className={cn("bg-white/5 backdrop-blur-xl border border-white/10", className)}>
        <CardContent className="p-6 text-center">
          <p className="text-white/60">Please log in to view your strategies</p>
        </CardContent>
      </Card>
    )
  }

  return (
    <div className={cn("space-y-6", className)}>
      {/* Main Overview Card */}
      <Card className="bg-gradient-to-br from-purple-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10">
        <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
        
        <div className="relative z-10">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
            <div>
              <CardTitle className="text-xl font-bold text-white/90">
                Strategy Portfolio Overview
              </CardTitle>
              <CardDescription className="text-white/60">
                Performance across all your trading strategies
              </CardDescription>
            </div>
            
            <div className="flex items-center space-x-2">
              <Button
                onClick={loadStrategies}
                variant="outline"
                size="sm"
                disabled={loading}
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
              </Button>
              
              <Link href="/dashboard/strategies/unified">
                <Button 
                  size="sm"
                  className="bg-white/10 hover:bg-white/20 text-white border-white/20"
                >
                  Manage All
                  <ArrowRight className="ml-1 h-3 w-3" />
                </Button>
              </Link>
            </div>
          </CardHeader>

          <CardContent className="space-y-6">
            {/* Key Metrics */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-white/90">
                  {loading ? '...' : summary.totalStrategies}
                </div>
                <div className="text-sm text-white/60">Total Strategies</div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-green-400">
                  {loading ? '...' : summary.activeStrategies}
                </div>
                <div className="text-sm text-white/60">Active</div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-white/90">
                  {loading ? '...' : formatCurrency(summary.totalInvested)}
                </div>
                <div className="text-sm text-white/60">Total Invested</div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className={cn(
                  "text-2xl font-bold",
                  parseFloat(summary.totalProfitLoss) >= 0 ? "text-green-400" : "text-red-400"
                )}>
                  {loading ? '...' : formatCurrency(summary.totalProfitLoss)}
                </div>
                <div className="text-sm text-white/60">Total P&L</div>
              </div>
            </div>

            {/* Strategy Type Breakdown */}
            {strategyBreakdown.length > 0 && (
              <div className="space-y-4">
                <h3 className="text-lg font-semibold text-white/90">Strategy Breakdown</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {strategyBreakdown.map(({ type, info, count, activeCount }) => (
                    <div 
                      key={type}
                      className={cn(
                        "p-3 rounded-lg bg-gradient-to-r backdrop-blur-sm border border-white/10",
                        info.color
                      )}
                    >
                      <div className="flex items-center space-x-3">
                        <div className="text-2xl">{info.icon}</div>
                        <div className="flex-1">
                          <div className="text-sm font-medium text-white/90">
                            {info.name}
                          </div>
                          <div className="text-xs text-white/60">
                            {count} total, {activeCount} active
                          </div>
                        </div>
                        <Badge 
                          variant="outline" 
                          className={cn(
                            "text-xs border",
                            info.riskLevel === 'Low' && "border-green-500/30 text-green-400",
                            info.riskLevel === 'Medium' && "border-yellow-500/30 text-yellow-400",
                            info.riskLevel === 'High' && "border-red-500/30 text-red-400"
                          )}
                        >
                          {info.riskLevel}
                        </Badge>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Quick Actions */}
            <div className="flex flex-wrap gap-3">
              <Link href="/dashboard/strategies/unified">
                <Button 
                  variant="outline"
                  size="sm"
                  className="border-purple-500/30 text-purple-400 hover:bg-purple-500/10"
                >
                  <Target className="mr-2 h-4 w-4" />
                  Create Strategy
                </Button>
              </Link>
              
              <Link href="/dashboard/backtesting">
                <Button 
                  variant="outline"
                  size="sm"
                  className="border-cyan-500/30 text-cyan-400 hover:bg-cyan-500/10"
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Run Backtest
                </Button>
              </Link>
              
              <Link href="/dashboard/portfolio">
                <Button 
                  variant="outline"
                  size="sm"
                  className="border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/10"
                >
                  <TrendingUp className="mr-2 h-4 w-4" />
                  View Analytics
                </Button>
              </Link>
            </div>
          </CardContent>
        </div>
      </Card>

      {/* Performance Highlights */}
      {(summary.bestPerformer || summary.worstPerformer) && (
        <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
          <CardHeader>
            <CardTitle className="text-lg font-bold text-white/90">
              Performance Highlights
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {summary.bestPerformer && parseFloat(summary.bestPerformer.strategy.current_profit_loss || '0') > 0 && (
                <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-4">
                  <div className="flex items-center space-x-2 mb-2">
                    <TrendingUp className="h-5 w-5 text-green-400" />
                    <span className="font-medium text-green-400">Top Performer</span>
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="text-xl">{getStrategyInfo(summary.bestPerformer.type).icon}</span>
                      <span className="font-medium text-white/90">
                        {summary.bestPerformer.strategy.name}
                      </span>
                    </div>
                    <div className="text-green-400 font-bold">
                      {formatCurrency(summary.bestPerformer.strategy.current_profit_loss || '0')}
                    </div>
                    <div className="text-xs text-white/60">
                      {getStrategyInfo(summary.bestPerformer.type).name} • {summary.bestPerformer.strategy.asset_symbol}
                    </div>
                  </div>
                </div>
              )}

              {summary.worstPerformer && parseFloat(summary.worstPerformer.strategy.current_profit_loss || '0') < 0 && (
                <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                  <div className="flex items-center space-x-2 mb-2">
                    <TrendingDown className="h-5 w-5 text-red-400" />
                    <span className="font-medium text-red-400">Needs Attention</span>
                  </div>
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span className="text-xl">{getStrategyInfo(summary.worstPerformer.type).icon}</span>
                      <span className="font-medium text-white/90">
                        {summary.worstPerformer.strategy.name}
                      </span>
                    </div>
                    <div className="text-red-400 font-bold">
                      {formatCurrency(summary.worstPerformer.strategy.current_profit_loss || '0')}
                    </div>
                    <div className="text-xs text-white/60">
                      {getStrategyInfo(summary.worstPerformer.type).name} • {summary.worstPerformer.strategy.asset_symbol}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Empty State */}
      {summary.totalStrategies === 0 && !loading && (
        <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
          <CardContent className="p-8 text-center">
            <div className="text-4xl mb-4">🚀</div>
            <h3 className="text-xl font-semibold text-white/90 mb-2">
              Ready to Start Trading?
            </h3>
            <p className="text-white/60 mb-6">
              Create your first automated trading strategy and start building your portfolio
            </p>
            <div className="flex flex-col sm:flex-row gap-3 justify-center">
              <Link href="/dashboard/strategies/unified">
                <Button className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white">
                  <PlusCircle className="mr-2 h-4 w-4" />
                  Create Strategy
                </Button>
              </Link>
              <Link href="/dashboard/backtesting">
                <Button 
                  variant="outline"
                  className="border-white/20 text-white/80 hover:bg-white/10"
                >
                  <BarChart3 className="mr-2 h-4 w-4" />
                  Explore Backtesting
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
