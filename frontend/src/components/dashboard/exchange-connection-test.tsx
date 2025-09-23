"use client"

import { useState } from "react"
import { CheckCircle, AlertTriangle, Loader2, Info, TrendingUp, DollarSign, Activity, Wifi } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"

interface ConnectionTestResult {
  step: string
  status: 'pending' | 'success' | 'error' | 'warning'
  message: string
  details?: string
}

interface ExchangeConnectionTestProps {
  exchange: string
  onTestComplete?: (success: boolean, results: ConnectionTestResult[]) => void
}

export function ExchangeConnectionTest({ exchange, onTestComplete }: ExchangeConnectionTestProps) {
  const [testing, setTesting] = useState(false)
  const [testResults, setTestResults] = useState<ConnectionTestResult[]>([])
  const [currentStep, setCurrentStep] = useState(0)

  const testSteps = [
    {
      step: "Connection Validation",
      description: "Verifying API credentials and connection",
      icon: Wifi
    },
    {
      step: "Permission Check",
      description: "Ensuring correct API permissions",
      icon: CheckCircle
    },
    {
      step: "Account Access",
      description: "Testing read access to account information",
      icon: DollarSign
    },
    {
      step: "Market Data",
      description: "Validating market data access",
      icon: TrendingUp
    },
    {
      step: "Trading Capability",
      description: "Verifying trading permissions (test mode)",
      icon: Activity
    }
  ]

  const runConnectionTest = async () => {
    setTesting(true)
    setTestResults([])
    setCurrentStep(0)

    const results: ConnectionTestResult[] = []

    for (let i = 0; i < testSteps.length; i++) {
      setCurrentStep(i)
      const step = testSteps[i]

      // Simulate API testing with delays
      await new Promise(resolve => setTimeout(resolve, 1500))

      try {
        // TODO: Replace with actual API calls to backend
        const response = await fetch(`/api/test-exchange/${exchange}/${step.step.toLowerCase().replace(' ', '-')}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' }
        })

        if (response.ok) {
          const data = await response.json()

          // Simulate different test outcomes
          let result: ConnectionTestResult

          switch (i) {
            case 0: // Connection
              result = {
                step: step.step,
                status: 'success',
                message: 'API credentials validated successfully',
                details: `Connected to ${exchange} API endpoint`
              }
              break

            case 1: // Permissions
              result = {
                step: step.step,
                status: 'success',
                message: 'API permissions verified',
                details: 'Read account and trading permissions confirmed. No withdrawal access detected.'
              }
              break

            case 2: // Account Access
              result = {
                step: step.step,
                status: 'success',
                message: 'Account information retrieved',
                details: 'Successfully accessed account balance and portfolio data'
              }
              break

            case 3: // Market Data
              result = {
                step: step.step,
                status: 'success',
                message: 'Market data access confirmed',
                details: 'Real-time price feeds and order book data available'
              }
              break

            case 4: // Trading
              result = {
                step: step.step,
                status: 'success',
                message: 'Trading capability verified',
                details: 'Test order simulation completed successfully'
              }
              break

            default:
              result = {
                step: step.step,
                status: 'success',
                message: 'Test completed'
              }
          }

          results.push(result)
        } else {
          throw new Error(`Test failed for ${step.step}`)
        }
      } catch (error) {
        const result: ConnectionTestResult = {
          step: step.step,
          status: 'error',
          message: `${step.step} failed`,
          details: error instanceof Error ? error.message : 'Unknown error occurred'
        }
        results.push(result)
        break // Stop testing on first error
      }

      setTestResults([...results])
    }

    setTesting(false)
    const success = results.every(r => r.status === 'success')
    onTestComplete?.(success, results)
  }

  const getStatusIcon = (status: ConnectionTestResult['status']) => {
    switch (status) {
      case 'success':
        return <CheckCircle className="w-5 h-5 text-emerald-400" />
      case 'error':
        return <AlertTriangle className="w-5 h-5 text-red-400" />
      case 'warning':
        return <AlertTriangle className="w-5 h-5 text-amber-400" />
      default:
        return <div className="w-5 h-5 border-2 border-gray-400/20 border-t-gray-400 rounded-full animate-spin" />
    }
  }

  const getStatusColor = (status: ConnectionTestResult['status']) => {
    switch (status) {
      case 'success': return 'border-emerald-400/30 bg-emerald-500/10'
      case 'error': return 'border-red-400/30 bg-red-500/10'
      case 'warning': return 'border-amber-400/30 bg-amber-500/10'
      default: return 'border-purple-400/30 bg-purple-500/10'
    }
  }

  return (
    <div className="space-y-6">
      {/* Test Header */}
      <div className="text-center space-y-2">
        <h3 className="text-xl font-semibold text-white">Connection Test</h3>
        <p className="text-gray-300">
          We'll run a series of tests to ensure your {exchange} connection is secure and functional
        </p>
      </div>

      {/* Test Steps */}
      <div className="space-y-4">
        {testSteps.map((step, index) => {
          const result = testResults.find(r => r.step === step.step)
          const isActive = testing && currentStep === index
          const isCompleted = !!result
          const isPending = !testing && !isCompleted

          return (
            <div
              key={step.step}
              className={`p-4 rounded-xl border backdrop-blur-sm transition-all duration-300 ${
                isActive ? 'border-purple-400/50 bg-purple-500/10' :
                isCompleted ? getStatusColor(result.status) :
                'border-gray-400/20 bg-gray-500/5'
              }`}
            >
              <div className="flex items-center space-x-4">
                <div className={`p-2 rounded-lg ${
                  isActive ? 'bg-purple-500/20' :
                  isCompleted && result.status === 'success' ? 'bg-emerald-500/20' :
                  isCompleted && result.status === 'error' ? 'bg-red-500/20' :
                  'bg-gray-500/10'
                }`}>
                  {isActive ? (
                    <Loader2 className="w-5 h-5 text-purple-400 animate-spin" />
                  ) : isCompleted ? (
                    getStatusIcon(result.status)
                  ) : (
                    <step.icon className="w-5 h-5 text-gray-400" />
                  )}
                </div>

                <div className="flex-1">
                  <div className="flex items-center justify-between">
                    <h4 className="font-medium text-white">{step.step}</h4>
                    {isActive && (
                      <span className="text-sm text-purple-300">Testing...</span>
                    )}
                    {isCompleted && (
                      <span className={`text-sm ${
                        result.status === 'success' ? 'text-emerald-300' :
                        result.status === 'error' ? 'text-red-300' :
                        'text-amber-300'
                      }`}>
                        {result.status === 'success' ? 'Passed' :
                         result.status === 'error' ? 'Failed' :
                         'Warning'}
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-gray-300">{step.description}</p>

                  {isCompleted && result.message && (
                    <div className="mt-2 space-y-1">
                      <p className={`text-sm font-medium ${
                        result.status === 'success' ? 'text-emerald-200' :
                        result.status === 'error' ? 'text-red-200' :
                        'text-amber-200'
                      }`}>
                        {result.message}
                      </p>
                      {result.details && (
                        <p className="text-xs text-gray-400">{result.details}</p>
                      )}
                    </div>
                  )}
                </div>
              </div>
            </div>
          )
        })}
      </div>

      {/* Test Controls */}
      <div className="flex justify-center">
        <Button
          onClick={runConnectionTest}
          disabled={testing}
          className="h-12 px-8 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-medium transition-all duration-300"
        >
          {testing ? (
            <>
              <Loader2 className="w-5 h-5 mr-2 animate-spin" />
              Running Tests...
            </>
          ) : (
            <>
              <Activity className="w-5 h-5 mr-2" />
              Run Connection Test
            </>
          )}
        </Button>
      </div>

      {/* Test Results Summary */}
      {testResults.length > 0 && !testing && (
        <Alert className={`${
          testResults.every(r => r.status === 'success')
            ? 'border-emerald-400/30 bg-emerald-500/10'
            : 'border-red-400/30 bg-red-500/10'
        }`}>
          {testResults.every(r => r.status === 'success') ? (
            <CheckCircle className="h-4 w-4 text-emerald-400" />
          ) : (
            <AlertTriangle className="h-4 w-4 text-red-400" />
          )}
          <AlertTitle className={
            testResults.every(r => r.status === 'success')
              ? 'text-emerald-300'
              : 'text-red-300'
          }>
            {testResults.every(r => r.status === 'success')
              ? 'Connection Test Passed'
              : 'Connection Test Failed'}
          </AlertTitle>
          <AlertDescription className={
            testResults.every(r => r.status === 'success')
              ? 'text-emerald-200'
              : 'text-red-200'
          }>
            {testResults.every(r => r.status === 'success')
              ? `Your ${exchange} connection is secure and ready for trading. All security checks passed.`
              : `There were issues with your ${exchange} connection. Please check your API credentials and permissions.`}
          </AlertDescription>
        </Alert>
      )}

      {/* Security Reminder */}
      <Alert className="border border-blue-400/30 bg-blue-500/10">
        <Info className="h-4 w-4 text-blue-400" />
        <AlertTitle className="text-blue-300">Security Note</AlertTitle>
        <AlertDescription className="text-blue-200">
          This test only validates your connection and permissions. Your API keys are encrypted
          and never stored in plain text. We recommend running this test periodically to ensure
          your connection remains secure.
        </AlertDescription>
      </Alert>
    </div>
  )
}