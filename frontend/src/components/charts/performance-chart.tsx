"use client"

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'

interface PerformanceDataPoint {
  date: string
  portfolio_value: number
  return_percentage: number
}

interface PerformanceChartProps {
  data: PerformanceDataPoint[]
  initialBalance: number
}

export function PerformanceChart({ data, initialBalance }: PerformanceChartProps) {
  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(value)
  }

  const formatPercentage = (value: number) => {
    return `${(value * 100).toFixed(2)}%`
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
  }

  // Custom tooltip component
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload
      return (
        <div className="bg-black/90 border border-[rgba(147,51,234,0.3)] rounded-lg p-3 shadow-lg">
          <p className="text-white font-semibold mb-2">{formatDate(label)}</p>
          <div className="space-y-1">
            <p className="text-purple-300">
              Portfolio Value: {formatCurrency(data.portfolio_value)}
            </p>
            <p className="text-blue-300">
              Daily Return: {formatPercentage(data.return_percentage)}
            </p>
          </div>
        </div>
      )
    }
    return null
  }

  return (
    <div className="w-full h-96">
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={data}
          margin={{
            top: 5,
            right: 30,
            left: 20,
            bottom: 5,
          }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(147,51,234,0.2)" />
          <XAxis
            dataKey="date"
            tickFormatter={formatDate}
            stroke="rgba(255,255,255,0.6)"
            fontSize={12}
          />
          <YAxis
            tickFormatter={formatCurrency}
            stroke="rgba(255,255,255,0.6)"
            fontSize={12}
          />
          <Tooltip content={<CustomTooltip />} />
          <Line
            type="monotone"
            dataKey="portfolio_value"
            stroke="rgb(147,51,234)"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4, fill: 'rgb(147,51,234)' }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  )
}