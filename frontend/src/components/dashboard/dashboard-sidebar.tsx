"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import {
  BarChart3,
  TrendingUp,
  Activity,
  Settings,
  LogOut,
  X,
  Wifi,
  Shield,
  Zap,
  Target,
  Cog,
  Home,
  LineChart,
  TestTube,
  Wallet
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { useAuth } from "@/contexts/auth-context"

interface DashboardSidebarProps {
  isOpen: boolean
  onClose: () => void
}

export function DashboardSidebar({ isOpen, onClose }: DashboardSidebarProps) {
  const pathname = usePathname()
  const { logout, user } = useAuth()

  const handleLogout = async () => {
    try {
      await logout()
      onClose() // Close sidebar on mobile after logout
    } catch (error) {
      console.error('Logout failed:', error)
    }
  }

  const navItems = [
    {
      href: "/dashboard",
      icon: Home,
      label: "Dashboard",
      description: "Overview & analytics",
      color: "hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20",
      activeColor: "bg-gradient-to-r from-purple-600/90 to-pink-600/90",
      iconColor: "text-purple-300"
    },
    {
      href: "/dashboard/strategies/unified",
      icon: Target,
      label: "Trading Strategies",
      description: "All strategy types",
      color: "hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-violet-500/20",
      activeColor: "bg-gradient-to-r from-purple-600/90 to-violet-600/90",
      iconColor: "text-purple-300"
    },
    {
      href: "/dashboard/backtesting",
      icon: TestTube,
      label: "Backtesting Lab",
      description: "Test strategies",
      color: "hover:bg-gradient-to-r hover:from-cyan-500/20 hover:to-blue-500/20",
      activeColor: "bg-gradient-to-r from-cyan-600/90 to-blue-600/90",
      iconColor: "text-cyan-300"
    },
    {
      href: "/dashboard/exchanges",
      icon: Shield,
      label: "Exchanges",
      description: "Connect accounts",
      color: "hover:bg-gradient-to-r hover:from-blue-500/20 hover:to-cyan-500/20",
      activeColor: "bg-gradient-to-r from-blue-600/90 to-cyan-600/90",
      iconColor: "text-blue-300"
    },
    {
      href: "/dashboard/wallets",
      icon: Wallet,
      label: "Crypto Wallets",
      description: "DEX trading",
      color: "hover:bg-gradient-to-r hover:from-indigo-500/20 hover:to-purple-500/20",
      activeColor: "bg-gradient-to-r from-indigo-600/90 to-purple-600/90",
      iconColor: "text-indigo-300"
    },
    {
      href: "/dashboard/portfolio",
      icon: LineChart,
      label: "Portfolio Analytics",
      description: "Performance insights",
      color: "hover:bg-gradient-to-r hover:from-emerald-500/20 hover:to-teal-500/20",
      activeColor: "bg-gradient-to-r from-emerald-600/90 to-teal-600/90",
      iconColor: "text-emerald-300"
    },
    {
      href: "/dashboard/trades",
      icon: Activity,
      label: "Live Executions",
      description: "Active positions",
      color: "hover:bg-gradient-to-r hover:from-orange-500/20 hover:to-red-500/20",
      activeColor: "bg-gradient-to-r from-orange-600/90 to-red-600/90",
      iconColor: "text-orange-300"
    },
    {
      href: "/dashboard/settings",
      icon: Cog,
      label: "Settings",
      description: "Configure app",
      color: "hover:bg-gradient-to-r hover:from-gray-500/20 hover:to-slate-500/20",
      activeColor: "bg-gradient-to-r from-gray-600/90 to-slate-600/90",
      iconColor: "text-gray-300"
    }
  ]

  return (
    <aside className={`w-72 bg-gradient-to-b from-[rgba(15,12,41,0.98)] via-[rgba(36,36,62,0.98)] to-[rgba(48,43,99,0.98)] backdrop-blur-xl border-r-2 border-[rgba(147,51,234,0.3)] shadow-2xl transform transition-transform duration-300 ease-in-out lg:translate-x-0 lg:static ${
      isOpen ? 'translate-x-0' : '-translate-x-full'
    } fixed inset-y-0 left-0 z-50 lg:relative lg:z-auto flex-shrink-0`}>
      {/* Decorative left border */}
      <div className="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-purple-600 via-pink-500 to-purple-600"></div>

      <div className="flex flex-col h-full overflow-hidden">
        {/* Sidebar Header */}
        <div className="flex items-center justify-between p-6 lg:hidden border-b-2 border-[rgba(147,51,234,0.3)]">
          <div className="flex items-center space-x-2">
            <div className="w-8 h-8 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center">
              <Home className="h-4 w-4 text-white" />
            </div>
            <span className="font-bold text-white text-lg">Navigation</span>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={onClose}
            className="hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 rounded-xl transition-all duration-300 hover:scale-110"
          >
            <X className="h-6 w-6 text-gray-200 hover:text-white transition-colors" />
          </Button>
        </div>


        {/* Navigation */}
        <nav className="flex-1 p-6 space-y-3 overflow-y-auto">
          {navItems.map((item) => {
            const isActive = pathname === item.href
            const Icon = item.icon

            return (
              <Link key={item.href} href={item.href} onClick={onClose}>
                <div
                  className={`group relative p-4 rounded-xl border transition-all duration-300 cursor-pointer hover:scale-105 hover:shadow-lg ${
                    isActive
                      ? `${item.activeColor} border-white/20 text-white shadow-xl`
                      : `${item.color} border-transparent text-gray-300 hover:text-white hover:border-white/10`
                  }`}
                >
                  <div className="flex items-center space-x-3">
                    <div className={`p-2 rounded-lg transition-all duration-300 ${
                      isActive
                        ? "bg-white/20 backdrop-blur-sm"
                        : `bg-gradient-to-r from-${item.iconColor.split('-')[1]}-500/20 to-${item.iconColor.split('-')[1]}-500/10`
                    }`}>
                      <Icon className={`h-5 w-5 transition-transform group-hover:scale-110 ${
                        isActive ? "text-white" : item.iconColor
                      }`} />
                    </div>
                    <div className="flex-1">
                      <div className={`font-semibold transition-colors ${
                        isActive ? "text-white" : "text-gray-200"
                      }`}>
                        {item.label}
                      </div>
                      <div className={`text-xs transition-colors ${
                        isActive ? "text-white/70" : "text-gray-400"
                      }`}>
                        {item.description}
                      </div>
                    </div>
                  </div>

                  {/* Active indicator */}
                  {isActive && (
                    <div className="absolute right-2 top-1/2 transform -translate-y-1/2 w-1 h-8 bg-white rounded-full"></div>
                  )}
                </div>
              </Link>
            )
          })}
        </nav>

        {/* Sidebar Footer */}
        <div className="p-6 border-t-2 border-[rgba(147,51,234,0.3)]">
          {/* User Info */}
          {user && (
            <div className="mb-4 p-3 bg-gradient-to-r from-purple-500/10 to-pink-500/10 border border-purple-400/20 rounded-xl">
              <div className="text-sm font-medium text-white">{user.email}</div>
              <div className="text-xs text-gray-300">
                {user.is_verified ? (
                  <span className="text-emerald-300">✓ Verified Trader</span>
                ) : (
                  <span className="text-yellow-300">Pending Verification</span>
                )}
              </div>
            </div>
          )}

          <Button
            variant="ghost"
            onClick={handleLogout}
            className="w-full justify-start text-red-400 hover:text-red-300 hover:bg-gradient-to-r hover:from-red-500/20 hover:to-red-500/10 rounded-xl p-4 transition-all duration-300 hover:scale-105"
          >
            <div className="p-2 bg-red-500/20 rounded-lg mr-3">
              <LogOut className="h-4 w-4" />
            </div>
            <div>
              <div className="font-semibold">Logout</div>
              <div className="text-xs text-red-300/70">Sign out securely</div>
            </div>
          </Button>
        </div>
      </div>
    </aside>
  )
}