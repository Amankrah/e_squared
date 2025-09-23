"use client"

import { useState } from "react"
import { Eye, EyeOff, Link2, Shield, AlertCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { apiClient } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"

interface ExchangeConnectionDialogProps {
  isOpen: boolean
  onClose: () => void
  onSuccess?: () => void
}

const SUPPORTED_EXCHANGES = [
  { value: 'binance', label: 'Binance', icon: '🟡' },
  { value: 'coinbase', label: 'Coinbase', icon: '🔵' },
  { value: 'kraken', label: 'Kraken', icon: '🟣' },
  { value: 'bybit', label: 'Bybit', icon: '🟠' },
  { value: 'kucoin', label: 'KuCoin', icon: '🟢' },
  { value: 'okx', label: 'OKX', icon: '⚪' }
]

export function ExchangeConnectionDialog({
  isOpen,
  onClose,
  onSuccess
}: ExchangeConnectionDialogProps) {
  const { toast } = useToast()
  const [formData, setFormData] = useState({
    exchange: '',
    displayName: '',
    apiKey: '',
    apiSecret: '',
    password: ''
  })
  const [showPassword, setShowPassword] = useState(false)
  const [showApiSecret, setShowApiSecret] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setIsLoading(true)

    try {
      // Validate required fields
      if (!formData.exchange || !formData.displayName || !formData.apiKey || !formData.apiSecret || !formData.password) {
        throw new Error('Please fill in all required fields')
      }


      // Create exchange connection
      const result = await apiClient.createExchangeConnection({
        exchange_name: formData.exchange,
        display_name: formData.displayName,
        api_key: formData.apiKey,
        api_secret: formData.apiSecret,
        password: formData.password
      })

      // Check if this was an update (200) or create (201) based on response
      const wasUpdate = result && 'updated_at' in result
      
      toast({
        title: wasUpdate ? "Exchange Updated" : "Exchange Connected",
        description: wasUpdate 
          ? `Successfully updated ${formData.displayName} connection`
          : `Successfully connected to ${formData.displayName}`,
      })

      onSuccess?.()
    } catch (err: any) {
      let userFriendlyMessage = 'Failed to connect exchange'
      let errorDetails = ''

      // Extract error message from ApiError or other error types
      const errorMessage = err?.message || err?.response?.data?.message || 'Unknown error'
      
      // Provide user-friendly messages for common scenarios
      if (errorMessage.includes('already exists')) {
        userFriendlyMessage = `${formData.exchange.charAt(0).toUpperCase() + formData.exchange.slice(1)} connection already exists`
        errorDetails = 'Your credentials have been updated for the existing connection.'
      } else if (errorMessage.includes('Invalid API credentials') || errorMessage.includes('Authentication failed')) {
        userFriendlyMessage = 'Invalid API credentials'
        errorDetails = 'Please check your API key and secret are correct.'
      } else if (errorMessage.includes('Connection test failed')) {
        userFriendlyMessage = 'Connection test failed'
        errorDetails = 'Unable to connect to the exchange. Check your credentials and network connection.'
      } else if (errorMessage.includes('Unsupported exchange')) {
        userFriendlyMessage = 'Unsupported exchange'
        errorDetails = 'This exchange is not currently supported by our platform.'
      } else if (errorMessage.includes('Password') || errorMessage.includes('password')) {
        userFriendlyMessage = 'Invalid password'
        errorDetails = 'Please enter your correct account password to encrypt the API credentials.'
      } else if (errorMessage.includes('Database operation failed') || errorMessage.includes('Internal server error')) {
        userFriendlyMessage = 'Server error'
        errorDetails = 'Please try again in a moment. If the problem persists, contact support.'
      } else if (errorMessage.includes('Network Error') || errorMessage.includes('fetch')) {
        userFriendlyMessage = 'Connection error'
        errorDetails = 'Unable to reach the server. Please check your internet connection.'
      } else {
        userFriendlyMessage = errorMessage || 'Connection failed'
      }

      const displayMessage = errorDetails ? `${userFriendlyMessage}. ${errorDetails}` : userFriendlyMessage
      
      setError(displayMessage)
      toast({
        title: "Connection Failed",
        description: displayMessage,
        variant: "destructive"
      })
    } finally {
      setIsLoading(false)
    }
  }

  const handleExchangeChange = (value: string) => {
    setFormData(prev => ({
      ...prev,
      exchange: value,
      displayName: SUPPORTED_EXCHANGES.find(e => e.value === value)?.label || ''
    }))
  }

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[500px] bg-gray-950 border-gray-800">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
              <Link2 className="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <DialogTitle className="text-xl">Connect Exchange</DialogTitle>
              <DialogDescription className="text-gray-400">
                Securely connect your exchange account using API keys
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4 mt-4">
          {error && (
            <Alert variant="destructive" className="bg-red-500/10 border-red-500/30">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <Label htmlFor="exchange">Exchange</Label>
            <Select value={formData.exchange} onValueChange={handleExchangeChange}>
              <SelectTrigger id="exchange" className="bg-gray-900 border-gray-800">
                <SelectValue placeholder="Select an exchange" />
              </SelectTrigger>
              <SelectContent className="bg-gray-900 border-gray-800">
                {SUPPORTED_EXCHANGES.map(exchange => (
                  <SelectItem key={exchange.value} value={exchange.value}>
                    <span className="flex items-center gap-2">
                      <span>{exchange.icon}</span>
                      <span>{exchange.label}</span>
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="displayName">Connection Name</Label>
            <Input
              id="displayName"
              value={formData.displayName}
              onChange={(e) => setFormData(prev => ({ ...prev, displayName: e.target.value }))}
              placeholder="e.g., My Binance Account"
              className="bg-gray-900 border-gray-800"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="apiKey">API Key</Label>
            <Input
              id="apiKey"
              value={formData.apiKey}
              onChange={(e) => setFormData(prev => ({ ...prev, apiKey: e.target.value }))}
              placeholder="Enter your API key"
              className="bg-gray-900 border-gray-800"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="apiSecret">API Secret</Label>
            <div className="relative">
              <Input
                id="apiSecret"
                type={showApiSecret ? "text" : "password"}
                value={formData.apiSecret}
                onChange={(e) => setFormData(prev => ({ ...prev, apiSecret: e.target.value }))}
                placeholder="Enter your API secret"
                className="bg-gray-900 border-gray-800 pr-10"
              />
              <button
                type="button"
                onClick={() => setShowApiSecret(!showApiSecret)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-300"
              >
                {showApiSecret ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
          </div>


          <div className="space-y-2">
            <Label htmlFor="password">Your Password</Label>
            <div className="relative">
              <Input
                id="password"
                type={showPassword ? "text" : "password"}
                value={formData.password}
                onChange={(e) => setFormData(prev => ({ ...prev, password: e.target.value }))}
                placeholder="Enter your account password"
                className="bg-gray-900 border-gray-800 pr-10"
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-300"
              >
                {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
            <p className="text-xs text-gray-400">
              Your password is used to encrypt the API credentials
            </p>
          </div>

          <Alert className="border-purple-500/30 bg-purple-500/10">
            <Shield className="h-4 w-4 text-purple-400" />
            <AlertDescription className="text-purple-200 text-sm">
              <strong>Security Note:</strong> Your API credentials are encrypted with your password
              and stored securely. We recommend using read-only API keys with IP restrictions.
            </AlertDescription>
          </Alert>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              disabled={isLoading}
              className="bg-transparent border-gray-700 hover:bg-gray-900"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isLoading}
              className="bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600"
            >
              {isLoading ? 'Connecting...' : 'Connect Exchange'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}