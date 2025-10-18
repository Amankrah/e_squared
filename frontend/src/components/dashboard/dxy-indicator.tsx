"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { TrendingUp, TrendingDown, DollarSign } from "lucide-react"

interface DxyData {
  value: string
  change: string | null
  percent_change: string | null
  high_24h: string | null
  low_24h: string | null
  timestamp: number
}

export function DxyIndicator() {
  const [dxyData, setDxyData] = useState<DxyData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchDxyData()
    // Refresh every 5 minutes
    const interval = setInterval(fetchDxyData, 5 * 60 * 1000)
    return () => clearInterval(interval)
  }, [])

  const fetchDxyData = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/v1/market-data/dxy', {
        credentials: 'include',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`Failed to fetch DXY data: ${response.statusText}`)
      }

      const data = await response.json()
      setDxyData(data)
      setError(null)
    } catch (err) {
      console.error('Error fetching DXY data:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch DXY data')
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">DXY (US Dollar Index)</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-blue-500 to-cyan-600 rounded-lg flex items-center justify-center">
            <DollarSign className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white">Loading...</div>
        </CardContent>
      </Card>
    )
  }

  if (error) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">DXY (US Dollar Index)</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-blue-500 to-cyan-600 rounded-lg flex items-center justify-center">
            <DollarSign className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-gray-400">Unable to load</div>
        </CardContent>
      </Card>
    )
  }

  if (!dxyData) {
    return null
  }

  const value = parseFloat(dxyData.value)
  const percentChange = dxyData.percent_change ? parseFloat(dxyData.percent_change) : null
  const isPositive = percentChange !== null && percentChange >= 0

  return (
    <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium text-gray-200">DXY (US Dollar Index)</CardTitle>
        <div className="h-8 w-8 bg-gradient-to-br from-blue-500 to-cyan-600 rounded-lg flex items-center justify-center">
          <DollarSign className="h-4 w-4 text-white" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-white">{value.toFixed(2)}</div>
        {percentChange !== null && (
          <div className="flex items-center gap-2 mt-1">
            <Badge
              variant={isPositive ? "default" : "destructive"}
              className={`flex items-center gap-1 ${isPositive ? 'bg-emerald-500/20 text-emerald-300 border-emerald-400/30' : 'bg-red-500/20 text-red-300 border-red-400/30'}`}
            >
              {isPositive ? (
                <TrendingUp className="h-3 w-3" />
              ) : (
                <TrendingDown className="h-3 w-3" />
              )}
              {isPositive ? '+' : ''}{percentChange.toFixed(2)}%
            </Badge>
            {dxyData.change && (
              <span className={`text-xs ${isPositive ? 'text-emerald-400' : 'text-red-400'}`}>
                {isPositive ? '+' : ''}{parseFloat(dxyData.change).toFixed(2)}
              </span>
            )}
          </div>
        )}
        {dxyData.high_24h && dxyData.low_24h && (
          <div className="text-xs text-gray-400 mt-2">
            24h: {parseFloat(dxyData.low_24h).toFixed(2)} - {parseFloat(dxyData.high_24h).toFixed(2)}
          </div>
        )}
        <p className="text-xs text-gray-400 mt-1">
          Live Data · Updated {new Date(dxyData.timestamp * 1000).toLocaleTimeString()}
        </p>
      </CardContent>
    </Card>
  )
}
