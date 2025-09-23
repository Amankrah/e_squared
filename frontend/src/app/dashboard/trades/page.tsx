"use client"

import { Activity, TrendingUp, DollarSign, AlertTriangle, Pause, Play } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"

export default function TradesPage() {
  return (
    <DashboardLayout>
      <div className="space-y-8">

        {/* Compact Page Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                Active Trades
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Monitor and manage your live trading positions across all connected exchanges
            </p>
          </div>
        </div>

        {/* Coming Soon Card */}
        <div className="relative">
          <div className="h-auto w-full border-2 border-[rgba(16,185,129,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.15)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-12 text-white shadow-2xl text-center">
            <div className="space-y-8">
              <div className="p-6 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-3xl backdrop-blur-sm border border-emerald-400/20 w-fit mx-auto">
                <Activity className="w-16 h-16 text-emerald-200" />
              </div>

              <div className="space-y-4">
                <h2 className="text-3xl font-bold bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                  Trade Monitoring Coming Soon
                </h2>
                <p className="text-xl text-gray-300 max-w-2xl mx-auto leading-relaxed">
                  Real-time trade monitoring, P&L tracking, and risk management tools will be available
                  once you connect your exchanges and deploy trading strategies.
                </p>
              </div>

              <div className="grid gap-6 md:grid-cols-3 max-w-4xl mx-auto">
                <div className="p-6 bg-white/5 rounded-2xl backdrop-blur-sm border border-emerald-400/20">
                  <TrendingUp className="w-8 h-8 text-emerald-300 mx-auto mb-3" />
                  <h3 className="text-lg font-semibold text-white mb-2">Real-time P&L</h3>
                  <p className="text-gray-300 text-sm">Track profits and losses as they happen</p>
                </div>

                <div className="p-6 bg-white/5 rounded-2xl backdrop-blur-sm border border-amber-400/20">
                  <AlertTriangle className="w-8 h-8 text-amber-300 mx-auto mb-3" />
                  <h3 className="text-lg font-semibold text-white mb-2">Risk Alerts</h3>
                  <p className="text-gray-300 text-sm">Get notified of stop-loss triggers and risks</p>
                </div>

                <div className="p-6 bg-white/5 rounded-2xl backdrop-blur-sm border border-blue-400/20">
                  <Play className="w-8 h-8 text-blue-300 mx-auto mb-3" />
                  <h3 className="text-lg font-semibold text-white mb-2">Trade Control</h3>
                  <p className="text-gray-300 text-sm">Pause, resume, or modify active trades</p>
                </div>
              </div>

              <div className="p-6 border border-emerald-400/30 rounded-xl bg-emerald-500/10 backdrop-blur-sm max-w-md mx-auto">
                <p className="text-emerald-300 font-semibold mb-2">📋 Next Steps</p>
                <p className="text-emerald-200 leading-relaxed text-sm">
                  Connect your exchanges and create trading strategies to start monitoring live trades
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </DashboardLayout>
  )
}