"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { TrendingUp, TrendingDown, DollarSign, Bitcoin, Wallet } from "lucide-react"

interface BtcDominanceData {
  value: string
  change_24h: string | null
  timestamp: number
}

interface M2Data {
  value: string
  change: string | null
  percent_change: string | null
  date: string
  timestamp: number
}

interface BtcPriceData {
  price: string
  change_24h: string | null
  percent_change_24h: string | null
  high_24h: string | null
  low_24h: string | null
  timestamp: number
}

export function BtcDominanceIndicator() {
  const [data, setData] = useState<BtcDominanceData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 5 * 60 * 1000) // Refresh every 5 minutes
    return () => clearInterval(interval)
  }, [])

  const fetchData = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/v1/market-data/btc-dominance', {
        credentials: 'include',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) throw new Error(`Failed to fetch BTC dominance: ${response.statusText}`)

      const result = await response.json()
      setData(result)
      setError(null)
    } catch (err) {
      console.error('Error fetching BTC dominance:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">BTC Dominance</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
            <Bitcoin className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white">Loading...</div>
        </CardContent>
      </Card>
    )
  }

  if (error || !data) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">BTC Dominance</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
            <Bitcoin className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-gray-400">Unable to load</div>
        </CardContent>
      </Card>
    )
  }

  const value = parseFloat(data.value)
  const change24h = data.change_24h ? parseFloat(data.change_24h) : null
  const isPositive = change24h !== null && change24h >= 0

  return (
    <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium text-gray-200">BTC Dominance</CardTitle>
        <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
          <Bitcoin className="h-4 w-4 text-white" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-white">{value.toFixed(2)}%</div>
        {change24h !== null && (
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
              {isPositive ? '+' : ''}{change24h.toFixed(2)}%
            </Badge>
          </div>
        )}
        <p className="text-xs text-gray-400 mt-1">
          Live Data · Updated {new Date(data.timestamp * 1000).toLocaleTimeString()}
        </p>
      </CardContent>
    </Card>
  )
}

export function M2Indicator() {
  const [data, setData] = useState<M2Data | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 60 * 60 * 1000) // Refresh every hour
    return () => clearInterval(interval)
  }, [])

  const fetchData = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/v1/market-data/m2', {
        credentials: 'include',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) throw new Error(`Failed to fetch M2: ${response.statusText}`)

      const result = await response.json()
      setData(result)
      setError(null)
    } catch (err) {
      console.error('Error fetching M2:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">M2 Money Supply (US)</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-green-500 to-emerald-600 rounded-lg flex items-center justify-center">
            <Wallet className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white">Loading...</div>
        </CardContent>
      </Card>
    )
  }

  if (error || !data) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">M2 Money Supply (US)</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-green-500 to-emerald-600 rounded-lg flex items-center justify-center">
            <Wallet className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-gray-400">Unable to load</div>
        </CardContent>
      </Card>
    )
  }

  const value = parseFloat(data.value)
  const percentChange = data.percent_change ? parseFloat(data.percent_change) : null
  const isPositive = percentChange !== null && percentChange >= 0

  return (
    <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium text-gray-200">M2 Money Supply (US)</CardTitle>
        <div className="h-8 w-8 bg-gradient-to-br from-green-500 to-emerald-600 rounded-lg flex items-center justify-center">
          <Wallet className="h-4 w-4 text-white" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-white">${(value / 1000).toFixed(2)}T</div>
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
            {data.change && (
              <span className={`text-xs ${isPositive ? 'text-emerald-400' : 'text-red-400'}`}>
                {isPositive ? '+' : ''}${(parseFloat(data.change) / 1000).toFixed(2)}T
              </span>
            )}
          </div>
        )}
        <p className="text-xs text-gray-400 mt-1">
          As of {data.date} · Updated {new Date(data.timestamp * 1000).toLocaleTimeString()}
        </p>
      </CardContent>
    </Card>
  )
}

export function BtcPriceIndicator() {
  const [data, setData] = useState<BtcPriceData | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetchData()
    const interval = setInterval(fetchData, 60 * 1000) // Refresh every 1 minute
    return () => clearInterval(interval)
  }, [])

  const fetchData = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/v1/market-data/btc-price', {
        credentials: 'include',
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) throw new Error(`Failed to fetch BTC price: ${response.statusText}`)

      const result = await response.json()
      setData(result)
      setError(null)
    } catch (err) {
      console.error('Error fetching BTC price:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch data')
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">Bitcoin Price</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
            <Bitcoin className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-white">Loading...</div>
        </CardContent>
      </Card>
    )
  }

  if (error || !data) {
    return (
      <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium text-gray-200">Bitcoin Price</CardTitle>
          <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
            <Bitcoin className="h-4 w-4 text-white" />
          </div>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-gray-400">Unable to load</div>
        </CardContent>
      </Card>
    )
  }

  const price = parseFloat(data.price)
  const percentChange = data.percent_change_24h ? parseFloat(data.percent_change_24h) : null
  const isPositive = percentChange !== null && percentChange >= 0

  return (
    <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium text-gray-200">Bitcoin Price</CardTitle>
        <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-yellow-600 rounded-lg flex items-center justify-center">
          <Bitcoin className="h-4 w-4 text-white" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-white">${price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
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
            {data.change_24h && (
              <span className={`text-xs ${isPositive ? 'text-emerald-400' : 'text-red-400'}`}>
                {isPositive ? '+' : ''}${parseFloat(data.change_24h).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </span>
            )}
          </div>
        )}
        <p className="text-xs text-gray-400 mt-1">
          Live Data · Updated {new Date(data.timestamp * 1000).toLocaleTimeString()}
        </p>
      </CardContent>
    </Card>
  )
}
