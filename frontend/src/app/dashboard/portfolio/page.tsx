"use client"

import React, { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  TrendingUp,
  DollarSign,
  BarChart3,
  Activity,
  Target,
  RefreshCw,
  Award,
  AlertTriangle,
  PieChart,
  LineChart,
  PlusCircle
} from "lucide-react"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { 
  apiClient, 
  Strategy, 
  StrategyType
} from "@/lib/api"
import { 
  getStrategyInfo, 
  formatCurrency, 
  formatPercentage,
  getStrategyStatusColor
} from "@/lib/strategies"
import { useAuth } from "@/contexts/auth-context"
import { cn } from "@/lib/utils"

interface PortfolioStats {
  totalValue: number
  totalInvested: number
  totalPnL: number
  totalPnLPercentage: number
  dayChange: number
  dayChangePercentage: number
  bestPerformer: { strategy: Strategy; type: StrategyType; return: number } | null
  worstPerformer: { strategy: Strategy; type: StrategyType; return: number } | null
  assetAllocation: { [asset: string]: number }
  strategyAllocation: { [strategy: string]: number }
}

export default function PortfolioPage() {
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
 
  const [portfolioStats, setPortfolioStats] = useState<PortfolioStats>({
    totalValue: 0,
    totalInvested: 0,
    totalPnL: 0,
    totalPnLPercentage: 0,
    dayChange: 0,
    dayChangePercentage: 0,
    bestPerformer: null,
    worstPerformer: null,
    assetAllocation: {},
    strategyAllocation: {}
  })
  const [loading, setLoading] = useState(false)
  const [selectedTab, setSelectedTab] = useState('overview')

  const loadPortfolioData = React.useCallback(async () => {
    if (!isAuthenticated) return

    setLoading(true)
    try {
      // STEP 1: Get lightweight summary
      const strategySummary = await apiClient.getUserStrategySummary().catch(() => ({
        authenticated: false,
        strategy_types: [],
        total_strategies: 0,
        total_active: 0
      }))

      // If user has no strategies, return early
      if (!strategySummary.authenticated || strategySummary.total_strategies === 0) {
        setAllStrategies({
          dca: [],
          gridTrading: [],
          smaCrossover: [],
          rsi: [],
          macd: []
        })
        return
      }

      // STEP 2: Only load strategy data for types the user actually has
      const activeStrategyTypes = strategySummary.strategy_types.map(st => st.strategy_type)
      const strategiesRes = await apiClient.getStrategiesByTypes(activeStrategyTypes).catch(() => ({
        dca: undefined,
        gridTrading: undefined,
        smaCrossover: undefined,
        rsi: undefined,
        macd: undefined
      }))

      setAllStrategies({
        dca: strategiesRes.dca?.strategies || [],
        gridTrading: strategiesRes.gridTrading?.strategies || [],
        smaCrossover: strategiesRes.smaCrossover?.strategies || [],
        rsi: strategiesRes.rsi?.strategies || [],
        macd: strategiesRes.macd?.strategies || []
      })

      // Calculate portfolio statistics
      calculatePortfolioStats(strategiesRes)

    } catch (error) {
      console.error('Failed to load portfolio data:', error)
    } finally {
      setLoading(false)
    }
  }, [isAuthenticated])

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadPortfolioData()
    }
  }, [isAuthenticated, authLoading, loadPortfolioData])

  const calculatePortfolioStats = (strategiesRes: {
    dca?: { strategies: Strategy[] };
    gridTrading?: { strategies: Strategy[] };
    smaCrossover?: { strategies: Strategy[] };
    rsi?: { strategies: Strategy[] };
    macd?: { strategies: Strategy[] };
  }) => {
    const allStrategyArrays = [
      ...(strategiesRes.dca?.strategies || []).map((s: Strategy) => ({ strategy: s, type: 'dca' as StrategyType })),
      ...(strategiesRes.gridTrading?.strategies || []).map((s: Strategy) => ({ strategy: s, type: 'grid_trading' as StrategyType })),
      ...(strategiesRes.smaCrossover?.strategies || []).map((s: Strategy) => ({ strategy: s, type: 'sma_crossover' as StrategyType })),
      ...(strategiesRes.rsi?.strategies || []).map((s: Strategy) => ({ strategy: s, type: 'rsi' as StrategyType })),
      ...(strategiesRes.macd?.strategies || []).map((s: Strategy) => ({ strategy: s, type: 'macd' as StrategyType }))
    ]

    const totalInvested = allStrategyArrays.reduce((sum, { strategy }) => 
      sum + parseFloat(strategy.total_invested || '0'), 0
    )
    
    const totalPnL = allStrategyArrays.reduce((sum, { strategy }) => 
      sum + parseFloat(strategy.current_profit_loss || '0'), 0
    )

    const totalValue = totalInvested + totalPnL
    const totalPnLPercentage = totalInvested > 0 ? (totalPnL / totalInvested) * 100 : 0

    // Find best and worst performers
    let bestPerformer = null
    let worstPerformer = null
    let bestReturn = -Infinity
    let worstReturn = Infinity

    allStrategyArrays.forEach(({ strategy, type }) => {
      const invested = parseFloat(strategy.total_invested || '0')
      const pnl = parseFloat(strategy.current_profit_loss || '0')
      const returnPercentage = invested > 0 ? (pnl / invested) * 100 : 0
      
      if (returnPercentage > bestReturn && invested > 0) {
        bestReturn = returnPercentage
        bestPerformer = { strategy, type, return: returnPercentage }
      }
      if (returnPercentage < worstReturn && invested > 0) {
        worstReturn = returnPercentage
        worstPerformer = { strategy, type, return: returnPercentage }
      }
    })

    // Calculate asset allocation
    const assetAllocation: { [asset: string]: number } = {}
    allStrategyArrays.forEach(({ strategy }) => {
      const invested = parseFloat(strategy.total_invested || '0')
      if (invested > 0) {
        assetAllocation[strategy.asset_symbol] = (assetAllocation[strategy.asset_symbol] || 0) + invested
      }
    })

    // Calculate strategy type allocation
    const strategyAllocation: { [strategy: string]: number } = {}
    Object.entries(strategiesRes).forEach(([key, data]) => {
      if (data && data.strategies) {
        const invested = data.strategies.reduce((sum: number, s: Strategy) =>
          sum + parseFloat(s.total_invested || '0'), 0
        )
        if (invested > 0) {
          strategyAllocation[key] = invested
        }
      }
    })

    setPortfolioStats({
      totalValue,
      totalInvested,
      totalPnL,
      totalPnLPercentage,
      dayChange: 0, // Would need historical data
      dayChangePercentage: 0,
      bestPerformer,
      worstPerformer,
      assetAllocation,
      strategyAllocation
    })
  }

  const getAllStrategiesFlat = () => {
    return [
      ...allStrategies.dca.map(s => ({ strategy: s, type: 'dca' as StrategyType })),
      ...allStrategies.gridTrading.map(s => ({ strategy: s, type: 'grid_trading' as StrategyType })),
      ...allStrategies.smaCrossover.map(s => ({ strategy: s, type: 'sma_crossover' as StrategyType })),
      ...allStrategies.rsi.map(s => ({ strategy: s, type: 'rsi' as StrategyType })),
      ...allStrategies.macd.map(s => ({ strategy: s, type: 'macd' as StrategyType }))
    ]
  }

  if (authLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
        </div>
      </DashboardLayout>
    )
  }

  if (!isAuthenticated) {
    return (
      <DashboardLayout>
        <div className="text-center py-12">
          <h2 className="text-2xl font-bold text-white/90 mb-4">Authentication Required</h2>
          <p className="text-white/60">Please log in to access your portfolio analytics.</p>
        </div>
      </DashboardLayout>
    )
  }

  const allStrategiesFlat = getAllStrategiesFlat()

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-4 sm:space-y-0">
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">
              Portfolio Analytics
            </h1>
            <p className="text-white/60 mt-1">
              Comprehensive performance tracking across all your trading strategies
            </p>
          </div>
          
          <Button
            onClick={loadPortfolioData}
            variant="outline"
            disabled={loading}
            className="border-white/20 text-white/80 hover:bg-white/10"
          >
            <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
            Refresh Data
          </Button>
        </div>

        {/* Portfolio Summary */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gradient-to-br from-green-500/20 to-emerald-500/20 backdrop-blur-xl border border-white/10">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <CardContent className="relative z-10 p-6">
              <div className="flex items-center space-x-2">
                <DollarSign className="h-8 w-8 text-green-400" />
                <div>
                  <p className="text-2xl font-bold text-white">{formatCurrency(portfolioStats.totalValue)}</p>
                  <p className="text-sm text-white/60">Total Portfolio Value</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-blue-500/20 to-cyan-500/20 backdrop-blur-xl border border-white/10">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <CardContent className="relative z-10 p-6">
              <div className="flex items-center space-x-2">
                <Target className="h-8 w-8 text-blue-400" />
                <div>
                  <p className="text-2xl font-bold text-white">{formatCurrency(portfolioStats.totalInvested)}</p>
                  <p className="text-sm text-white/60">Total Invested</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-purple-500/20 to-pink-500/20 backdrop-blur-xl border border-white/10">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <CardContent className="relative z-10 p-6">
              <div className="flex items-center space-x-2">
                <TrendingUp className={cn(
                  "h-8 w-8",
                  portfolioStats.totalPnL >= 0 ? "text-green-400" : "text-red-400"
                )} />
                <div>
                  <p className={cn(
                    "text-2xl font-bold",
                    portfolioStats.totalPnL >= 0 ? "text-green-400" : "text-red-400"
                  )}>
                    {formatCurrency(portfolioStats.totalPnL)}
                  </p>
                  <p className="text-sm text-white/60">
                    Total P&L ({formatPercentage(portfolioStats.totalPnLPercentage)})
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gradient-to-br from-yellow-500/20 to-orange-500/20 backdrop-blur-xl border border-white/10">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <CardContent className="relative z-10 p-6">
              <div className="flex items-center space-x-2">
                <Activity className="h-8 w-8 text-yellow-400" />
                <div>
                  <p className="text-2xl font-bold text-white">{allStrategiesFlat.length}</p>
                  <p className="text-sm text-white/60">
                    Active Strategies ({allStrategiesFlat.filter(({ strategy }) => strategy.status?.toLowerCase() === 'active').length})
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Performance Highlights */}
        {(portfolioStats.bestPerformer || portfolioStats.worstPerformer) && (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {portfolioStats.bestPerformer && (
              <Card className="bg-gradient-to-br from-green-500/20 to-emerald-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <div className="relative z-10">
                  <CardHeader>
                    <div className="flex items-center space-x-2">
                      <Award className="h-5 w-5 text-green-400" />
                      <CardTitle className="text-lg font-bold text-green-400">
                        Top Performer
                      </CardTitle>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex items-center space-x-3">
                      <div className="text-2xl">
                        {getStrategyInfo(portfolioStats.bestPerformer.type).icon}
                      </div>
                      <div>
                        <p className="font-semibold text-white/90">
                          {portfolioStats.bestPerformer.strategy.name}
                        </p>
                        <p className="text-sm text-white/60">
                          {getStrategyInfo(portfolioStats.bestPerformer.type).name} • {portfolioStats.bestPerformer.strategy.asset_symbol}
                        </p>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <p className="text-white/60">Return</p>
                        <p className="text-green-400 font-bold">
                          {formatPercentage(portfolioStats.bestPerformer.return)}
                        </p>
                      </div>
                      <div>
                        <p className="text-white/60">P&L</p>
                        <p className="text-green-400 font-bold">
                          {formatCurrency(portfolioStats.bestPerformer.strategy.current_profit_loss || '0')}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </div>
              </Card>
            )}

            {portfolioStats.worstPerformer && portfolioStats.worstPerformer.return < 0 && (
              <Card className="bg-gradient-to-br from-red-500/20 to-pink-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <div className="relative z-10">
                  <CardHeader>
                    <div className="flex items-center space-x-2">
                      <AlertTriangle className="h-5 w-5 text-red-400" />
                      <CardTitle className="text-lg font-bold text-red-400">
                        Needs Attention
                      </CardTitle>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex items-center space-x-3">
                      <div className="text-2xl">
                        {getStrategyInfo(portfolioStats.worstPerformer.type).icon}
                      </div>
                      <div>
                        <p className="font-semibold text-white/90">
                          {portfolioStats.worstPerformer.strategy.name}
                        </p>
                        <p className="text-sm text-white/60">
                          {getStrategyInfo(portfolioStats.worstPerformer.type).name} • {portfolioStats.worstPerformer.strategy.asset_symbol}
                        </p>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <p className="text-white/60">Return</p>
                        <p className="text-red-400 font-bold">
                          {formatPercentage(portfolioStats.worstPerformer.return)}
                        </p>
                      </div>
                      <div>
                        <p className="text-white/60">P&L</p>
                        <p className="text-red-400 font-bold">
                          {formatCurrency(portfolioStats.worstPerformer.strategy.current_profit_loss || '0')}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </div>
              </Card>
            )}
          </div>
        )}

        {/* Detailed Analytics */}
        <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
          <CardContent className="p-0">
            <Tabs value={selectedTab} onValueChange={setSelectedTab} className="w-full">
              <TabsList className="grid w-full grid-cols-4 bg-transparent border-b border-white/10">
                <TabsTrigger value="overview" className="data-[state=active]:bg-white/10">
                  Overview
                </TabsTrigger>
                <TabsTrigger value="allocation" className="data-[state=active]:bg-white/10">
                  Allocation
                </TabsTrigger>
                <TabsTrigger value="performance" className="data-[state=active]:bg-white/10">
                  Performance
                </TabsTrigger>
                <TabsTrigger value="strategies" className="data-[state=active]:bg-white/10">
                  Strategies
                </TabsTrigger>
              </TabsList>
              
              <TabsContent value="overview" className="p-6 space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {/* Asset Allocation */}
                  <div className="space-y-4">
                    <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                      <PieChart className="h-5 w-5" />
                      <span>Asset Allocation</span>
                    </h3>
                    {Object.entries(portfolioStats.assetAllocation).length > 0 ? (
                      <div className="space-y-2">
                        {Object.entries(portfolioStats.assetAllocation)
                          .sort(([,a], [,b]) => b - a)
                          .map(([asset, amount]) => {
                            const percentage = portfolioStats.totalInvested > 0 
                              ? (amount / portfolioStats.totalInvested) * 100 
                              : 0
                            return (
                              <div key={asset} className="flex items-center justify-between p-3 bg-white/5 rounded-lg">
                                <span className="font-medium text-white/90">{asset}</span>
                                <div className="text-right">
                                  <div className="text-white/90 font-medium">{formatCurrency(amount)}</div>
                                  <div className="text-xs text-white/60">{percentage.toFixed(1)}%</div>
                                </div>
                              </div>
                            )
                          })}
                      </div>
                    ) : (
                      <p className="text-white/60 text-center py-8">No asset allocation data</p>
                    )}
                  </div>

                  {/* Strategy Type Allocation */}
                  <div className="space-y-4">
                    <h3 className="text-lg font-semibold text-white/90 flex items-center space-x-2">
                      <BarChart3 className="h-5 w-5" />
                      <span>Strategy Allocation</span>
                    </h3>
                    {Object.entries(portfolioStats.strategyAllocation).length > 0 ? (
                      <div className="space-y-2">
                        {Object.entries(portfolioStats.strategyAllocation)
                          .sort(([,a], [,b]) => b - a)
                          .map(([strategyKey, amount]) => {
                            const percentage = portfolioStats.totalInvested > 0 
                              ? (amount / portfolioStats.totalInvested) * 100 
                              : 0
                            const type = strategyKey === 'gridTrading' ? 'grid_trading' : 
                                       strategyKey === 'smaCrossover' ? 'sma_crossover' : strategyKey as StrategyType
                            const info = getStrategyInfo(type)
                            return (
                              <div key={strategyKey} className="flex items-center justify-between p-3 bg-white/5 rounded-lg">
                                <div className="flex items-center space-x-2">
                                  <span className="text-lg">{info.icon}</span>
                                  <span className="font-medium text-white/90">{info.name}</span>
                                </div>
                                <div className="text-right">
                                  <div className="text-white/90 font-medium">{formatCurrency(amount)}</div>
                                  <div className="text-xs text-white/60">{percentage.toFixed(1)}%</div>
                                </div>
                              </div>
                            )
                          })}
                      </div>
                    ) : (
                      <p className="text-white/60 text-center py-8">No strategy allocation data</p>
                    )}
                  </div>
                </div>
              </TabsContent>
              
              <TabsContent value="allocation" className="p-6 space-y-6">
                <div className="text-center">
                  <h3 className="text-xl font-semibold text-white/90 mb-4">Allocation Breakdown</h3>
                  
                  {allStrategiesFlat.length === 0 ? (
                    <div className="py-12">
                      <PieChart className="h-16 w-16 text-white/40 mx-auto mb-4" />
                      <p className="text-white/60">No strategies to analyze</p>
                      <p className="text-sm text-white/50 mt-2">Create strategies to see allocation breakdown</p>
                    </div>
                  ) : (
                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                      {/* Asset Breakdown */}
                      <div className="space-y-4">
                        <h4 className="text-lg font-medium text-white/80">By Asset</h4>
                        <div className="space-y-3">
                          {Object.entries(portfolioStats.assetAllocation).map(([asset, amount]) => {
                            const percentage = portfolioStats.totalInvested > 0 ? (amount / portfolioStats.totalInvested) * 100 : 0
                            return (
                              <div key={asset} className="bg-white/5 rounded-lg p-4">
                                <div className="flex justify-between items-center mb-2">
                                  <span className="font-semibold text-white/90">{asset}</span>
                                  <span className="text-white/70">{percentage.toFixed(1)}%</span>
                                </div>
                                <div className="w-full bg-white/10 rounded-full h-2">
                                  <div 
                                    className="bg-gradient-to-r from-purple-500 to-pink-500 h-2 rounded-full transition-all duration-300"
                                    style={{ width: `${percentage}%` }}
                                  />
                                </div>
                                <div className="text-sm text-white/60 mt-1">
                                  {formatCurrency(amount)}
                                </div>
                              </div>
                            )
                          })}
                        </div>
                      </div>

                      {/* Strategy Type Breakdown */}
                      <div className="space-y-4">
                        <h4 className="text-lg font-medium text-white/80">By Strategy Type</h4>
                        <div className="space-y-3">
                          {Object.entries(portfolioStats.strategyAllocation).map(([strategyKey, amount]) => {
                            const percentage = portfolioStats.totalInvested > 0 ? (amount / portfolioStats.totalInvested) * 100 : 0
                            const type = strategyKey === 'gridTrading' ? 'grid_trading' : 
                                       strategyKey === 'smaCrossover' ? 'sma_crossover' : strategyKey as StrategyType
                            const info = getStrategyInfo(type)
                            return (
                              <div key={strategyKey} className="bg-white/5 rounded-lg p-4">
                                <div className="flex justify-between items-center mb-2">
                                  <div className="flex items-center space-x-2">
                                    <span className="text-lg">{info.icon}</span>
                                    <span className="font-semibold text-white/90">{info.name}</span>
                                  </div>
                                  <span className="text-white/70">{percentage.toFixed(1)}%</span>
                                </div>
                                <div className="w-full bg-white/10 rounded-full h-2">
                                  <div 
                                    className={cn("h-2 rounded-full transition-all duration-300 bg-gradient-to-r", info.color.replace('from-', 'from-').replace('to-', 'to-').replace('/20', ''))}
                                    style={{ width: `${percentage}%` }}
                                  />
                                </div>
                                <div className="text-sm text-white/60 mt-1">
                                  {formatCurrency(amount)} • {info.riskLevel} Risk
                                </div>
                              </div>
                            )
                          })}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </TabsContent>
              
              <TabsContent value="performance" className="p-6 space-y-6">
                <h3 className="text-xl font-semibold text-white/90 mb-4">Performance Analysis</h3>
                
                {allStrategiesFlat.length === 0 ? (
                  <div className="text-center py-12">
                    <LineChart className="h-16 w-16 text-white/40 mx-auto mb-4" />
                    <p className="text-white/60">No performance data available</p>
                    <p className="text-sm text-white/50 mt-2">Create and run strategies to see performance analytics</p>
                  </div>
                ) : (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    {/* Risk Metrics */}
                    <div className="space-y-4">
                      <h4 className="text-lg font-medium text-white/80">Risk Assessment</h4>
                      <div className="space-y-3">
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Portfolio Risk Level</span>
                          <Badge className={cn(
                            portfolioStats.totalPnLPercentage > 10 ? "border-green-500/30 text-green-400" :
                            portfolioStats.totalPnLPercentage > 0 ? "border-yellow-500/30 text-yellow-400" :
                            "border-red-500/30 text-red-400"
                          )}>
                            {portfolioStats.totalPnLPercentage > 10 ? "Conservative" :
                             portfolioStats.totalPnLPercentage > 0 ? "Moderate" : "Aggressive"}
                          </Badge>
                        </div>
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Diversification</span>
                          <span className="text-white/90">
                            {Object.keys(portfolioStats.assetAllocation).length} assets, {Object.keys(portfolioStats.strategyAllocation).length} strategy types
                          </span>
                        </div>
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Active Management</span>
                          <span className="text-white/90">
                            {allStrategiesFlat.filter(({ strategy }) => strategy.status?.toLowerCase() === 'active').length} / {allStrategiesFlat.length} strategies
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Returns Analysis */}
                    <div className="space-y-4">
                      <h4 className="text-lg font-medium text-white/80">Returns Analysis</h4>
                      <div className="space-y-3">
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Total Return</span>
                          <span className={cn(
                            "font-medium",
                            portfolioStats.totalPnL >= 0 ? "text-green-400" : "text-red-400"
                          )}>
                            {formatPercentage(portfolioStats.totalPnLPercentage)}
                          </span>
                        </div>
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Best Strategy</span>
                          <span className="text-green-400 font-medium">
                            {portfolioStats.bestPerformer 
                              ? formatPercentage(portfolioStats.bestPerformer.return)
                              : 'N/A'
                            }
                          </span>
                        </div>
                        <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                          <span className="text-white/70">Strategy Count</span>
                          <span className="text-white/90 font-medium">
                            {allStrategiesFlat.length} total
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </TabsContent>
              
              <TabsContent value="strategies" className="p-6">
                <h3 className="text-xl font-semibold text-white/90 mb-6">Strategy Details</h3>
                
                {allStrategiesFlat.length === 0 ? (
                  <div className="text-center py-12">
                    <Target className="h-16 w-16 text-white/40 mx-auto mb-4" />
                    <p className="text-white/60">No strategies found</p>
                    <p className="text-sm text-white/50 mt-2">Create your first strategy to start tracking performance</p>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {allStrategiesFlat.map(({ strategy, type }) => {
                      const info = getStrategyInfo(type)
                      const invested = parseFloat(strategy.total_invested || '0')
                      const pnl = parseFloat(strategy.current_profit_loss || '0')
                      const returnPercentage = invested > 0 ? (pnl / invested) * 100 : 0
                      
                      return (
                        <Card key={`${type}-${strategy.id}`} className={cn(
                          "bg-gradient-to-r backdrop-blur-sm border border-white/10",
                          info.color
                        )}>
                          <CardContent className="p-4">
                            <div className="flex items-center justify-between">
                              <div className="flex items-center space-x-4">
                                <div className="text-2xl">{info.icon}</div>
                                <div>
                                  <h4 className="font-semibold text-white/90">{strategy.name}</h4>
                                  <div className="flex items-center space-x-2 text-sm text-white/60">
                                    <span>{info.name}</span>
                                    <span>•</span>
                                    <span>{strategy.asset_symbol}</span>
                                    <Badge className={getStrategyStatusColor(strategy.status)}>
                                      {strategy.status}
                                    </Badge>
                                  </div>
                                </div>
                              </div>
                              
                              <div className="text-right space-y-1">
                                <div className="text-sm text-white/60">Invested</div>
                                <div className="font-medium text-white/90">{formatCurrency(invested)}</div>
                                <div className={cn(
                                  "text-sm font-medium",
                                  pnl >= 0 ? "text-green-400" : "text-red-400"
                                )}>
                                  {formatCurrency(pnl)} ({formatPercentage(returnPercentage)})
                                </div>
                              </div>
                            </div>
                          </CardContent>
                        </Card>
                      )
                    })}
                  </div>
                )}
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>

        {/* Empty State */}
        {allStrategiesFlat.length === 0 && (
          <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
            <CardContent className="p-12 text-center">
              <div className="text-4xl mb-4">📊</div>
              <h3 className="text-xl font-semibold text-white/90 mb-2">
                No Portfolio Data Yet
              </h3>
              <p className="text-white/60 mb-6">
                Create trading strategies to start building your portfolio analytics
              </p>
              <Button 
                asChild
                className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500"
              >
                <a href="/dashboard/strategies/unified">
                  <PlusCircle className="mr-2 h-4 w-4" />
                  Create Your First Strategy
                </a>
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    </DashboardLayout>
  )
}
