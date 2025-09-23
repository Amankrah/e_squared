"use client"

import Link from "next/link"
import { useState } from "react"
import { Eye, EyeOff, ArrowRight, Shield, TrendingUp, Lock } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { useAuth } from "@/contexts/auth-context"

export default function Login() {
  const [showPassword, setShowPassword] = useState(false)
  const [formData, setFormData] = useState({
    email: "",
    password: "",
    rememberMe: false
  })
  const [forgotPasswordEmail, setForgotPasswordEmail] = useState("")
  const [forgotPasswordSent, setForgotPasswordSent] = useState(false)

  const { login, isLoading, error, clearError } = useAuth()

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: type === "checkbox" ? checked : value
    }))
    // Clear error when user starts typing
    if (error) {
      clearError()
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await login({
        email: formData.email,
        password: formData.password,
      })
    } catch (error) {
      // Error is handled by the auth context
    }
  }

  const handleForgotPassword = (e: React.FormEvent) => {
    e.preventDefault()
    // TODO: Implement forgot password logic
    console.log("Forgot password for:", forgotPasswordEmail)
    setForgotPasswordSent(true)
    setTimeout(() => {
      setForgotPasswordSent(false)
      setForgotPasswordEmail("")
    }, 3000)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden relative">
      {/* Animated Background Elements */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/20 to-purple-400/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
        <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-indigo-400/20 to-blue-400/20 rounded-full blur-3xl animate-pulse delay-500"></div>
      </div>

      {/* Grid Pattern Overlay */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

      <div className="relative w-full max-w-md mx-auto p-6">
        {/* Logo and Back Link */}
        <div className="text-center mb-8">
          <Link href="/" className="inline-flex items-center space-x-2 text-white hover:text-purple-300 transition-colors mb-6">
            <ArrowRight className="w-5 h-5 rotate-180" />
            <span className="text-lg font-medium">Back to Home</span>
          </Link>

          <div className="flex items-center justify-center space-x-3 mb-2">
            <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-2xl backdrop-blur-sm border border-purple-400/20">
              <TrendingUp className="h-8 w-8 text-purple-200" />
            </div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
              E-Squared
            </h1>
          </div>
          <p className="text-gray-300 text-lg">Welcome back, trader</p>
        </div>

        {/* Glass Morphism Login Card */}
        <div className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8">
          <div className="space-y-6">
            <div className="text-center space-y-2">
              <h2 className="text-2xl font-bold text-white">Sign In</h2>
              <p className="text-gray-300">Access your trading dashboard</p>
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Error Message */}
              {error && (
                <div className="p-4 border border-red-400/30 rounded-xl bg-red-500/10 backdrop-blur-sm">
                  <p className="text-red-300 text-sm">{error}</p>
                </div>
              )}

              {/* Email Field */}
              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                <Input
                  id="email"
                  name="email"
                  type="email"
                  placeholder="Enter your email"
                  value={formData.email}
                  onChange={handleInputChange}
                  required
                  className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300"
                />
              </div>

              {/* Password Field */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label htmlFor="password" className="text-gray-200 font-medium">Password</Label>

                  {/* Forgot Password Dialog */}
                  <Dialog>
                    <DialogTrigger asChild>
                      <button type="button" className="text-sm text-purple-300 hover:text-purple-200 transition-colors">
                        Forgot password?
                      </button>
                    </DialogTrigger>
                    <DialogContent className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                      <DialogHeader>
                        <DialogTitle className="text-white flex items-center space-x-2">
                          <Lock className="w-5 h-5 text-purple-300" />
                          <span>Reset Password</span>
                        </DialogTitle>
                        <DialogDescription className="text-gray-300">
                          Enter your email address and we'll send you a link to reset your password.
                        </DialogDescription>
                      </DialogHeader>

                      <form onSubmit={handleForgotPassword} className="space-y-4">
                        <div className="space-y-2">
                          <Label htmlFor="forgot-email" className="text-gray-200 font-medium">Email Address</Label>
                          <Input
                            id="forgot-email"
                            type="email"
                            placeholder="Enter your email"
                            value={forgotPasswordEmail}
                            onChange={(e) => setForgotPasswordEmail(e.target.value)}
                            required
                            className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300"
                          />
                        </div>

                        {forgotPasswordSent ? (
                          <div className="p-4 border border-emerald-400/30 rounded-xl bg-emerald-500/10 backdrop-blur-sm">
                            <p className="text-emerald-300 text-sm">
                              ✓ Reset link sent! Check your email inbox.
                            </p>
                          </div>
                        ) : (
                          <Button
                            type="submit"
                            className="w-full h-12 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-semibold transition-all duration-300 backdrop-blur-sm"
                          >
                            Send Reset Link
                          </Button>
                        )}
                      </form>
                    </DialogContent>
                  </Dialog>
                </div>

                <div className="relative">
                  <Input
                    id="password"
                    name="password"
                    type={showPassword ? "text" : "password"}
                    placeholder="Enter your password"
                    value={formData.password}
                    onChange={handleInputChange}
                    required
                    className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300 pr-12"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors"
                  >
                    {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                  </button>
                </div>
              </div>

              {/* Remember Me Checkbox */}
              <div className="flex items-center space-x-3">
                <input
                  id="rememberMe"
                  name="rememberMe"
                  type="checkbox"
                  checked={formData.rememberMe}
                  onChange={handleInputChange}
                  className="h-4 w-4 rounded border-2 border-white/20 bg-white/10 text-purple-600 focus:ring-purple-500 focus:ring-offset-0"
                />
                <Label htmlFor="rememberMe" className="text-sm text-gray-300">
                  Remember me for 30 days
                </Label>
              </div>

              {/* Submit Button */}
              <Button
                type="submit"
                disabled={isLoading}
                className="w-full h-12 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-semibold text-lg transition-all duration-300 backdrop-blur-sm hover:translate-y-0.5 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin mr-2"></div>
                    Signing In...
                  </>
                ) : (
                  <>
                    Sign In
                    <ArrowRight className="w-5 h-5 ml-2" />
                  </>
                )}
              </Button>
            </form>

            {/* Divider */}
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/20"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-4 bg-gradient-to-r from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] text-gray-300">
                  Don't have an account?
                </span>
              </div>
            </div>

            {/* Signup Link */}
            <div className="text-center">
              <Link
                href="/signup"
                className="inline-flex items-center space-x-2 text-purple-300 hover:text-purple-200 font-medium transition-colors"
              >
                <span>Create your account</span>
                <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          </div>
        </div>

        {/* Trust Indicators */}
        <div className="flex justify-center items-center space-x-6 mt-8 text-sm">
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-emerald-400" />
            <span className="text-gray-300">Secure Login</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-2 h-2 bg-emerald-400 rounded-full"></div>
            <span className="text-gray-300">2FA Protected</span>
          </div>
        </div>
      </div>
    </div>
  )
}