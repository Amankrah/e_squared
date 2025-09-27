"use client"

import { useState } from "react"
import { useRouter } from "next/navigation"
import { Menu, Bell, User, Search, Settings, Palette, LogOut } from "lucide-react"
import { Button } from "@/components/ui/button"
import { ThemeToggle } from "@/components/theme-toggle"
import { ProgrammableLogo } from "@/components/programmable-logo"
import { useAuth } from "@/contexts/auth-context"

interface DashboardHeaderProps {
  onMenuToggle: () => void
}

export function DashboardHeader({ onMenuToggle }: DashboardHeaderProps) {
  const router = useRouter()
  const { logout, user } = useAuth()

  const handleLogoClick = () => {
    router.push('/')
  }

  const handleLogout = async () => {
    try {
      await logout()
    } catch (error) {
      console.error('Logout failed:', error)
    }
  }

  return (
    <header className="sticky top-0 z-40 w-full border-b-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-r from-[rgba(15,12,41,0.98)] via-[rgba(36,36,62,0.98)] to-[rgba(48,43,99,0.98)] backdrop-blur-xl shadow-2xl">
      {/* Decorative top border */}
      <div className="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-purple-600 via-pink-500 to-purple-600"></div>

      <div className="flex h-18 items-center justify-between px-6">
        <div className="flex items-center space-x-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={onMenuToggle}
            className="lg:hidden hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:scale-110 transition-all duration-300 rounded-xl p-2"
          >
            <Menu className="h-6 w-6 text-gray-200 hover:text-white transition-colors" />
          </Button>

          <button
            onClick={handleLogoClick}
            className="hover:scale-105 transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-purple-500/50 rounded-xl p-2 hover:bg-gradient-to-r hover:from-purple-500/10 hover:to-pink-500/10"
            aria-label="Go to home page"
          >
            <ProgrammableLogo size="sm" showText={true} />
          </button>

          {/* Search Bar */}
          <div className="hidden md:flex items-center space-x-2 bg-gradient-to-r from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.05)] border border-[rgba(147,51,234,0.3)] rounded-xl px-4 py-2 backdrop-blur-sm min-w-[300px]">
            <Search className="h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search exchanges, strategies, trades..."
              className="bg-transparent text-gray-200 placeholder-gray-400 focus:outline-none flex-1 text-sm"
            />
          </div>
        </div>

        <div className="flex items-center space-x-3">

          {/* Notifications */}
          <Button
            variant="ghost"
            size="icon"
            className="relative hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:scale-110 transition-all duration-300 rounded-xl"
          >
            <Bell className="h-5 w-5 text-gray-300 hover:text-white transition-colors" />
            <div className="absolute -top-1 -right-1 w-3 h-3 bg-gradient-to-r from-pink-500 to-red-500 rounded-full border border-white/20"></div>
          </Button>

          <div
            className="relative group"
            title="Theme & Appearance Settings"
          >
            <ThemeToggle />
            <button
              onClick={() => router.push('/dashboard/settings?section=appearance')}
              className="absolute inset-0 opacity-0 hover:opacity-100 bg-gradient-to-r from-pink-500/20 to-red-500/20 rounded-xl transition-all duration-300 hover:scale-110 flex items-center justify-center"
              aria-label="Open appearance settings"
            >
              <Palette className="w-3 h-3 text-pink-300" />
            </button>
          </div>

          {/* Settings */}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => router.push('/dashboard/settings')}
            className="hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:scale-110 transition-all duration-300 rounded-xl"
          >
            <Settings className="h-5 w-5 text-gray-300 hover:text-white transition-colors" />
          </Button>

          {/* Profile */}
          <Button
            variant="ghost"
            onClick={() => router.push('/dashboard/settings?section=profile')}
            className="flex items-center space-x-2 text-gray-200 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 transition-all duration-300 rounded-xl px-3 py-2"
          >
            <div className="w-8 h-8 bg-gradient-to-r from-purple-500 to-pink-500 rounded-full flex items-center justify-center">
              <User className="h-4 w-4 text-white" />
            </div>
            <span className="hidden sm:block text-sm font-medium">
              {user?.email?.split('@')[0] || 'Profile'}
            </span>
          </Button>

          {/* Logout Button */}
          <Button
            variant="ghost"
            onClick={handleLogout}
            className="flex items-center space-x-2 text-gray-200 hover:bg-gradient-to-r hover:from-red-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 transition-all duration-300 rounded-xl px-3 py-2"
            title="Logout"
          >
            <LogOut className="h-5 w-5" />
            <span className="hidden sm:block text-sm font-medium">Logout</span>
          </Button>
        </div>
      </div>
    </header>
  )
}