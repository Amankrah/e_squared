"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { 
  PlusCircle, 
  BarChart3, 
  Settings, 
  Zap, 
  Target,
  TestTube,
  LineChart,
  Shield,
  ArrowRight,
  Activity
} from "lucide-react"
import Link from "next/link"
import { cn } from "@/lib/utils"

interface QuickActionsPanelProps {
  hasStrategies: boolean
  hasExchangeConnections: boolean
  className?: string
}

export function QuickActionsPanel({ 
  hasStrategies, 
  hasExchangeConnections, 
  className 
}: QuickActionsPanelProps) {
  const quickActions = [
    {
      title: "Create Strategy",
      description: "Build a new automated trading strategy",
      icon: PlusCircle,
      href: "/dashboard/strategies/unified",
      color: "from-purple-600 to-pink-600",
      hoverColor: "hover:from-purple-500 hover:to-pink-500",
      available: true,
      priority: 1
    },
    {
      title: "Connect Exchange",
      description: "Link your trading account",
      icon: Shield,
      href: "/dashboard/exchanges",
      color: "from-blue-600 to-cyan-600",
      hoverColor: "hover:from-blue-500 hover:to-cyan-500",
      available: true,
      priority: hasExchangeConnections ? 3 : 2,
      badge: !hasExchangeConnections ? "Required" : undefined
    },
    {
      title: "Run Backtest",
      description: "Test strategies with historical data",
      icon: TestTube,
      href: "/dashboard/backtesting",
      color: "from-cyan-600 to-blue-600",
      hoverColor: "hover:from-cyan-500 hover:to-blue-500",
      available: true,
      priority: 3
    },
    {
      title: "View Analytics",
      description: "Analyze portfolio performance",
      icon: LineChart,
      href: "/dashboard/portfolio",
      color: "from-emerald-600 to-teal-600",
      hoverColor: "hover:from-emerald-500 hover:to-teal-500",
      available: hasStrategies,
      priority: hasStrategies ? 2 : 4
    },
    {
      title: "Strategy Settings",
      description: "Configure existing strategies",
      icon: Settings,
      href: "/dashboard/strategies/unified",
      color: "from-gray-600 to-slate-600",
      hoverColor: "hover:from-gray-500 hover:to-slate-500",
      available: hasStrategies,
      priority: hasStrategies ? 4 : 5
    },
    {
      title: "Live Executions",
      description: "Monitor active trades",
      icon: Activity,
      href: "/dashboard/trades",
      color: "from-orange-600 to-red-600",
      hoverColor: "hover:from-orange-500 hover:to-red-500",
      available: hasStrategies && hasExchangeConnections,
      priority: 5
    }
  ]

  // Sort by priority and availability
  const sortedActions = quickActions
    .filter(action => action.available)
    .sort((a, b) => a.priority - b.priority)
    .slice(0, 6) // Show max 6 actions

  return (
    <Card className={cn(
      "bg-white/5 backdrop-blur-xl border border-white/10",
      className
    )}>
      <CardHeader>
        <CardTitle className="text-lg font-bold text-white/90">
          Quick Actions
        </CardTitle>
        <CardDescription className="text-white/60">
          Jump to the most important tasks
        </CardDescription>
      </CardHeader>

      <CardContent>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {sortedActions.map((action) => {
            const Icon = action.icon
            
            return (
              <Link key={action.title} href={action.href}>
                <Card className={cn(
                  "group relative overflow-hidden transition-all duration-300 hover:scale-[1.02]",
                  "bg-gradient-to-br backdrop-blur-sm border border-white/10",
                  "hover:border-white/20 hover:shadow-lg cursor-pointer",
                  action.available ? "opacity-100" : "opacity-50"
                )}>
                  <div className="absolute inset-0 bg-white/5 backdrop-blur-sm" />
                  
                  <div className="relative z-10 p-4">
                    <div className="flex items-center justify-between mb-3">
                      <div className={cn(
                        "p-2 rounded-lg bg-gradient-to-r",
                        action.color,
                        "text-white"
                      )}>
                        <Icon className="h-5 w-5" />
                      </div>
                      
                      {action.badge && (
                        <Badge 
                          variant="outline" 
                          className="border-yellow-500/30 text-yellow-400 text-xs"
                        >
                          {action.badge}
                        </Badge>
                      )}
                    </div>
                    
                    <div className="space-y-1">
                      <h4 className="font-semibold text-white/90 text-sm">
                        {action.title}
                      </h4>
                      <p className="text-xs text-white/60 leading-relaxed">
                        {action.description}
                      </p>
                    </div>
                    
                    <div className="flex items-center justify-end mt-3">
                      <ArrowRight className="h-4 w-4 text-white/40 group-hover:text-white/70 group-hover:translate-x-1 transition-all duration-300" />
                    </div>
                  </div>

                  {/* Hover effect */}
                  <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 translate-x-[-100%] group-hover:translate-x-[100%] transform transition-transform duration-1000" />
                </Card>
              </Link>
            )
          })}
        </div>

        {/* Context-sensitive recommendations */}
        {!hasExchangeConnections && (
          <div className="mt-6 p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <Shield className="h-5 w-5 text-yellow-400" />
              <span className="font-medium text-yellow-400">Get Started</span>
            </div>
            <p className="text-sm text-white/70 mb-3">
              Connect an exchange to start live trading with your strategies
            </p>
            <Link href="/dashboard/exchanges">
              <Button 
                size="sm"
                className="bg-gradient-to-r from-yellow-600 to-orange-600 hover:from-yellow-500 hover:to-orange-500 text-white"
              >
                Connect Exchange
                <ArrowRight className="ml-1 h-3 w-3" />
              </Button>
            </Link>
          </div>
        )}

        {!hasStrategies && hasExchangeConnections && (
          <div className="mt-6 p-4 bg-purple-500/10 border border-purple-500/20 rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <Target className="h-5 w-5 text-purple-400" />
              <span className="font-medium text-purple-400">Next Step</span>
            </div>
            <p className="text-sm text-white/70 mb-3">
              Your exchange is connected! Create your first trading strategy to get started
            </p>
            <Link href="/dashboard/strategies/unified">
              <Button 
                size="sm"
                className="bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white"
              >
                Create Strategy
                <ArrowRight className="ml-1 h-3 w-3" />
              </Button>
            </Link>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
