"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { StrategyType, Strategy } from "@/lib/api"
import { getStrategyInfo, formatCurrency, RISK_LEVEL_COLORS } from "@/lib/strategies"
import { ArrowRight, Plus } from "lucide-react"
import Link from "next/link"
import { cn } from "@/lib/utils"

interface StrategyTypeStatsProps {
  strategies: {
    dca: Strategy[]
    gridTrading: Strategy[]
    smaCrossover: Strategy[]
    rsi: Strategy[]
    macd: Strategy[]
  }
  className?: string
}

export function StrategyTypeStats({ strategies, className }: StrategyTypeStatsProps) {
  const strategyTypes: StrategyType[] = ['dca', 'grid_trading', 'sma_crossover', 'rsi', 'macd']

  const getTypeStats = (type: StrategyType) => {
    const key = type === 'grid_trading' ? 'gridTrading' : 
                type === 'sma_crossover' ? 'smaCrossover' : type
    const strategyList = strategies[key as keyof typeof strategies] as Strategy[]
    
    const totalCount = strategyList.length
    const activeCount = strategyList.filter(s => s.status?.toLowerCase() === 'active').length
    const totalInvested = strategyList.reduce((sum, s) => sum + parseFloat(s.total_invested || '0'), 0)
    const totalPnL = strategyList.reduce((sum, s) => sum + parseFloat(s.current_profit_loss || '0'), 0)
    
    return {
      totalCount,
      activeCount,
      totalInvested,
      totalPnL,
      hasStrategies: totalCount > 0
    }
  }

  return (
    <div className={cn("space-y-4", className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-white/90">Strategy Types</h3>
        <Link href="/dashboard/strategies/unified">
          <Button variant="outline" size="sm" className="border-white/20 text-white/80 hover:bg-white/10">
            View All
            <ArrowRight className="ml-1 h-3 w-3" />
          </Button>
        </Link>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        {strategyTypes.map((type) => {
          const info = getStrategyInfo(type)
          const stats = getTypeStats(type)
          
          return (
            <Card 
              key={type}
              className={cn(
                "group relative overflow-hidden transition-all duration-300 hover:scale-[1.02]",
                "bg-gradient-to-br backdrop-blur-xl border border-white/10",
                "hover:border-white/20 hover:shadow-lg",
                info.color
              )}
            >
              <div className="absolute inset-0 bg-white/5 backdrop-blur-sm" />
              
              <div className="relative z-10">
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <div className="text-2xl">{info.icon}</div>
                    <Badge 
                      variant="outline" 
                      className={cn(
                        "text-xs border",
                        RISK_LEVEL_COLORS[info.riskLevel]
                      )}
                    >
                      {info.riskLevel}
                    </Badge>
                  </div>
                  
                  <CardTitle className="text-sm font-semibold text-white/90">
                    {info.name}
                  </CardTitle>
                </CardHeader>

                <CardContent className="space-y-3">
                  {stats.hasStrategies ? (
                    <>
                      <div className="space-y-2">
                        <div className="flex justify-between text-xs">
                          <span className="text-white/60">Total</span>
                          <span className="text-white/80 font-medium">{stats.totalCount}</span>
                        </div>
                        <div className="flex justify-between text-xs">
                          <span className="text-white/60">Active</span>
                          <span className="text-green-400 font-medium">{stats.activeCount}</span>
                        </div>
                        <div className="flex justify-between text-xs">
                          <span className="text-white/60">Invested</span>
                          <span className="text-white/80 font-medium">
                            {stats.totalInvested > 0 ? formatCurrency(stats.totalInvested) : '$0'}
                          </span>
                        </div>
                        <div className="flex justify-between text-xs">
                          <span className="text-white/60">P&L</span>
                          <span className={cn(
                            "font-medium",
                            stats.totalPnL >= 0 ? "text-green-400" : "text-red-400"
                          )}>
                            {stats.totalPnL !== 0 ? formatCurrency(stats.totalPnL) : '$0.00'}
                          </span>
                        </div>
                      </div>
                    </>
                  ) : (
                    <div className="text-center space-y-2">
                      <div className="text-xs text-white/50">No strategies yet</div>
                      <Link href={`/dashboard/strategies/unified?type=${type}`}>
                        <Button 
                          size="sm" 
                          variant="outline"
                          className="w-full border-white/20 text-white/70 hover:bg-white/10 text-xs"
                        >
                          <Plus className="mr-1 h-3 w-3" />
                          Create
                        </Button>
                      </Link>
                    </div>
                  )}
                </CardContent>
              </div>
            </Card>
          )
        })}
      </div>
    </div>
  )
}
