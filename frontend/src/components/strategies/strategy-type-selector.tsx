"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { StrategyType } from "@/lib/api"
import { getStrategyInfo, RISK_LEVEL_COLORS } from "@/lib/strategies"
import { cn } from "@/lib/utils"

interface StrategyTypeSelectorProps {
  selectedType?: StrategyType
  onTypeSelect: (type: StrategyType) => void
  className?: string
}

export function StrategyTypeSelector({
  selectedType,
  onTypeSelect,
  className
}: StrategyTypeSelectorProps) {
  const strategyTypes: StrategyType[] = ['dca', 'grid_trading', 'sma_crossover', 'rsi', 'macd']

  return (
    <div className={cn("grid gap-6", className)}>
      <div className="text-center space-y-2">
        <h2 className="text-2xl font-bold bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent">
          Choose Your Strategy
        </h2>
        <p className="text-white/60 max-w-2xl mx-auto">
          Select a trading strategy that matches your investment goals and risk tolerance. 
          Each strategy can be backtested before going live.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {strategyTypes.map((type) => {
          const info = getStrategyInfo(type)
          const isSelected = selectedType === type

          return (
            <Card
              key={type}
              className={cn(
                "group relative overflow-hidden cursor-pointer transition-all duration-300",
                "bg-gradient-to-br backdrop-blur-xl border",
                "hover:scale-[1.02] hover:shadow-2xl hover:shadow-black/20",
                info.color,
                isSelected 
                  ? "border-white/30 shadow-xl shadow-white/10" 
                  : "border-white/10 hover:border-white/20"
              )}
              onClick={() => onTypeSelect(type)}
            >
              {/* Glassmorphism overlay */}
              <div className="absolute inset-0 bg-white/5 backdrop-blur-sm" />
              
              {/* Selection indicator */}
              {isSelected && (
                <div className="absolute top-4 right-4 w-3 h-3 bg-white rounded-full shadow-lg z-10" />
              )}
              
              {/* Content */}
              <div className="relative z-10">
                <CardHeader className="text-center space-y-4">
                  <div className="text-4xl mx-auto">{info.icon}</div>
                  <div className="space-y-2">
                    <CardTitle className="text-xl font-semibold text-white/90">
                      {info.name}
                    </CardTitle>
                    <CardDescription className="text-white/60 text-sm leading-relaxed">
                      {info.description}
                    </CardDescription>
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  {/* Risk and Time Horizon */}
                  <div className="flex items-center justify-between">
                    <Badge 
                      variant="outline" 
                      className={cn(
                        "border font-medium",
                        `border-current ${RISK_LEVEL_COLORS[info.riskLevel]}`
                      )}
                    >
                      {info.riskLevel} Risk
                    </Badge>
                    <span className="text-xs text-white/60">
                      {info.timeHorizon}
                    </span>
                  </div>

                  {/* Min Investment */}
                  <div className="text-center">
                    <p className="text-sm text-white/60">Minimum Investment</p>
                    <p className="text-lg font-semibold text-white/90">
                      ${info.minInvestment.toLocaleString()}
                    </p>
                  </div>

                  {/* Key Features */}
                  <div className="space-y-2">
                    <p className="text-sm font-medium text-white/80">Key Features:</p>
                    <ul className="text-xs text-white/60 space-y-1">
                      {info.features.slice(0, 3).map((feature, index) => (
                        <li key={index} className="flex items-center space-x-2">
                          <div className="w-1 h-1 bg-white/40 rounded-full" />
                          <span>{feature}</span>
                        </li>
                      ))}
                      {info.features.length > 3 && (
                        <li className="text-white/40 italic">
                          +{info.features.length - 3} more...
                        </li>
                      )}
                    </ul>
                  </div>
                </CardContent>
              </div>

              {/* Hover effect */}
              <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 translate-x-[-100%] group-hover:translate-x-[100%] transform transition-transform duration-1000" />
            </Card>
          )
        })}
      </div>
    </div>
  )
}
