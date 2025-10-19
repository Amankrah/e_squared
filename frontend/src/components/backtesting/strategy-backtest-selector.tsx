"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { StrategyType, StrategyConfig } from "@/lib/api"
import { getStrategyInfo, RISK_LEVEL_COLORS, DEFAULT_CONFIGS } from "@/lib/strategies"
import { cn } from "@/lib/utils"
import { TestTube, BarChart3, Target, TrendingUp, Settings } from "lucide-react"

interface StrategyBacktestSelectorProps {
  selectedType?: StrategyType
  onStrategySelect: (type: StrategyType, config: StrategyConfig) => void
  onConfigureStrategy: (type: StrategyType) => void
  className?: string
}

export function StrategyBacktestSelector({
  selectedType,
  onStrategySelect,
  onConfigureStrategy,
  className
}: StrategyBacktestSelectorProps) {
  const strategyTypes: StrategyType[] = ['dca', 'grid_trading', 'sma_crossover']
  const [hoveredStrategy, setHoveredStrategy] = useState<StrategyType | null>(null)

  const handleStrategySelect = (type: StrategyType) => {
    const defaultConfig = DEFAULT_CONFIGS[type]
    onStrategySelect(type, defaultConfig)
  }

  return (
    <div className={cn("space-y-8", className)}>
      <div className="text-center space-y-3">
        <h2 className="text-3xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
          Select Strategy for Backtesting
        </h2>
        <p className="text-white/70 max-w-3xl mx-auto text-lg">
          Choose a trading strategy to test with historical data. Each strategy has its own
          specialized configuration interface for optimal parameter tuning.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8 max-w-7xl mx-auto">
        {strategyTypes.map((type) => {
          const info = getStrategyInfo(type)
          const isSelected = selectedType === type
          const isHovered = hoveredStrategy === type

          return (
            <Card
              key={type}
              className={cn(
                "group relative overflow-hidden transition-all duration-300",
                "bg-gradient-to-br backdrop-blur-xl border",
                "hover:scale-[1.03] hover:shadow-2xl hover:shadow-black/20",
                "cursor-pointer min-h-[420px]", // Increased height for better proportions
                info.color,
                isSelected
                  ? "border-white/40 shadow-xl shadow-white/20 scale-[1.02]"
                  : "border-white/10 hover:border-white/30"
              )}
              onMouseEnter={() => setHoveredStrategy(type)}
              onMouseLeave={() => setHoveredStrategy(null)}
            >
              {/* Enhanced glassmorphism overlay */}
              <div className="absolute inset-0 bg-white/5 backdrop-blur-sm" />

              {/* Selection indicator */}
              {isSelected && (
                <div className="absolute top-4 right-4 w-4 h-4 bg-white rounded-full shadow-lg z-10 animate-pulse" />
              )}

              {/* Content */}
              <div className="relative z-10 h-full flex flex-col">
                <CardHeader className="text-center space-y-4 pb-4">
                  <div className="text-5xl mx-auto transform transition-transform duration-300 group-hover:scale-110">
                    {info.icon}
                  </div>
                  <div className="space-y-2">
                    <CardTitle className="text-xl font-bold text-white/90 leading-tight">
                      {info.name}
                    </CardTitle>
                    <CardDescription className="text-white/70 text-sm leading-relaxed px-2">
                      {info.description}
                    </CardDescription>
                  </div>
                </CardHeader>

                <CardContent className="flex-1 flex flex-col justify-between space-y-6">
                  {/* Strategy Metrics */}
                  <div className="space-y-4">
                    {/* Risk and Performance Indicators */}
                    <div className="grid grid-cols-2 gap-3">
                      <div className="text-center bg-white/5 rounded-lg p-3">
                        <div className="flex items-center justify-center mb-1">
                          <Target className="h-4 w-4 text-white/60 mr-1" />
                          <span className="text-xs text-white/60">Risk Level</span>
                        </div>
                        <Badge
                          variant="outline"
                          className={cn(
                            "border font-medium text-xs",
                            `border-current ${RISK_LEVEL_COLORS[info.riskLevel]}`
                          )}
                        >
                          {info.riskLevel}
                        </Badge>
                      </div>

                      <div className="text-center bg-white/5 rounded-lg p-3">
                        <div className="flex items-center justify-center mb-1">
                          <BarChart3 className="h-4 w-4 text-white/60 mr-1" />
                          <span className="text-xs text-white/60">Time Frame</span>
                        </div>
                        <span className="text-xs font-medium text-white/80">
                          {info.timeHorizon}
                        </span>
                      </div>
                    </div>

                    {/* Min Investment */}
                    <div className="text-center bg-white/5 rounded-lg p-3">
                      <p className="text-xs text-white/60 mb-1">Minimum Investment</p>
                      <p className="text-lg font-bold text-white/90">
                        ${info.minInvestment.toLocaleString()}
                      </p>
                    </div>

                    {/* Key Features Preview */}
                    <div className="space-y-2">
                      <p className="text-sm font-medium text-white/80 flex items-center">
                        <TrendingUp className="h-4 w-4 mr-1" />
                        Key Features:
                      </p>
                      <ul className="text-xs text-white/70 space-y-1">
                        {info.features.slice(0, isHovered ? info.features.length : 2).map((feature, index) => (
                          <li key={index} className="flex items-start space-x-2">
                            <div className="w-1.5 h-1.5 bg-white/50 rounded-full mt-1.5 flex-shrink-0" />
                            <span className="leading-relaxed">{feature}</span>
                          </li>
                        ))}
                        {!isHovered && info.features.length > 2 && (
                          <li className="text-white/50 italic text-center pt-1">
                            Hover to see {info.features.length - 2} more features...
                          </li>
                        )}
                      </ul>
                    </div>
                  </div>

                  {/* Action Buttons */}
                  <div className="space-y-3 pt-4 border-t border-white/10">
                    <Button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleStrategySelect(type)
                      }}
                      className="w-full bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white font-medium"
                      size="sm"
                    >
                      <TestTube className="h-4 w-4 mr-2" />
                      Configure & Test
                    </Button>

                    <Button
                      onClick={(e) => {
                        e.stopPropagation()
                        onConfigureStrategy(type)
                      }}
                      variant="outline"
                      className="w-full border-white/30 text-white/80 hover:bg-white/10 hover:text-white"
                      size="sm"
                    >
                      <Settings className="h-4 w-4 mr-2" />
                      Custom Config
                    </Button>
                  </div>
                </CardContent>
              </div>

              {/* Enhanced hover effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 translate-x-[-100%] group-hover:translate-x-[100%] transform transition-transform duration-1000" />
            </Card>
          )
        })}
      </div>

      {/* Strategy Comparison Hint */}
      <div className="text-center bg-white/5 backdrop-blur-sm rounded-lg p-6 max-w-4xl mx-auto border border-white/10">
        <div className="flex items-center justify-center space-x-2 mb-2">
          <BarChart3 className="h-5 w-5 text-cyan-400" />
          <h3 className="text-lg font-semibold text-white/90">Pro Tip</h3>
        </div>
        <p className="text-white/70 text-sm leading-relaxed">
          Each strategy provides a specialized configuration interface tailored to its unique parameters.
          Configure your strategy with precision and run comprehensive backtests with historical data.
        </p>
      </div>
    </div>
  )
}