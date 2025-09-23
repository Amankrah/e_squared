"use client"

import { useState } from "react"
import { Key, Plus, Eye, EyeOff, Copy, Trash2, RotateCcw, Shield, AlertTriangle, CheckCircle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

interface ApiKey {
  id: string
  name: string
  exchange: string
  permissions: string[]
  created: string
  lastUsed: string
  status: "active" | "expired" | "revoked"
  key: string
  secret: string
}

export function ApiKeysSettings() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([
    {
      id: "1",
      name: "Binance Trading Bot",
      exchange: "binance",
      permissions: ["Read Account", "Spot Trading"],
      created: "2024-01-15",
      lastUsed: "2024-01-20",
      status: "active",
      key: "abc123***hidden***",
      secret: "***hidden***"
    },
    {
      id: "2",
      name: "Coinbase Portfolio",
      exchange: "coinbase",
      permissions: ["Read Account"],
      created: "2024-01-10",
      lastUsed: "Never",
      status: "expired",
      key: "def456***hidden***",
      secret: "***hidden***"
    }
  ])

  const [showKeys, setShowKeys] = useState<{[key: string]: boolean}>({})
  const [isCreating, setIsCreating] = useState(false)
  const [newKey, setNewKey] = useState({
    name: "",
    exchange: "binance",
    permissions: [] as string[]
  })

  const exchanges = [
    { value: "binance", name: "Binance", logo: "🔶" },
    { value: "coinbase", name: "Coinbase", logo: "🔵" },
    { value: "kraken", name: "Kraken", logo: "🟣" },
    { value: "bybit", name: "Bybit", logo: "🟡" }
  ]

  const permissionOptions = [
    "Read Account",
    "Spot Trading",
    "Futures Trading",
    "Margin Trading",
    "Withdraw Funds"
  ]

  const toggleKeyVisibility = (keyId: string) => {
    setShowKeys(prev => ({ ...prev, [keyId]: !prev[keyId] }))
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    // TODO: Show toast notification
  }

  const revokeKey = (keyId: string) => {
    setApiKeys(prev => prev.map(key =>
      key.id === keyId ? { ...key, status: "revoked" as const } : key
    ))
  }

  const deleteKey = (keyId: string) => {
    setApiKeys(prev => prev.filter(key => key.id !== keyId))
  }

  const rotateKey = (keyId: string) => {
    // TODO: Implement key rotation logic
    console.log("Rotating key:", keyId)
  }

  const createNewKey = () => {
    if (!newKey.name || newKey.permissions.length === 0) {
      alert("Please fill in all required fields")
      return
    }

    const key: ApiKey = {
      id: Date.now().toString(),
      name: newKey.name,
      exchange: newKey.exchange,
      permissions: newKey.permissions,
      created: new Date().toISOString().split('T')[0],
      lastUsed: "Never",
      status: "active",
      key: "new_key_***generated***",
      secret: "***generated***"
    }

    setApiKeys(prev => [...prev, key])
    setNewKey({ name: "", exchange: "binance", permissions: [] })
    setIsCreating(false)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case "active": return "text-emerald-400 bg-emerald-500/20 border-emerald-400/20"
      case "expired": return "text-amber-400 bg-amber-500/20 border-amber-400/20"
      case "revoked": return "text-red-400 bg-red-500/20 border-red-400/20"
      default: return "text-gray-400 bg-gray-500/20 border-gray-400/20"
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "active": return <CheckCircle className="w-4 h-4" />
      case "expired": return <AlertTriangle className="w-4 h-4" />
      case "revoked": return <Shield className="w-4 h-4" />
      default: return <Key className="w-4 h-4" />
    }
  }

  return (
    <div className="space-y-6">
      {/* API Keys Overview */}
      <Card className="border-2 border-[rgba(34,197,94,0.2)] bg-gradient-to-br from-[rgba(34,197,94,0.1)] to-[rgba(34,197,94,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="p-3 bg-gradient-to-br from-green-500/30 to-emerald-500/30 rounded-xl backdrop-blur-sm border border-green-400/20">
                <Key className="w-6 h-6 text-green-200" />
              </div>
              <div>
                <CardTitle className="text-xl font-bold text-white">API Key Management</CardTitle>
                <CardDescription className="text-green-300">
                  Securely manage your exchange API credentials
                </CardDescription>
              </div>
            </div>
            <Button
              onClick={() => setIsCreating(true)}
              className="bg-gradient-to-r from-green-600/80 to-emerald-600/80 hover:from-green-500/90 hover:to-emerald-500/90 text-white"
            >
              <Plus className="w-4 h-4 mr-2" />
              Add API Key
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4">
            <div className="p-4 bg-green-500/10 border border-green-400/20 rounded-xl text-center">
              <p className="text-2xl font-bold text-green-400">
                {apiKeys.filter(k => k.status === 'active').length}
              </p>
              <p className="text-sm text-green-300">Active Keys</p>
            </div>
            <div className="p-4 bg-amber-500/10 border border-amber-400/20 rounded-xl text-center">
              <p className="text-2xl font-bold text-amber-400">
                {apiKeys.filter(k => k.status === 'expired').length}
              </p>
              <p className="text-sm text-amber-300">Expired Keys</p>
            </div>
            <div className="p-4 bg-blue-500/10 border border-blue-400/20 rounded-xl text-center">
              <p className="text-2xl font-bold text-blue-400">{apiKeys.length}</p>
              <p className="text-sm text-blue-300">Total Keys</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Create New API Key */}
      {isCreating && (
        <Card className="border-2 border-[rgba(34,197,94,0.2)] bg-gradient-to-br from-[rgba(34,197,94,0.1)] to-[rgba(34,197,94,0.02)] backdrop-blur-xl">
          <CardHeader>
            <CardTitle className="text-xl font-bold text-white">Create New API Key</CardTitle>
            <CardDescription className="text-green-300">
              Add a new exchange API key for trading automation
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Key Name */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-green-200">Key Name</label>
              <input
                type="text"
                value={newKey.name}
                onChange={(e) => setNewKey({...newKey, name: e.target.value})}
                placeholder="e.g., Binance Trading Bot"
                className="w-full p-3 bg-green-500/10 border border-green-400/20 rounded-xl text-white placeholder-green-300 focus:outline-none focus:ring-2 focus:ring-green-500/50 backdrop-blur-sm"
              />
            </div>

            {/* Exchange Selection */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-green-200">Exchange</label>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {exchanges.map((exchange) => (
                  <button
                    key={exchange.value}
                    onClick={() => setNewKey({...newKey, exchange: exchange.value})}
                    className={`p-3 rounded-xl border transition-all duration-300 ${
                      newKey.exchange === exchange.value
                        ? 'border-green-400/50 bg-green-500/20'
                        : 'border-green-400/20 bg-green-500/5 hover:bg-green-500/10'
                    }`}
                  >
                    <div className="text-2xl mb-1">{exchange.logo}</div>
                    <p className="text-sm font-medium text-white">{exchange.name}</p>
                  </button>
                ))}
              </div>
            </div>

            {/* Permissions */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-green-200">Permissions</label>
              <div className="grid md:grid-cols-2 gap-2">
                {permissionOptions.map((permission) => (
                  <label key={permission} className="flex items-center space-x-3 p-3 bg-green-500/5 border border-green-400/10 rounded-xl hover:bg-green-500/10 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={newKey.permissions.includes(permission)}
                      onChange={(e) => {
                        if (e.target.checked) {
                          setNewKey({...newKey, permissions: [...newKey.permissions, permission]})
                        } else {
                          setNewKey({...newKey, permissions: newKey.permissions.filter(p => p !== permission)})
                        }
                      }}
                      className="w-4 h-4 text-green-600 bg-green-500/20 border-green-400/30 rounded focus:ring-green-500/50"
                    />
                    <span className="text-sm text-white">{permission}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* Actions */}
            <div className="flex justify-end space-x-3 pt-4">
              <Button
                onClick={() => setIsCreating(false)}
                variant="outline"
                className="border-gray-400/30 text-gray-200 hover:bg-gray-500/20"
              >
                Cancel
              </Button>
              <Button
                onClick={createNewKey}
                className="bg-gradient-to-r from-green-600/80 to-emerald-600/80 hover:from-green-500/90 hover:to-emerald-500/90 text-white"
              >
                Create API Key
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Existing API Keys */}
      <div className="space-y-4">
        {apiKeys.map((apiKey) => (
          <Card key={apiKey.id} className="border-2 border-[rgba(34,197,94,0.2)] bg-gradient-to-br from-[rgba(34,197,94,0.1)] to-[rgba(34,197,94,0.02)] backdrop-blur-xl">
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className="text-2xl">
                    {exchanges.find(e => e.value === apiKey.exchange)?.logo}
                  </div>
                  <div>
                    <CardTitle className="text-lg font-bold text-white">{apiKey.name}</CardTitle>
                    <CardDescription className="text-green-300">
                      {exchanges.find(e => e.value === apiKey.exchange)?.name} • Created {apiKey.created}
                    </CardDescription>
                  </div>
                </div>
                <div className={`px-3 py-1 rounded-lg border text-sm font-medium flex items-center space-x-1 ${getStatusColor(apiKey.status)}`}>
                  {getStatusIcon(apiKey.status)}
                  <span className="capitalize">{apiKey.status}</span>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Permissions */}
              <div>
                <p className="text-sm font-medium text-green-200 mb-2">Permissions</p>
                <div className="flex flex-wrap gap-2">
                  {apiKey.permissions.map((permission) => (
                    <span key={permission} className="px-2 py-1 bg-green-500/20 border border-green-400/20 rounded-lg text-xs text-green-300">
                      {permission}
                    </span>
                  ))}
                </div>
              </div>

              {/* API Credentials */}
              <div className="space-y-3">
                <div>
                  <p className="text-sm font-medium text-green-200 mb-1">API Key</p>
                  <div className="flex items-center space-x-2">
                    <input
                      type={showKeys[apiKey.id] ? "text" : "password"}
                      value={apiKey.key}
                      readOnly
                      className="flex-1 p-2 bg-green-500/10 border border-green-400/20 rounded-lg text-white text-sm"
                    />
                    <Button
                      onClick={() => toggleKeyVisibility(apiKey.id)}
                      variant="outline"
                      size="sm"
                      className="border-green-400/30 text-green-200"
                    >
                      {showKeys[apiKey.id] ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </Button>
                    <Button
                      onClick={() => copyToClipboard(apiKey.key)}
                      variant="outline"
                      size="sm"
                      className="border-green-400/30 text-green-200"
                    >
                      <Copy className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                <div>
                  <p className="text-sm font-medium text-green-200 mb-1">Secret Key</p>
                  <div className="flex items-center space-x-2">
                    <input
                      type="password"
                      value={apiKey.secret}
                      readOnly
                      className="flex-1 p-2 bg-green-500/10 border border-green-400/20 rounded-lg text-white text-sm"
                    />
                    <Button
                      onClick={() => copyToClipboard(apiKey.secret)}
                      variant="outline"
                      size="sm"
                      className="border-green-400/30 text-green-200"
                    >
                      <Copy className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </div>

              {/* Last Used */}
              <div>
                <p className="text-sm text-green-300">Last used: {apiKey.lastUsed}</p>
              </div>

              {/* Actions */}
              <div className="flex justify-end space-x-2 pt-4 border-t border-green-400/20">
                {apiKey.status === 'active' && (
                  <>
                    <Button
                      onClick={() => rotateKey(apiKey.id)}
                      variant="outline"
                      size="sm"
                      className="border-amber-400/30 text-amber-200 hover:bg-amber-500/20"
                    >
                      <RotateCcw className="w-4 h-4 mr-1" />
                      Rotate
                    </Button>
                    <Button
                      onClick={() => revokeKey(apiKey.id)}
                      variant="outline"
                      size="sm"
                      className="border-red-400/30 text-red-200 hover:bg-red-500/20"
                    >
                      <Shield className="w-4 h-4 mr-1" />
                      Revoke
                    </Button>
                  </>
                )}
                <Button
                  onClick={() => deleteKey(apiKey.id)}
                  variant="outline"
                  size="sm"
                  className="border-red-400/30 text-red-200 hover:bg-red-500/20"
                >
                  <Trash2 className="w-4 h-4 mr-1" />
                  Delete
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Security Notice */}
      <Card className="border-2 border-[rgba(255,193,7,0.2)] bg-gradient-to-br from-[rgba(255,193,7,0.1)] to-[rgba(255,193,7,0.02)] backdrop-blur-xl">
        <CardContent className="p-6">
          <div className="flex items-start space-x-3">
            <AlertTriangle className="w-6 h-6 text-amber-400 mt-1" />
            <div>
              <h3 className="font-semibold text-amber-300 mb-2">Security Best Practices</h3>
              <ul className="space-y-1 text-sm text-amber-200">
                <li>• Only grant the minimum permissions required for your trading strategy</li>
                <li>• Regularly rotate your API keys for enhanced security</li>
                <li>• Never share your API keys or secret keys with third parties</li>
                <li>• Monitor API key usage and revoke unused or suspicious keys</li>
                <li>• Enable IP whitelisting on your exchange accounts when possible</li>
              </ul>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}