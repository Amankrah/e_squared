"use client"

import { useState } from "react"
import { Database, Download, Trash2, Shield, Eye, EyeOff, FileText, Calendar, Archive, AlertTriangle } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

export function DataPrivacySettings() {
  const [privacySettings, setPrivacySettings] = useState({
    dataCollection: {
      analytics: true,
      performance: true,
      errors: true,
      usage: false
    },
    dataRetention: "1year",
    thirdPartySharing: false,
    marketingEmails: false,
    dataProcessing: true
  })

  const [isExporting, setIsExporting] = useState(false)
  const [isDeletingAccount, setIsDeletingAccount] = useState(false)

  const updatePrivacySetting = (category: string, key: string, value: boolean) => {
    setPrivacySettings(prev => ({
      ...prev,
      [category]: {
        ...prev[category as keyof typeof prev] as any,
        [key]: value
      }
    }))
  }

  const updateRetentionPeriod = (period: string) => {
    setPrivacySettings(prev => ({ ...prev, dataRetention: period }))
  }

  const exportData = async () => {
    setIsExporting(true)
    // TODO: Implement data export functionality
    setTimeout(() => {
      setIsExporting(false)
      console.log("Data export initiated")
    }, 2000)
  }

  const deleteAccount = () => {
    // TODO: Implement account deletion
    console.log("Account deletion requested")
  }

  const savePrivacySettings = () => {
    // TODO: Implement API call to save privacy settings
    console.log("Privacy settings saved:", privacySettings)
  }

  const dataTypes = [
    {
      key: "analytics",
      title: "Analytics Data",
      description: "Usage patterns and feature interactions",
      icon: Database,
      color: "text-blue-300"
    },
    {
      key: "performance",
      title: "Performance Data",
      description: "App performance metrics and error reports",
      icon: FileText,
      color: "text-emerald-300"
    },
    {
      key: "errors",
      title: "Error Reports",
      description: "Crash reports and debugging information",
      icon: AlertTriangle,
      color: "text-amber-300"
    },
    {
      key: "usage",
      title: "Usage Statistics",
      description: "Detailed feature usage and behavior tracking",
      icon: Eye,
      color: "text-purple-300"
    }
  ]

  const retentionOptions = [
    { value: "3months", label: "3 Months", description: "Minimal retention period" },
    { value: "6months", label: "6 Months", description: "Standard retention" },
    { value: "1year", label: "1 Year", description: "Recommended for analytics" },
    { value: "2years", label: "2 Years", description: "Extended retention" },
    { value: "indefinite", label: "Indefinite", description: "Keep until manually deleted" }
  ]

  return (
    <div className="space-y-6">
      {/* Privacy Overview */}
      <Card className="border-2 border-[rgba(168,85,247,0.2)] bg-gradient-to-br from-[rgba(168,85,247,0.1)] to-[rgba(168,85,247,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-gradient-to-br from-violet-500/30 to-purple-500/30 rounded-xl backdrop-blur-sm border border-violet-400/20">
              <Database className="w-6 h-6 text-violet-200" />
            </div>
            <div>
              <CardTitle className="text-xl font-bold text-white">Data & Privacy Control</CardTitle>
              <CardDescription className="text-violet-300">
                Manage your data collection, storage, and privacy preferences
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4">
            <div className="p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl text-center">
              <Shield className="w-8 h-8 text-violet-300 mx-auto mb-2" />
              <p className="font-medium text-white">Privacy Protected</p>
              <p className="text-sm text-violet-300">GDPR Compliant</p>
            </div>
            <div className="p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl text-center">
              <Database className="w-8 h-8 text-violet-300 mx-auto mb-2" />
              <p className="font-medium text-white">Data Encrypted</p>
              <p className="text-sm text-violet-300">End-to-end security</p>
            </div>
            <div className="p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl text-center">
              <Eye className="w-8 h-8 text-violet-300 mx-auto mb-2" />
              <p className="font-medium text-white">Full Transparency</p>
              <p className="text-sm text-violet-300">You control your data</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Collection Settings */}
      <Card className="border-2 border-[rgba(168,85,247,0.2)] bg-gradient-to-br from-[rgba(168,85,247,0.1)] to-[rgba(168,85,247,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Data Collection Preferences</CardTitle>
          <CardDescription className="text-violet-300">
            Choose what data we can collect to improve your experience
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {dataTypes.map((dataType) => {
              const Icon = dataType.icon
              return (
                <div key={dataType.key} className="flex items-center justify-between p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl">
                  <div className="flex items-center space-x-3">
                    <Icon className={`w-5 h-5 ${dataType.color}`} />
                    <div>
                      <p className="font-medium text-white">{dataType.title}</p>
                      <p className="text-sm text-violet-300">{dataType.description}</p>
                    </div>
                  </div>
                  <button
                    onClick={() => updatePrivacySetting('dataCollection', dataType.key, !privacySettings.dataCollection[dataType.key as keyof typeof privacySettings.dataCollection])}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                      privacySettings.dataCollection[dataType.key as keyof typeof privacySettings.dataCollection]
                        ? 'bg-gradient-to-r from-violet-600 to-purple-600'
                        : 'bg-gray-600'
                    }`}
                  >
                    <span
                      className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                        privacySettings.dataCollection[dataType.key as keyof typeof privacySettings.dataCollection] ? 'translate-x-6' : 'translate-x-1'
                      }`}
                    />
                  </button>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>

      {/* Data Retention */}
      <Card className="border-2 border-[rgba(168,85,247,0.2)] bg-gradient-to-br from-[rgba(168,85,247,0.1)] to-[rgba(168,85,247,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Data Retention Period</CardTitle>
          <CardDescription className="text-violet-300">
            How long should we keep your data?
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-3">
            {retentionOptions.map((option) => (
              <button
                key={option.value}
                onClick={() => updateRetentionPeriod(option.value)}
                className={`p-4 rounded-xl border text-left transition-all duration-300 ${
                  privacySettings.dataRetention === option.value
                    ? 'border-violet-400/50 bg-violet-500/20'
                    : 'border-violet-400/20 bg-violet-500/5 hover:bg-violet-500/10'
                }`}
              >
                <div className="flex items-center space-x-2 mb-2">
                  <Calendar className="w-4 h-4 text-violet-300" />
                  <p className="font-medium text-white">{option.label}</p>
                </div>
                <p className="text-sm text-violet-300">{option.description}</p>
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Privacy Controls */}
      <Card className="border-2 border-[rgba(168,85,247,0.2)] bg-gradient-to-br from-[rgba(168,85,247,0.1)] to-[rgba(168,85,247,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Additional Privacy Controls</CardTitle>
          <CardDescription className="text-violet-300">
            Fine-tune your privacy and communication preferences
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl">
            <div>
              <p className="font-medium text-white">Third-Party Data Sharing</p>
              <p className="text-sm text-violet-300">Allow sharing anonymized data with partners</p>
            </div>
            <button
              onClick={() => setPrivacySettings(prev => ({ ...prev, thirdPartySharing: !prev.thirdPartySharing }))}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                privacySettings.thirdPartySharing
                  ? 'bg-gradient-to-r from-violet-600 to-purple-600'
                  : 'bg-gray-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  privacySettings.thirdPartySharing ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl">
            <div>
              <p className="font-medium text-white">Marketing Communications</p>
              <p className="text-sm text-violet-300">Receive promotional emails and updates</p>
            </div>
            <button
              onClick={() => setPrivacySettings(prev => ({ ...prev, marketingEmails: !prev.marketingEmails }))}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                privacySettings.marketingEmails
                  ? 'bg-gradient-to-r from-violet-600 to-purple-600'
                  : 'bg-gray-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  privacySettings.marketingEmails ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between p-4 bg-violet-500/10 border border-violet-400/20 rounded-xl">
            <div>
              <p className="font-medium text-white">Data Processing for AI</p>
              <p className="text-sm text-violet-300">Use data to improve AI recommendations</p>
            </div>
            <button
              onClick={() => setPrivacySettings(prev => ({ ...prev, dataProcessing: !prev.dataProcessing }))}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                privacySettings.dataProcessing
                  ? 'bg-gradient-to-r from-violet-600 to-purple-600'
                  : 'bg-gray-600'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  privacySettings.dataProcessing ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </CardContent>
      </Card>

      {/* Data Management */}
      <Card className="border-2 border-[rgba(168,85,247,0.2)] bg-gradient-to-br from-[rgba(168,85,247,0.1)] to-[rgba(168,85,247,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Data Management</CardTitle>
          <CardDescription className="text-violet-300">
            Export or delete your personal data
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Export Data */}
          <div className="p-4 bg-blue-500/10 border border-blue-400/20 rounded-xl">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-3">
                <Download className="w-5 h-5 text-blue-300" />
                <div>
                  <p className="font-medium text-white">Export Your Data</p>
                  <p className="text-sm text-blue-300">Download all your personal data in JSON format</p>
                </div>
              </div>
              <Button
                onClick={exportData}
                disabled={isExporting}
                className="bg-gradient-to-r from-blue-600/80 to-cyan-600/80 hover:from-blue-500/90 hover:to-cyan-500/90 text-white"
              >
                {isExporting ? (
                  <>
                    <Archive className="w-4 h-4 mr-2 animate-spin" />
                    Exporting...
                  </>
                ) : (
                  <>
                    <Download className="w-4 h-4 mr-2" />
                    Export Data
                  </>
                )}
              </Button>
            </div>
            <p className="text-xs text-blue-200">
              Includes: Profile information, trading history, settings, and preferences
            </p>
          </div>

          {/* Delete Account */}
          <div className="p-4 bg-red-500/10 border border-red-400/20 rounded-xl">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-3">
                <Trash2 className="w-5 h-5 text-red-300" />
                <div>
                  <p className="font-medium text-white">Delete Account</p>
                  <p className="text-sm text-red-300">Permanently delete your account and all data</p>
                </div>
              </div>
              <Button
                onClick={() => setIsDeletingAccount(true)}
                variant="outline"
                className="border-red-400/30 text-red-200 hover:bg-red-500/20"
              >
                <Trash2 className="w-4 h-4 mr-2" />
                Delete Account
              </Button>
            </div>
            <div className="p-3 bg-red-500/20 border border-red-400/30 rounded-lg">
              <div className="flex items-start space-x-2">
                <AlertTriangle className="w-4 h-4 text-red-400 mt-0.5" />
                <div className="text-xs text-red-200">
                  <p className="font-medium mb-1">This action cannot be undone!</p>
                  <p>All your data, including trading history, API keys, and settings will be permanently deleted.</p>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Delete Account Confirmation */}
      {isDeletingAccount && (
        <Card className="border-2 border-[rgba(255,0,0,0.3)] bg-gradient-to-br from-[rgba(255,0,0,0.15)] to-[rgba(255,0,0,0.02)] backdrop-blur-xl">
          <CardHeader>
            <div className="flex items-center space-x-3">
              <AlertTriangle className="w-6 h-6 text-red-400" />
              <CardTitle className="text-xl font-bold text-red-300">Confirm Account Deletion</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-red-200">
              Please type <strong>"DELETE MY ACCOUNT"</strong> to confirm permanent deletion:
            </p>
            <input
              type="text"
              placeholder="Type DELETE MY ACCOUNT"
              className="w-full p-3 bg-red-500/10 border border-red-400/20 rounded-xl text-white placeholder-red-300 focus:outline-none focus:ring-2 focus:ring-red-500/50 backdrop-blur-sm"
            />
            <div className="flex justify-end space-x-3">
              <Button
                onClick={() => setIsDeletingAccount(false)}
                variant="outline"
                className="border-gray-400/30 text-gray-200"
              >
                Cancel
              </Button>
              <Button
                onClick={deleteAccount}
                className="bg-gradient-to-r from-red-600/80 to-red-600/80 hover:from-red-500/90 hover:to-red-500/90 text-white"
              >
                <Trash2 className="w-4 h-4 mr-2" />
                Delete Account Permanently
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Save Settings */}
      <div className="flex justify-end">
        <Button
          onClick={savePrivacySettings}
          className="bg-gradient-to-r from-violet-600/80 to-purple-600/80 hover:from-violet-500/90 hover:to-purple-500/90 text-white px-8"
        >
          Save Privacy Settings
        </Button>
      </div>
    </div>
  )
}