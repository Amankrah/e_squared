"use client"

import { useState, useEffect, useCallback, useMemo } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { 
  PlusCircle, 
  TrendingUp, 
  DollarSign, 
  Activity, 
  Filter,
  Search,
  RefreshCw
} from "lucide-react"
import { Input } from "@/components/ui/input"
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { StrategyCard } from "@/components/strategies/strategy-card"
import { StrategyTypeSelector } from "@/components/strategies/strategy-type-selector"
import { GridTradingConfig } from "@/components/strategies/config/grid-trading-config"
import { SMAConfig } from "@/components/strategies/config/sma-crossover-config"
import { DCAConfig } from "@/components/strategies/config/dca-config"
import {
  apiClient,
  StrategyType,
  Strategy,
  CreateGridTradingStrategyRequest,
  CreateSMACrossoverStrategyRequest,
  CreateDCAStrategyRequest,
  StrategyConfig
} from "@/lib/api"
import { 
  getStrategyInfo, 
  getStrategyDisplayName, 
  formatCurrency,
  STRATEGY_INFO 
} from "@/lib/strategies"
import { useAuth } from "@/contexts/auth-context"
import { cn } from "@/lib/utils"

type ViewMode = 'overview' | 'create' | 'configure'

export default function UnifiedStrategiesPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [viewMode, setViewMode] = useState<ViewMode>('overview')
  const [selectedStrategyType, setSelectedStrategyType] = useState<StrategyType>()
  const [editingStrategy, setEditingStrategy] = useState<{ strategy: Strategy, type: StrategyType } | null>(null)
  const [allStrategies, setAllStrategies] = useState<{
    dca: Strategy[]
    gridTrading: Strategy[]
    smaCrossover: Strategy[]
  }>({
    dca: [],
    gridTrading: [],
    smaCrossover: []
  })

  const [loading, setLoading] = useState(false)
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [typeFilter, setTypeFilter] = useState<string>('all')
  
  // Summary stats
  const [summaryStats, setSummaryStats] = useState({
    totalStrategies: 0,
    activeStrategies: 0,
    totalInvested: '0',
    totalProfitLoss: '0'
  })

  const loadAllStrategies = useCallback(async () => {
    if (!isAuthenticated) return

    setLoading(true)
    try {
      // STEP 1: Get lightweight summary first
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
          smaCrossover: []
        })
        return
      }

      // STEP 2: Only load strategy data for types the user actually has
      const activeStrategyTypes = strategySummary.strategy_types.map(st => st.strategy_type)
      const results = await apiClient.getStrategiesByTypes(activeStrategyTypes).catch(() => ({}))

      // Type guard to ensure results has the expected structure
      const hasValidResults = results && typeof results === 'object' && !Array.isArray(results)

      setAllStrategies({
        dca: hasValidResults && 'dca' in results ? results.dca?.strategies || [] : [],
        gridTrading: hasValidResults && 'gridTrading' in results ? results.gridTrading?.strategies || [] : [],
        smaCrossover: hasValidResults && 'smaCrossover' in results ? results.smaCrossover?.strategies || [] : []
      })

      // Calculate summary stats from only loaded data
      const allStrategyArrays = [
        hasValidResults && 'dca' in results ? results.dca?.strategies || [] : [],
        hasValidResults && 'gridTrading' in results ? results.gridTrading?.strategies || [] : [],
        hasValidResults && 'smaCrossover' in results ? results.smaCrossover?.strategies || [] : []
      ]
      
      const totalStrategies = allStrategyArrays.reduce((sum, arr) => sum + arr.length, 0)
      const activeStrategies = allStrategyArrays.reduce((sum, arr) => 
        sum + arr.filter(s => s.status?.toLowerCase() === 'active').length, 0
      )
      
      const totalInvested = allStrategyArrays.reduce((sum, arr) => 
        sum + arr.reduce((innerSum, s) => innerSum + parseFloat(s.total_invested || '0'), 0), 0
      )
      
      const totalProfitLoss = allStrategyArrays.reduce((sum, arr) => 
        sum + arr.reduce((innerSum, s) => innerSum + parseFloat(s.current_profit_loss || '0'), 0), 0
      )

      setSummaryStats({
        totalStrategies,
        activeStrategies,
        totalInvested: totalInvested.toString(),
        totalProfitLoss: totalProfitLoss.toString()
      })

    } catch (error) {
      console.error('Failed to load strategies:', error)
    } finally {
      setLoading(false)
    }
  }, [isAuthenticated])

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadAllStrategies()
    }
  }, [isAuthenticated, authLoading, loadAllStrategies])

  const handleCreateStrategy = useCallback((type: StrategyType) => {
    setSelectedStrategyType(type)
    setViewMode('configure')
  }, [])

  const handleCreateGridTradingStrategy = useCallback(async (data: CreateGridTradingStrategyRequest) => {
    try {
      if (editingStrategy) {
        // Update existing strategy
        await apiClient.updateGridTradingStrategy(editingStrategy.strategy.id, {
          name: data.name,
          status: data.status,
          config: data.config
        })
      } else {
        // Create new strategy
        await apiClient.createGridTradingStrategy(data)
      }
      await loadAllStrategies()
      setViewMode('overview')
      setSelectedStrategyType(undefined)
      setEditingStrategy(null)
    } catch (error) {
      console.error('Failed to create/update grid trading strategy:', error)
      throw error
    }
  }, [loadAllStrategies, editingStrategy])

  const handleCreateSMACrossoverStrategy = useCallback(async (data: CreateSMACrossoverStrategyRequest) => {
    try {
      if (editingStrategy) {
        // Update existing strategy
        await apiClient.updateSMACrossoverStrategy(editingStrategy.strategy.id, {
          name: data.name,
          status: data.status,
          config: data.config
        })
      } else {
        // Create new strategy
        await apiClient.createSMACrossoverStrategy(data)
      }
      await loadAllStrategies()
      setViewMode('overview')
      setSelectedStrategyType(undefined)
      setEditingStrategy(null)
    } catch (error) {
      console.error('Failed to create/update SMA crossover strategy:', error)
      throw error
    }
  }, [loadAllStrategies, editingStrategy])

  const handleCreateDCAStrategy = useCallback(async (data: CreateDCAStrategyRequest) => {
    try {
      if (editingStrategy) {
        // Update existing strategy
        await apiClient.updateDCAStrategy(editingStrategy.strategy.id, {
          name: data.name,
          status: data.status,
          config: data.config
        })
      } else {
        // Create new strategy
        await apiClient.createDCAStrategy(data)
      }
      await loadAllStrategies()
      setViewMode('overview')
      setSelectedStrategyType(undefined)
      setEditingStrategy(null)
    } catch (error) {
      console.error('Failed to create/update DCA strategy:', error)
      throw error
    }
  }, [loadAllStrategies, editingStrategy])

  const handleBacktestStrategy = useCallback((type: StrategyType, config: StrategyConfig, name: string, assetSymbol: string) => {
    // Redirect to backtesting lab with strategy configuration
    const strategyData = { type, config, name, assetSymbol }
    localStorage.setItem('pendingBacktestStrategy', JSON.stringify(strategyData))
    window.location.href = '/dashboard/backtesting'
  }, [])

  // Memoize backtest handlers to prevent re-renders
  const dcaBacktestHandler = useCallback((config: StrategyConfig, name: string, assetSymbol: string) =>
    handleBacktestStrategy('dca', config, name, assetSymbol), [handleBacktestStrategy])

  const gridTradingBacktestHandler = useCallback((config: StrategyConfig, name: string, assetSymbol: string) =>
    handleBacktestStrategy('grid_trading', config, name, assetSymbol), [handleBacktestStrategy])

  const smaBacktestHandler = useCallback((config: StrategyConfig, name: string, assetSymbol: string) =>
    handleBacktestStrategy('sma_crossover', config, name, assetSymbol), [handleBacktestStrategy])

  // Memoize cancel handlers to prevent re-renders
  const handleCancelStrategy = useCallback(() => {
    setViewMode('overview')
    setSelectedStrategyType(undefined)
    setEditingStrategy(null)
  }, [])


  const handleEditStrategy = (strategy: Strategy, type: StrategyType) => {
    setEditingStrategy({ strategy, type })
    setSelectedStrategyType(type)
    setViewMode('configure')
  }

  const handleDeleteStrategy = async (strategy: Strategy, type: StrategyType) => {
    if (!confirm(`Are you sure you want to delete "${strategy.name}"?`)) return

    try {
      switch (type) {
        case 'dca':
          await apiClient.deleteDCAStrategy(strategy.id)
          break
        case 'grid_trading':
          await apiClient.deleteGridTradingStrategy(strategy.id)
          break
        case 'sma_crossover':
          await apiClient.deleteSMACrossoverStrategy(strategy.id)
          break
      }
      await loadAllStrategies()
    } catch (error) {
      console.error('Failed to delete strategy:', error)
    }
  }

  // Get all strategies as a flat array for filtering
  const getAllStrategiesFlat = (): Array<{ strategy: Strategy; type: StrategyType }> => {
    return [
      ...allStrategies.dca.map(s => ({ strategy: s, type: 'dca' as StrategyType })),
      ...allStrategies.gridTrading.map(s => ({ strategy: s, type: 'grid_trading' as StrategyType })),
      ...allStrategies.smaCrossover.map(s => ({ strategy: s, type: 'sma_crossover' as StrategyType }))
    ]
  }

  // Filter strategies based on search and filters
  const filteredStrategies = getAllStrategiesFlat().filter(({ strategy, type }) => {
    const matchesSearch = strategy.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         strategy.asset_symbol.toLowerCase().includes(searchTerm.toLowerCase())
    const matchesStatus = statusFilter === 'all' || strategy.status?.toLowerCase() === statusFilter
    const matchesType = typeFilter === 'all' || type === typeFilter
    
    return matchesSearch && matchesStatus && matchesType
  })

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
          <p className="text-white/60">Please log in to access your strategies.</p>
        </div>
      </DashboardLayout>
    )
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-4 sm:space-y-0">
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">
              Trading Strategies
            </h1>
            <p className="text-white/60 mt-1">
              Manage all your automated trading strategies in one place
            </p>
          </div>
          
          <div className="flex items-center space-x-3">
            <Button
              onClick={loadAllStrategies}
              variant="outline"
              size="sm"
              disabled={loading}
              className="border-white/20 text-white/80 hover:bg-white/10"
            >
              <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
              Refresh
            </Button>
            
            <Button
              onClick={() => setViewMode('create')}
              className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500"
            >
              <PlusCircle className="h-4 w-4 mr-2" />
              New Strategy
            </Button>
          </div>
        </div>

        {viewMode === 'overview' && (
          <>
            {/* Summary Stats */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              <Card className="bg-gradient-to-br from-blue-500/20 to-cyan-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <Activity className="h-8 w-8 text-blue-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">{summaryStats.totalStrategies}</p>
                      <p className="text-sm text-white/60">Total Strategies</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-gradient-to-br from-green-500/20 to-emerald-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <TrendingUp className="h-8 w-8 text-green-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">{summaryStats.activeStrategies}</p>
                      <p className="text-sm text-white/60">Active</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-gradient-to-br from-yellow-500/20 to-orange-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <DollarSign className="h-8 w-8 text-yellow-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">{formatCurrency(summaryStats.totalInvested)}</p>
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
                      parseFloat(summaryStats.totalProfitLoss) >= 0 ? "text-green-400" : "text-red-400"
                    )} />
                    <div>
                      <p className={cn(
                        "text-2xl font-bold",
                        parseFloat(summaryStats.totalProfitLoss) >= 0 ? "text-green-400" : "text-red-400"
                      )}>
                        {formatCurrency(summaryStats.totalProfitLoss)}
                      </p>
                      <p className="text-sm text-white/60">Total P&L</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Filters */}
            <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
              <CardContent className="p-6">
                <div className="flex flex-col sm:flex-row gap-4">
                  <div className="flex-1">
                    <div className="relative">
                      <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-white/40" />
                      <Input
                        placeholder="Search strategies..."
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        className="pl-10 bg-white/10 border-white/20 text-white placeholder:text-white/50"
                      />
                    </div>
                  </div>
                  
                  <Select value={statusFilter} onValueChange={setStatusFilter}>
                    <SelectTrigger className="w-full sm:w-[150px] bg-white/10 border-white/20 text-white">
                      <SelectValue placeholder="Status" />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 border-white/20 text-white">
                      <SelectItem value="all" className="text-white hover:bg-white/10">All Status</SelectItem>
                      <SelectItem value="active" className="text-white hover:bg-white/10">Active</SelectItem>
                      <SelectItem value="paused" className="text-white hover:bg-white/10">Paused</SelectItem>
                      <SelectItem value="stopped" className="text-white hover:bg-white/10">Stopped</SelectItem>
                    </SelectContent>
                  </Select>

                  <Select value={typeFilter} onValueChange={setTypeFilter}>
                    <SelectTrigger className="w-full sm:w-[150px] bg-white/10 border-white/20 text-white">
                      <SelectValue placeholder="Type" />
                    </SelectTrigger>
                    <SelectContent className="bg-black/90 border-white/20 text-white">
                      <SelectItem value="all" className="text-white hover:bg-white/10">All Types</SelectItem>
                      <SelectItem value="dca" className="text-white hover:bg-white/10">DCA</SelectItem>
                      <SelectItem value="grid_trading" className="text-white hover:bg-white/10">Grid Trading</SelectItem>
                      <SelectItem value="sma_crossover" className="text-white hover:bg-white/10">SMA Crossover</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>

            {/* Strategies Grid */}
            {loading ? (
              <div className="flex items-center justify-center py-12">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
              </div>
            ) : filteredStrategies.length > 0 ? (
              <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                {filteredStrategies.map(({ strategy, type }) => (
                  <StrategyCard
                    key={`${type}-${strategy.id}`}
                    strategy={strategy}
                    strategyType={type}
                    onEdit={(s) => handleEditStrategy(s, type)}
                    onDelete={(s) => handleDeleteStrategy(s, type)}
                    className="h-full"
                  />
                ))}
              </div>
            ) : (
              <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
                <CardContent className="p-12 text-center">
                  <div className="text-4xl mb-4">📊</div>
                  <h3 className="text-xl font-semibold text-white/90 mb-2">
                    {searchTerm || statusFilter !== 'all' || typeFilter !== 'all' 
                      ? 'No strategies match your filters' 
                      : 'No strategies yet'
                    }
                  </h3>
                  <p className="text-white/60 mb-6">
                    {searchTerm || statusFilter !== 'all' || typeFilter !== 'all'
                      ? 'Try adjusting your search criteria or filters'
                      : 'Create your first automated trading strategy to get started'
                    }
                  </p>
                  {(!searchTerm && statusFilter === 'all' && typeFilter === 'all') && (
                    <Button
                      onClick={() => setViewMode('create')}
                      className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500"
                    >
                      <PlusCircle className="h-4 w-4 mr-2" />
                      Create Strategy
                    </Button>
                  )}
                </CardContent>
              </Card>
            )}
          </>
        )}

        {viewMode === 'create' && (
          <StrategyTypeSelector
            onTypeSelect={handleCreateStrategy}
            className="max-w-6xl mx-auto"
          />
        )}

        {viewMode === 'configure' && selectedStrategyType && (
          <div className="max-w-4xl mx-auto">
            {selectedStrategyType === 'dca' && (
              <DCAConfig
                initialData={editingStrategy?.strategy}
                onSubmit={handleCreateDCAStrategy}
                onCancel={handleCancelStrategy}
                onBacktest={dcaBacktestHandler}
              />
            )}

            {selectedStrategyType === 'grid_trading' && (
              <GridTradingConfig
                initialData={editingStrategy?.strategy}
                onSubmit={handleCreateGridTradingStrategy}
                onCancel={handleCancelStrategy}
                onBacktest={gridTradingBacktestHandler}
              />
            )}

            {selectedStrategyType === 'sma_crossover' && (
              <SMAConfig
                initialData={editingStrategy?.strategy}
                onSubmit={handleCreateSMACrossoverStrategy}
                onCancel={handleCancelStrategy}
                onBacktest={smaBacktestHandler}
              />
            )}

            {/* Fallback for strategies without config components */}
            {!['dca', 'grid_trading', 'sma_crossover'].includes(selectedStrategyType) && (
              <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
                <CardContent className="p-12 text-center">
                  <div className="text-4xl mb-4">🚧</div>
                  <h3 className="text-xl font-semibold text-white/90 mb-2">
                    {getStrategyDisplayName(selectedStrategyType)} Configuration
                  </h3>
                  <p className="text-white/60 mb-6">
                    Configuration component for {getStrategyDisplayName(selectedStrategyType)} is coming soon!
                  </p>
                  <Button
                    onClick={handleCancelStrategy}
                    variant="outline"
                    className="border-white/20 text-white/80 hover:bg-white/10"
                  >
                    Back to Overview
                  </Button>
                </CardContent>
              </Card>
            )}
          </div>
        )}

      </div>
    </DashboardLayout>
  )
}
