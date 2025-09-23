"use client"

import { useState } from "react"
import { Bell, Mail, Smartphone, DollarSign, TrendingUp, AlertTriangle, Volume2, VolumeX } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

export function NotificationsSettings() {
  const [notifications, setNotifications] = useState({
    email: {
      trades: true,
      profits: true,
      losses: true,
      connections: true,
      security: true,
      marketing: false
    },
    push: {
      trades: true,
      profits: true,
      losses: true,
      connections: false,
      security: true,
      marketing: false
    },
    sound: {
      enabled: true,
      volume: 50
    }
  })

  const updateEmailSetting = (key: string, value: boolean) => {
    setNotifications(prev => ({
      ...prev,
      email: { ...prev.email, [key]: value }
    }))
  }

  const updatePushSetting = (key: string, value: boolean) => {
    setNotifications(prev => ({
      ...prev,
      push: { ...prev.push, [key]: value }
    }))
  }

  const updateSoundSetting = (key: string, value: any) => {
    setNotifications(prev => ({
      ...prev,
      sound: { ...prev.sound, [key]: value }
    }))
  }

  const saveSettings = () => {
    // TODO: Implement API call to save notification settings
    console.log("Notification settings saved:", notifications)
  }

  const notificationTypes = [
    {
      key: "trades",
      icon: TrendingUp,
      title: "Trade Executions",
      description: "Get notified when trades are executed",
      color: "text-blue-300"
    },
    {
      key: "profits",
      icon: DollarSign,
      title: "Profit Alerts",
      description: "Notifications for profitable trades",
      color: "text-emerald-300"
    },
    {
      key: "losses",
      icon: AlertTriangle,
      title: "Loss Alerts",
      description: "Notifications for losing trades or stop-losses",
      color: "text-red-300"
    },
    {
      key: "connections",
      icon: Bell,
      title: "Exchange Status",
      description: "Connection issues and exchange updates",
      color: "text-purple-300"
    },
    {
      key: "security",
      icon: AlertTriangle,
      title: "Security Alerts",
      description: "Login attempts and security events",
      color: "text-amber-300"
    },
    {
      key: "marketing",
      icon: Mail,
      title: "Marketing & Updates",
      description: "Product updates and promotional content",
      color: "text-gray-300"
    }
  ]

  return (
    <div className="space-y-6">
      {/* Notification Overview */}
      <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-xl backdrop-blur-sm border border-purple-400/20">
              <Bell className="w-6 h-6 text-purple-200" />
            </div>
            <div>
              <CardTitle className="text-xl font-bold text-white">Notification Preferences</CardTitle>
              <CardDescription className="text-purple-300">
                Configure how and when you receive trading alerts and updates
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4">
            <div className="p-4 bg-purple-500/10 border border-purple-400/20 rounded-xl text-center">
              <Mail className="w-8 h-8 text-purple-300 mx-auto mb-2" />
              <p className="font-medium text-white">Email</p>
              <p className="text-sm text-purple-300">
                {Object.values(notifications.email).filter(Boolean).length} active
              </p>
            </div>
            <div className="p-4 bg-purple-500/10 border border-purple-400/20 rounded-xl text-center">
              <Smartphone className="w-8 h-8 text-purple-300 mx-auto mb-2" />
              <p className="font-medium text-white">Push</p>
              <p className="text-sm text-purple-300">
                {Object.values(notifications.push).filter(Boolean).length} active
              </p>
            </div>
            <div className="p-4 bg-purple-500/10 border border-purple-400/20 rounded-xl text-center">
              {notifications.sound.enabled ? (
                <Volume2 className="w-8 h-8 text-purple-300 mx-auto mb-2" />
              ) : (
                <VolumeX className="w-8 h-8 text-purple-300 mx-auto mb-2" />
              )}
              <p className="font-medium text-white">Sound</p>
              <p className="text-sm text-purple-300">
                {notifications.sound.enabled ? `${notifications.sound.volume}%` : "Disabled"}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Notification Types */}
      <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Notification Types</CardTitle>
          <CardDescription className="text-purple-300">
            Choose which events trigger notifications
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid md:grid-cols-3 gap-4 p-4 bg-purple-500/5 border border-purple-400/10 rounded-xl">
              <div className="font-medium text-purple-200">Notification Type</div>
              <div className="font-medium text-purple-200 text-center">Email</div>
              <div className="font-medium text-purple-200 text-center">Push</div>
            </div>

            {notificationTypes.map((type) => {
              const Icon = type.icon
              return (
                <div key={type.key} className="grid md:grid-cols-3 gap-4 p-4 bg-purple-500/5 border border-purple-400/10 rounded-xl hover:bg-purple-500/10 transition-colors">
                  <div className="flex items-center space-x-3">
                    <Icon className={`w-5 h-5 ${type.color}`} />
                    <div>
                      <p className="font-medium text-white">{type.title}</p>
                      <p className="text-sm text-gray-300">{type.description}</p>
                    </div>
                  </div>

                  <div className="flex justify-center">
                    <button
                      onClick={() => updateEmailSetting(type.key, !notifications.email[type.key as keyof typeof notifications.email])}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        notifications.email[type.key as keyof typeof notifications.email]
                          ? 'bg-gradient-to-r from-purple-600 to-pink-600'
                          : 'bg-gray-600'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          notifications.email[type.key as keyof typeof notifications.email] ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>

                  <div className="flex justify-center">
                    <button
                      onClick={() => updatePushSetting(type.key, !notifications.push[type.key as keyof typeof notifications.push])}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        notifications.push[type.key as keyof typeof notifications.push]
                          ? 'bg-gradient-to-r from-purple-600 to-pink-600'
                          : 'bg-gray-600'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          notifications.push[type.key as keyof typeof notifications.push] ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>

      {/* Sound Settings */}
      <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Sound Preferences</CardTitle>
          <CardDescription className="text-purple-300">
            Configure notification sounds and volume levels
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Sound Toggle */}
          <div className="flex items-center justify-between p-4 bg-purple-500/10 border border-purple-400/20 rounded-xl">
            <div className="flex items-center space-x-3">
              {notifications.sound.enabled ? (
                <Volume2 className="w-6 h-6 text-purple-300" />
              ) : (
                <VolumeX className="w-6 h-6 text-purple-300" />
              )}
              <div>
                <p className="font-medium text-white">Sound Notifications</p>
                <p className="text-sm text-purple-300">Play sounds for important alerts</p>
              </div>
            </div>
            <button
              onClick={() => updateSoundSetting('enabled', !notifications.sound.enabled)}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                notifications.sound.enabled
                  ? 'bg-gradient-to-r from-purple-600 to-pink-600'
                  : 'bg-gray-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  notifications.sound.enabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          {/* Volume Control */}
          {notifications.sound.enabled && (
            <div className="space-y-3">
              <label className="text-sm font-medium text-purple-200">Volume Level</label>
              <div className="flex items-center space-x-4">
                <VolumeX className="w-4 h-4 text-purple-300" />
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={notifications.sound.volume}
                  onChange={(e) => updateSoundSetting('volume', parseInt(e.target.value))}
                  className="flex-1 h-2 bg-purple-500/20 rounded-lg appearance-none cursor-pointer slider"
                />
                <Volume2 className="w-4 h-4 text-purple-300" />
                <span className="text-sm text-purple-300 w-12">{notifications.sound.volume}%</span>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Quick Actions</CardTitle>
          <CardDescription className="text-purple-300">
            Bulk notification management options
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 gap-4">
            <Button
              onClick={() => {
                const allTrue = Object.fromEntries(Object.keys(notifications.email).map(k => [k, true]))
                setNotifications(prev => ({ ...prev, email: allTrue }))
              }}
              variant="outline"
              className="border-emerald-400/30 text-emerald-200 hover:bg-emerald-500/20"
            >
              Enable All Email
            </Button>
            <Button
              onClick={() => {
                const allFalse = Object.fromEntries(Object.keys(notifications.email).map(k => [k, false]))
                setNotifications(prev => ({ ...prev, email: allFalse }))
              }}
              variant="outline"
              className="border-red-400/30 text-red-200 hover:bg-red-500/20"
            >
              Disable All Email
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Save Button */}
      <div className="flex justify-end">
        <Button
          onClick={saveSettings}
          className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white px-8"
        >
          Save Notification Settings
        </Button>
      </div>
    </div>
  )
}