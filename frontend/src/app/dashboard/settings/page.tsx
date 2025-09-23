"use client"

import React, { useState } from "react"
import { User, Shield, Bell, Palette, Database, Key } from "lucide-react"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { ProfileSettings } from "@/components/settings/profile-settings"
import { SecuritySettings } from "@/components/settings/security-settings"
import { NotificationsSettings } from "@/components/settings/notifications-settings"
import { ApiKeysSettings } from "@/components/settings/api-keys-settings"
import { DataPrivacySettings } from "@/components/settings/data-privacy-settings"

export default function SettingsPage() {
  const [activeSection, setActiveSection] = useState("profile")

  const settingsSections = [
    {
      id: "profile",
      name: "Profile",
      icon: User,
      description: "Manage your personal information and account details",
      color: "from-blue-600 to-cyan-600",
      borderColor: "border-blue-400/30",
      component: ProfileSettings
    },
    {
      id: "security",
      name: "Security",
      icon: Shield,
      description: "Two-factor authentication and password management",
      color: "from-emerald-600 to-teal-600",
      borderColor: "border-emerald-400/30",
      component: SecuritySettings
    },
    {
      id: "notifications",
      name: "Notifications",
      icon: Bell,
      description: "Configure trading alerts and email preferences",
      color: "from-purple-600 to-pink-600",
      borderColor: "border-purple-400/30",
      component: NotificationsSettings
    },
    {
      id: "api-keys",
      name: "API Keys",
      icon: Key,
      description: "Manage and rotate your exchange API credentials",
      color: "from-green-600 to-emerald-600",
      borderColor: "border-green-400/30",
      component: ApiKeysSettings
    },
    {
      id: "data-privacy",
      name: "Data & Privacy",
      icon: Database,
      description: "Control your data, exports, and privacy settings",
      color: "from-violet-600 to-purple-600",
      borderColor: "border-violet-400/30",
      component: DataPrivacySettings
    }
  ]

  // Handle URL parameters for direct section navigation
  React.useEffect(() => {
    if (typeof window !== 'undefined') {
      const urlParams = new URLSearchParams(window.location.search)
      const section = urlParams.get('section')
      if (section && settingsSections.find(s => s.id === section)) {
        setActiveSection(section)
      }
    }
  }, [settingsSections])

  const ActiveComponent = settingsSections.find(section => section.id === activeSection)?.component || ProfileSettings

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Compact Page Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-blue-100 to-blue-300 bg-clip-text text-transparent">
                Settings
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Customize your trading experience and manage your account preferences
            </p>
          </div>
        </div>

        <div className="flex flex-col lg:flex-row gap-8">
          {/* Settings Navigation */}
          <div className="lg:w-80 space-y-2">
            {settingsSections.map((section) => {
              const Icon = section.icon
              const isActive = activeSection === section.id

              return (
                <button
                  key={section.id}
                  onClick={() => setActiveSection(section.id)}
                  className={`w-full p-4 rounded-xl border-2 text-left transition-all duration-300 hover:scale-105 hover:shadow-lg ${
                    isActive
                      ? `${section.borderColor} bg-gradient-to-r ${section.color} bg-opacity-20 backdrop-blur-xl shadow-xl`
                      : 'border-gray-600/30 bg-gradient-to-r from-gray-800/50 to-gray-800/30 backdrop-blur-xl hover:border-gray-500/40'
                  }`}
                >
                  <div className="flex items-center space-x-4">
                    <div className={`p-3 rounded-xl backdrop-blur-sm border ${
                      isActive ? section.borderColor : 'border-gray-500/20'
                    } ${isActive ? `bg-gradient-to-br ${section.color} bg-opacity-30` : 'bg-gray-700/30'}`}>
                      <Icon className={`w-5 h-5 ${isActive ? 'text-white' : 'text-gray-300'}`} />
                    </div>
                    <div className="flex-1">
                      <h3 className={`font-semibold ${isActive ? 'text-white' : 'text-gray-200'}`}>
                        {section.name}
                      </h3>
                      <p className={`text-xs ${isActive ? 'text-white/70' : 'text-gray-400'}`}>
                        {section.description}
                      </p>
                    </div>
                  </div>
                </button>
              )
            })}
          </div>

          {/* Settings Content */}
          <div className="flex-1">
            <ActiveComponent />
          </div>
        </div>
      </div>
    </DashboardLayout>
  )
}