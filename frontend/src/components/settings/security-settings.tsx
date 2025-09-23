"use client"

import { useState, useEffect } from "react"
import { Shield, Key, Smartphone, Eye, EyeOff, Check, X, AlertTriangle, Save } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useAuth } from "@/contexts/auth-context"
import { apiClient, Setup2FAResponse, TwoFactorStatus, UserSession } from "@/lib/api"

export function SecuritySettings() {
  const { user } = useAuth()
  const [currentPassword, setCurrentPassword] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [showPasswords, setShowPasswords] = useState({
    current: false,
    new: false,
    confirm: false
  })
  const [twoFactorData, setTwoFactorData] = useState<TwoFactorStatus>({ enabled: false, has_secret: false })
  const [qrCodeData, setQrCodeData] = useState<Setup2FAResponse | null>(null)
  const [verificationCode, setVerificationCode] = useState("")
  const [disablePassword, setDisablePassword] = useState("")
  const [isSettingUp2FA, setIsSettingUp2FA] = useState(false)
  const [isDisabling2FA, setIsDisabling2FA] = useState(false)
  const [isChangingPassword, setIsChangingPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)
  const [activeSessions, setActiveSessions] = useState<UserSession[]>([])
  const [loadingSessions, setLoadingSessions] = useState(false)

  useEffect(() => {
    const loadData = async () => {
      if (user) {
        try {
          // Load 2FA status
          const status = await apiClient.get2FAStatus()
          setTwoFactorData(status)

          // Load active sessions
          setLoadingSessions(true)
          const sessionsResponse = await apiClient.getActiveSessions()
          setActiveSessions(sessionsResponse.sessions)
        } catch (err) {
          console.error('Failed to load data:', err)
        } finally {
          setLoadingSessions(false)
        }
      }
    }
    loadData()
  }, [user])

  const handlePasswordChange = async () => {
    if (!currentPassword.trim()) {
      setError("Current password is required")
      return
    }

    if (newPassword.length < 8) {
      setError("New password must be at least 8 characters long")
      return
    }

    if (newPassword !== confirmPassword) {
      setError("New passwords don't match")
      return
    }

    if (currentPassword === newPassword) {
      setError("New password must be different from current password")
      return
    }

    setIsLoading(true)
    setError(null)
    setSuccess(null)

    try {
      await apiClient.changePassword({
        current_password: currentPassword,
        new_password: newPassword
      })

      setSuccess("Password changed successfully")
      setCurrentPassword("")
      setNewPassword("")
      setConfirmPassword("")
      setIsChangingPassword(false)
    } catch (err: any) {
      setError(err.message || 'Failed to change password')
    } finally {
      setIsLoading(false)
    }
  }

  const handleSetup2FA = async () => {
    setIsLoading(true)
    setError(null)
    try {
      const response = await apiClient.setup2FA()
      setQrCodeData(response)
      setIsSettingUp2FA(true)
    } catch (err: any) {
      setError(err.message || 'Failed to setup 2FA')
    } finally {
      setIsLoading(false)
    }
  }

  const handleVerifySetup = async () => {
    if (!verificationCode.trim()) {
      setError('Verification code is required')
      return
    }

    setIsLoading(true)
    setError(null)
    try {
      await apiClient.verifySetup2FA({ code: verificationCode })
      setSuccess('Two-factor authentication enabled successfully')
      setTwoFactorData({ enabled: true, has_secret: true })
      setIsSettingUp2FA(false)
      setQrCodeData(null)
      setVerificationCode('')
    } catch (err: any) {
      setError(err.message || 'Invalid verification code')
    } finally {
      setIsLoading(false)
    }
  }

  const handleDisable2FA = async () => {
    if (!disablePassword.trim()) {
      setError('Password is required to disable 2FA')
      return
    }

    setIsLoading(true)
    setError(null)
    try {
      await apiClient.disable2FA({ password: disablePassword })
      setSuccess('Two-factor authentication disabled successfully')
      setTwoFactorData({ enabled: false, has_secret: false })
      setIsDisabling2FA(false)
      setDisablePassword('')
    } catch (err: any) {
      setError(err.message || 'Failed to disable 2FA')
    } finally {
      setIsLoading(false)
    }
  }

  const handleRevokeSession = async (sessionId: string) => {
    try {
      await apiClient.revokeSession(sessionId)
      setSuccess('Session revoked successfully')
      // Reload sessions
      const sessionsResponse = await apiClient.getActiveSessions()
      setActiveSessions(sessionsResponse.sessions)
    } catch (err: any) {
      setError(err.message || 'Failed to revoke session')
    }
  }

  const handleRevokeAllSessions = async () => {
    try {
      await apiClient.revokeAllSessions()
      setSuccess('All other sessions revoked successfully')
      // Reload sessions
      const sessionsResponse = await apiClient.getActiveSessions()
      setActiveSessions(sessionsResponse.sessions)
    } catch (err: any) {
      setError(err.message || 'Failed to revoke sessions')
    }
  }

  return (
    <div className="space-y-6">
      {/* Security Overview */}
      <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-xl backdrop-blur-sm border border-emerald-400/20">
              <Shield className="w-6 h-6 text-emerald-200" />
            </div>
            <div>
              <CardTitle className="text-xl font-bold text-white">Security Overview</CardTitle>
              <CardDescription className="text-emerald-300">
                Your account security status and recommendations
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid md:grid-cols-3 gap-4">
            <div className="p-4 bg-emerald-500/10 border border-emerald-400/20 rounded-xl">
              <div className="flex items-center space-x-2 mb-2">
                <Check className="w-4 h-4 text-emerald-400" />
                <span className="text-sm font-medium text-emerald-300">Strong Password</span>
              </div>
              <p className="text-xs text-emerald-200">Your password meets security requirements</p>
            </div>

            <div className="p-4 bg-amber-500/10 border border-amber-400/20 rounded-xl">
              <div className="flex items-center space-x-2 mb-2">
                {twoFactorData.enabled ? (
                  <Check className="w-4 h-4 text-emerald-400" />
                ) : (
                  <X className="w-4 h-4 text-amber-400" />
                )}
                <span className="text-sm font-medium text-amber-300">Two-Factor Auth</span>
              </div>
              <p className="text-xs text-amber-200">
                {twoFactorData.enabled ? "2FA is enabled" : "Enable 2FA for better security"}
              </p>
            </div>

            <div className="p-4 bg-blue-500/10 border border-blue-400/20 rounded-xl">
              <div className="flex items-center space-x-2 mb-2">
                <Check className="w-4 h-4 text-blue-400" />
                <span className="text-sm font-medium text-blue-300">Secure Connection</span>
              </div>
              <p className="text-xs text-blue-200">All connections are encrypted</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Password Management */}
      <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-xl font-bold text-white">Password Management</CardTitle>
              <CardDescription className="text-emerald-300">
                Change your account password and security settings
              </CardDescription>
            </div>
            <Button
              onClick={() => setIsChangingPassword(!isChangingPassword)}
              variant="outline"
              className="border-emerald-400/30 text-emerald-200 hover:bg-emerald-500/20"
            >
              <Key className="w-4 h-4 mr-2" />
              {isChangingPassword ? "Cancel" : "Change Password"}
            </Button>
          </div>
        </CardHeader>
        {isChangingPassword && (
          <CardContent className="space-y-4">
            {error && (
              <div className="p-3 bg-red-500/10 border border-red-400/20 rounded-xl text-red-300">
                {error}
              </div>
            )}

            {success && (
              <div className="p-3 bg-emerald-500/10 border border-emerald-400/20 rounded-xl text-emerald-300">
                {success}
              </div>
            )}
            {/* Current Password */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-emerald-200">Current Password</label>
              <div className="relative">
                <input
                  type={showPasswords.current ? "text" : "password"}
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  className="w-full p-3 pr-12 bg-emerald-500/10 border border-emerald-400/20 rounded-xl text-white placeholder-emerald-300 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 backdrop-blur-sm"
                  placeholder="Enter current password"
                />
                <button
                  type="button"
                  onClick={() => setShowPasswords({...showPasswords, current: !showPasswords.current})}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-emerald-300 hover:text-emerald-200"
                >
                  {showPasswords.current ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            {/* New Password */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-emerald-200">New Password</label>
              <div className="relative">
                <input
                  type={showPasswords.new ? "text" : "password"}
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  className="w-full p-3 pr-12 bg-emerald-500/10 border border-emerald-400/20 rounded-xl text-white placeholder-emerald-300 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 backdrop-blur-sm"
                  placeholder="Enter new password"
                />
                <button
                  type="button"
                  onClick={() => setShowPasswords({...showPasswords, new: !showPasswords.new})}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-emerald-300 hover:text-emerald-200"
                >
                  {showPasswords.new ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            {/* Confirm Password */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-emerald-200">Confirm New Password</label>
              <div className="relative">
                <input
                  type={showPasswords.confirm ? "text" : "password"}
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="w-full p-3 pr-12 bg-emerald-500/10 border border-emerald-400/20 rounded-xl text-white placeholder-emerald-300 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 backdrop-blur-sm"
                  placeholder="Confirm new password"
                />
                <button
                  type="button"
                  onClick={() => setShowPasswords({...showPasswords, confirm: !showPasswords.confirm})}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-emerald-300 hover:text-emerald-200"
                >
                  {showPasswords.confirm ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <div className="flex space-x-3 pt-4">
              <Button
                onClick={() => {
                  setIsChangingPassword(false)
                  setCurrentPassword("")
                  setNewPassword("")
                  setConfirmPassword("")
                  setError(null)
                  setSuccess(null)
                }}
                variant="ghost"
                className="flex-1 text-gray-300 hover:text-white hover:bg-gray-600/20"
                disabled={isLoading}
              >
                Cancel
              </Button>
              <Button
                onClick={handlePasswordChange}
                className="flex-1 bg-gradient-to-r from-emerald-600/80 to-teal-600/80 hover:from-emerald-500/90 hover:to-teal-500/90 text-white"
                disabled={isLoading || !currentPassword.trim() || !newPassword.trim() || !confirmPassword.trim()}
              >
                {isLoading ? (
                  <div className="w-4 h-4 mr-2 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                ) : (
                  <Save className="w-4 h-4 mr-2" />
                )}
                {isLoading ? 'Updating...' : 'Update Password'}
              </Button>
            </div>
          </CardContent>
        )}
      </Card>

      {/* Two-Factor Authentication */}
      <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="p-3 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-xl backdrop-blur-sm border border-emerald-400/20">
                <Smartphone className="w-6 h-6 text-emerald-200" />
              </div>
              <div>
                <CardTitle className="text-xl font-bold text-white">Two-Factor Authentication</CardTitle>
                <CardDescription className="text-emerald-300">
                  Add an extra layer of security to your account
                </CardDescription>
              </div>
            </div>
            <div className="flex items-center space-x-3">
              <span className={`text-sm font-medium ${twoFactorData.enabled ? 'text-emerald-300' : 'text-amber-300'}`}>
                {twoFactorData.enabled ? 'Enabled' : 'Disabled'}
              </span>
              <Button
                onClick={twoFactorData.enabled ? () => setIsDisabling2FA(true) : handleSetup2FA}
                className={`${
                  twoFactorData.enabled
                    ? "bg-gradient-to-r from-red-600/80 to-red-600/80 hover:from-red-500/90 hover:to-red-500/90"
                    : "bg-gradient-to-r from-emerald-600/80 to-teal-600/80 hover:from-emerald-500/90 hover:to-teal-500/90"
                } text-white`}
                disabled={isLoading || isSettingUp2FA}
              >
                {isLoading ? 'Loading...' : (twoFactorData.enabled ? 'Disable' : 'Enable')} 2FA
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isSettingUp2FA && qrCodeData ? (
            <div className="space-y-4">
              <div className="p-4 bg-blue-500/10 border border-blue-400/20 rounded-xl">
                <h3 className="font-medium text-blue-300 mb-3">Scan QR Code</h3>
                <div className="flex flex-col md:flex-row items-center space-y-4 md:space-y-0 md:space-x-6">
                  <div className="bg-white p-4 rounded-xl">
                    <img src={qrCodeData.qr_code} alt="QR Code" className="w-48 h-48" />
                  </div>
                  <div className="flex-1">
                    <p className="text-blue-200 text-sm mb-3">
                      Scan this QR code with your authenticator app, or enter the manual key:
                    </p>
                    <div className="p-3 bg-blue-500/10 border border-blue-400/20 rounded-lg font-mono text-sm text-blue-300 break-all">
                      {qrCodeData.manual_entry_key}
                    </div>
                  </div>
                </div>
              </div>

              <div className="space-y-3">
                <label className="text-sm font-medium text-blue-200">Enter 6-digit code from your app</label>
                <input
                  type="text"
                  value={verificationCode}
                  onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm text-center font-mono text-lg"
                  placeholder="000000"
                  maxLength={6}
                />

                <div className="flex space-x-3">
                  <Button
                    onClick={() => {
                      setIsSettingUp2FA(false)
                      setQrCodeData(null)
                      setVerificationCode('')
                      setError(null)
                    }}
                    variant="ghost"
                    className="flex-1 text-gray-300 hover:text-white hover:bg-gray-600/20"
                    disabled={isLoading}
                  >
                    Cancel
                  </Button>
                  <Button
                    onClick={handleVerifySetup}
                    className="flex-1 bg-gradient-to-r from-emerald-600/80 to-teal-600/80 hover:from-emerald-500/90 hover:to-teal-500/90 text-white"
                    disabled={isLoading || verificationCode.length !== 6}
                  >
                    {isLoading ? 'Verifying...' : 'Verify & Enable'}
                  </Button>
                </div>
              </div>
            </div>
          ) : isDisabling2FA ? (
            <div className="space-y-4">
              <div className="p-4 bg-red-500/10 border border-red-400/20 rounded-xl">
                <div className="flex items-center space-x-2 mb-2">
                  <AlertTriangle className="w-5 h-5 text-red-400" />
                  <span className="font-medium text-red-300">Disable Two-Factor Authentication</span>
                </div>
                <p className="text-red-200 text-sm">
                  Enter your password to confirm disabling 2FA. This will reduce your account security.
                </p>
              </div>

              <div className="space-y-3">
                <label className="text-sm font-medium text-red-200">Password</label>
                <input
                  type="password"
                  value={disablePassword}
                  onChange={(e) => setDisablePassword(e.target.value)}
                  className="w-full p-3 bg-red-500/10 border border-red-400/20 rounded-xl text-white placeholder-red-300 focus:outline-none focus:ring-2 focus:ring-red-500/50 backdrop-blur-sm"
                  placeholder="Enter your password"
                />

                <div className="flex space-x-3">
                  <Button
                    onClick={() => {
                      setIsDisabling2FA(false)
                      setDisablePassword('')
                      setError(null)
                    }}
                    variant="ghost"
                    className="flex-1 text-gray-300 hover:text-white hover:bg-gray-600/20"
                    disabled={isLoading}
                  >
                    Cancel
                  </Button>
                  <Button
                    onClick={handleDisable2FA}
                    className="flex-1 bg-gradient-to-r from-red-600/80 to-red-600/80 hover:from-red-500/90 hover:to-red-500/90 text-white"
                    disabled={isLoading || !disablePassword.trim()}
                  >
                    {isLoading ? 'Disabling...' : 'Disable 2FA'}
                  </Button>
                </div>
              </div>
            </div>
          ) : twoFactorData.enabled ? (
            <div className="p-4 bg-emerald-500/10 border border-emerald-400/20 rounded-xl">
              <div className="flex items-center space-x-2 mb-2">
                <Check className="w-5 h-5 text-emerald-400" />
                <span className="font-medium text-emerald-300">Two-Factor Authentication is Active</span>
              </div>
              <p className="text-emerald-200 text-sm">
                Your account is protected with 2FA. You'll need to enter a code from your authenticator app when signing in.
              </p>
            </div>
          ) : (
            <div className="p-4 bg-amber-500/10 border border-amber-400/20 rounded-xl">
              <div className="flex items-center space-x-2 mb-2">
                <AlertTriangle className="w-5 h-5 text-amber-400" />
                <span className="font-medium text-amber-300">Enable Two-Factor Authentication</span>
              </div>
              <p className="text-amber-200 text-sm mb-3">
                Protect your account with an additional security layer. You'll need to scan a QR code with your authenticator app.
              </p>
              <div className="space-y-2 text-sm text-amber-200">
                <p>1. Download an authenticator app (Google Authenticator, Authy, etc.)</p>
                <p>2. Click "Enable 2FA" to generate a QR code</p>
                <p>3. Scan the QR code with your authenticator app</p>
                <p>4. Enter the 6-digit code to complete setup</p>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Session Management */}
      <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Active Sessions</CardTitle>
          <CardDescription className="text-emerald-300">
            Manage your active login sessions across devices
          </CardDescription>
        </CardHeader>
        <CardContent>
          {loadingSessions ? (
            <div className="flex items-center justify-center p-8">
              <div className="w-6 h-6 border-2 border-emerald-400/30 border-t-emerald-400 rounded-full animate-spin"></div>
            </div>
          ) : (
            <>
              <div className="space-y-3">
                {activeSessions.map((session) => (
                  <div
                    key={session.id}
                    className={`flex items-center justify-between p-4 rounded-xl ${
                      session.is_current
                        ? "bg-emerald-500/10 border border-emerald-400/20"
                        : "bg-gray-500/10 border border-gray-400/20"
                    }`}
                  >
                    <div className="flex items-center space-x-3">
                      <div
                        className={`w-3 h-3 rounded-full ${
                          session.is_current ? "bg-emerald-400 animate-pulse" : "bg-gray-400"
                        }`}
                      ></div>
                      <div>
                        <p className="font-medium text-white">
                          {session.is_current ? "Current Session - " : ""}{session.platform} ({session.browser})
                        </p>
                        <p className={`text-sm ${session.is_current ? "text-emerald-300" : "text-gray-300"}`}>
                          {session.location || `IP: ${session.ip_address}`} • {
                            session.is_current
                              ? "Active now"
                              : new Date(session.last_activity).toLocaleString()
                          }
                        </p>
                      </div>
                    </div>
                    {session.is_current ? (
                      <span className="text-xs text-emerald-400 px-2 py-1 bg-emerald-500/20 rounded-lg">
                        Current
                      </span>
                    ) : (
                      <Button
                        variant="outline"
                        size="sm"
                        className="border-red-400/30 text-red-300 hover:bg-red-500/20"
                        onClick={() => handleRevokeSession(session.id)}
                      >
                        Revoke
                      </Button>
                    )}
                  </div>
                ))}
              </div>

              {activeSessions.filter(s => !s.is_current).length > 0 && (
                <div className="mt-4 pt-4 border-t border-emerald-400/20">
                  <Button
                    variant="outline"
                    className="w-full border-red-400/30 text-red-300 hover:bg-red-500/20"
                    onClick={handleRevokeAllSessions}
                  >
                    Sign Out All Other Sessions
                  </Button>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  )
}