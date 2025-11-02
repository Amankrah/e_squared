"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { 
  Play, 
  Pause, 
  Settings, 
  TrendingUp, 
  TrendingDown, 
  MoreHorizontal,
  AlertTriangle,
  Clock
} from "lucide-react"
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Strategy, StrategyType } from "@/lib/api"
import { 
  getStrategyInfo, 
  getStrategyStatusColor, 
  formatStrategyStatus,
  formatCurrency,
  formatPercentage 
} from "@/lib/strategies"
import { cn } from "@/lib/utils"

interface StrategyCardProps {
  strategy: Strategy
  strategyType: StrategyType
  onEdit?: (strategy: Strategy) => void
  onDelete?: (strategy: Strategy) => void
  onPause?: (strategy: Strategy) => Promise<void>
  onResume?: (strategy: Strategy) => Promise<void>
  onExecute?: (strategy: Strategy) => Promise<void>
  className?: string
}

export function StrategyCard({
  strategy,
  strategyType,
  onEdit,
  onDelete,
  onPause,
  onResume,
  onExecute,
  className
}: StrategyCardProps) {
  const [isLoading, setIsLoading] = useState(false)
  const strategyInfo = getStrategyInfo(strategyType)
  
  const profitLoss = strategy.current_profit_loss ? parseFloat(strategy.current_profit_loss) : 0
  const profitLossPercentage = strategy.profit_loss_percentage ? parseFloat(strategy.profit_loss_percentage) : 0
  
  const isProfit = profitLoss >= 0
  const isActive = strategy.status?.toLowerCase() === 'active'
  const isPaused = strategy.status?.toLowerCase() === 'paused'

  const handleAction = async (action: () => Promise<void>) => {
    setIsLoading(true)
    try {
      await action()
    } catch (error) {
      console.error('Strategy action failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Card className={cn(
      "group relative overflow-hidden transition-all duration-300 hover:scale-[1.02]",
      "bg-gradient-to-br backdrop-blur-xl border border-white/10",
      "hover:border-white/20 hover:shadow-2xl hover:shadow-black/20",
      strategyInfo.color,
      className
    )}>
      {/* Glassmorphism overlay */}
      <div className="absolute inset-0 bg-white/5 backdrop-blur-sm" />
      
      {/* Content */}
      <div className="relative z-10">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <div className="flex items-center space-x-3">
            <div className="text-2xl">{strategyInfo.icon}</div>
            <div>
              <CardTitle className="text-lg font-semibold text-white/90">
                {strategy.name}
              </CardTitle>
              <CardDescription className="text-white/60 font-medium">
                {strategyInfo.name} • {strategy.asset_symbol}
              </CardDescription>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            <Badge 
              variant="outline" 
              className={cn(
                "border font-medium",
                getStrategyStatusColor(strategy.status)
              )}
            >
              {formatStrategyStatus(strategy.status)}
            </Badge>
            
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button 
                  variant="ghost" 
                  size="sm" 
                  className="h-8 w-8 p-0 text-white/70 hover:text-white hover:bg-white/10"
                  disabled={isLoading}
                >
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent 
                align="end" 
                className="bg-black/80 backdrop-blur-xl border-white/10"
              >
                <DropdownMenuItem 
                  onClick={() => onEdit?.(strategy)}
                  className="text-white/80 hover:text-white hover:bg-white/10"
                >
                  <Settings className="mr-2 h-4 w-4" />
                  Configure
                </DropdownMenuItem>
                
                {isActive ? (
                  <DropdownMenuItem 
                    onClick={() => onPause && handleAction(() => onPause(strategy))}
                    className="text-yellow-400 hover:text-yellow-300 hover:bg-yellow-500/10"
                  >
                    <Pause className="mr-2 h-4 w-4" />
                    Pause
                  </DropdownMenuItem>
                ) : (
                  <DropdownMenuItem 
                    onClick={() => onResume && handleAction(() => onResume(strategy))}
                    className="text-green-400 hover:text-green-300 hover:bg-green-500/10"
                  >
                    <Play className="mr-2 h-4 w-4" />
                    Resume
                  </DropdownMenuItem>
                )}
                
                <DropdownMenuItem 
                  onClick={() => onExecute && handleAction(() => onExecute(strategy))}
                  className="text-blue-400 hover:text-blue-300 hover:bg-blue-500/10"
                  disabled={!isActive}
                >
                  <Play className="mr-2 h-4 w-4" />
                  Execute Now
                </DropdownMenuItem>
                
                <DropdownMenuItem 
                  onClick={() => onDelete?.(strategy)}
                  className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
                >
                  <AlertTriangle className="mr-2 h-4 w-4" />
                  Delete
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Performance Metrics */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <p className="text-sm text-white/60">Total Invested</p>
              <p className="text-lg font-semibold text-white/90">
                {formatCurrency(strategy.total_invested)}
              </p>
            </div>
            
            <div className="space-y-1">
              <p className="text-sm text-white/60">Current P&L</p>
              <div className="flex items-center space-x-1">
                {isProfit ? (
                  <TrendingUp className="h-4 w-4 text-green-400" />
                ) : (
                  <TrendingDown className="h-4 w-4 text-red-400" />
                )}
                <p className={cn(
                  "text-lg font-semibold",
                  isProfit ? "text-green-400" : "text-red-400"
                )}>
                  {formatCurrency(profitLoss)}
                </p>
                <span className={cn(
                  "text-sm",
                  isProfit ? "text-green-400/70" : "text-red-400/70"
                )}>
                  ({formatPercentage(profitLossPercentage)})
                </span>
              </div>
            </div>
          </div>

          {/* Holdings */}
          {strategy.total_purchased && parseFloat(strategy.total_purchased) > 0 && (
            <div className="space-y-1">
              <p className="text-sm text-white/60">Holdings</p>
              <p className="text-base font-medium text-white/80">
                {parseFloat(strategy.total_purchased).toFixed(6)} {strategy.asset_symbol}
                {strategy.average_buy_price && (
                  <span className="text-sm text-white/60 ml-2">
                    @ {formatCurrency(strategy.average_buy_price)}
                  </span>
                )}
              </p>
            </div>
          )}

          {/* Execution Status */}
          <div className="flex items-center justify-between text-sm">
            <div className="flex items-center space-x-2 text-white/60">
              <Clock className="h-4 w-4" />
              <span>
                {strategy.last_execution_at 
                  ? `Last: ${new Date(strategy.last_execution_at).toLocaleDateString()}`
                  : 'Never executed'
                }
              </span>
            </div>
            
            {strategy.next_execution_at && isActive && (
              <div className="text-white/60">
                Next: {new Date(strategy.next_execution_at).toLocaleDateString()}
              </div>
            )}
          </div>

          {/* Strategy-specific metrics */}
          <div className="pt-2 border-t border-white/10">
            <div className="flex items-center justify-between text-sm">
              <span className="text-white/60">Risk Level</span>
              <Badge 
                variant="outline" 
                className={cn(
                  "text-xs border",
                  strategyInfo.riskLevel === 'Low' && "border-green-500/30 text-green-400",
                  strategyInfo.riskLevel === 'Medium' && "border-yellow-500/30 text-yellow-400",
                  strategyInfo.riskLevel === 'High' && "border-red-500/30 text-red-400"
                )}
              >
                {strategyInfo.riskLevel}
              </Badge>
            </div>
          </div>
        </CardContent>
      </div>

      {/* Loading overlay */}
      {isLoading && (
        <div className="absolute inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-20">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
        </div>
      )}
    </Card>
  )
}
