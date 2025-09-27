"use client"

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { useRouter } from 'next/navigation'
import { apiClient, User, UserProfile, LoginRequest, SignupRequest, ApiError } from '@/lib/api'

interface AuthContextType {
  user: User | null
  profile: UserProfile | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (credentials: LoginRequest) => Promise<void>
  signup: (credentials: SignupRequest) => Promise<void>
  logout: () => void
  refreshUser: (silent?: boolean) => Promise<boolean>
  refreshProfile: (silent?: boolean) => Promise<boolean>
  error: string | null
  clearError: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

interface AuthProviderProps {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null)
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isNavigating, setIsNavigating] = useState(false)
  const router = useRouter()

  const clearError = () => setError(null)

  const refreshUser = async (silent = false) => {
    try {
      const userData = await apiClient.getCurrentUser()
      setUser(userData)
      return true
    } catch (error) {
      // Only log errors if not in silent mode (initialization)
      if (!silent) {
        console.error('Failed to fetch user:', error)
      }
      
      // Clear user state for any authentication error (401, 500, etc.)
      setUser(null)
      setProfile(null)

      // Clear the auth token from API client
      apiClient.setCsrfToken(null)

      if (error instanceof ApiError && (error.status === 401 || error.status === 500)) {
        // Only redirect to login if not in silent mode, not currently navigating,
        // and not already on the login page
        if (!silent && !isNavigating && typeof window !== 'undefined') {
          const currentPath = window.location.pathname
          // Don't redirect if already on auth pages
          if (!currentPath.includes('/login') && !currentPath.includes('/signup') && !currentPath.includes('/')) {
            try {
              router.push('/login')
            } catch (routerError) {
              console.error('Router navigation failed:', routerError)
              // Fallback to window.location if router fails
              window.location.href = '/login'
            }
          }
        }
      }
      return false
    }
  }

  const refreshProfile = async (silent = false) => {
    try {
      const profileData = await apiClient.getProfile()
      setProfile(profileData)
      return true
    } catch (error) {
      // Handle authentication errors
      if (error instanceof ApiError && error.status === 401) {
        // Token is invalid/expired, clear state
        setUser(null)
        setProfile(null)

        // Clear the auth token from API client
        apiClient.setCsrfToken(null)

        // Only redirect to login if not in silent mode, not currently navigating,
        // and not already on the login page
        if (!silent && !isNavigating && typeof window !== 'undefined') {
          const currentPath = window.location.pathname
          // Don't redirect if already on auth pages
          if (!currentPath.includes('/login') && !currentPath.includes('/signup') && !currentPath.includes('/')) {
            try {
              router.push('/login')
            } catch (routerError) {
              console.error('Router navigation failed:', routerError)
              // Fallback to window.location if router fails
              window.location.href = '/login'
            }
          }
        }
        return false
      }
      // Profile doesn't exist yet - this is normal for new users
      if (error instanceof ApiError && error.status === 404) {
        if (!silent) {
          console.log('User profile not found - user needs to create profile')
        }
        setProfile(null)
        return true
      }
      // For other errors, only log if not in silent mode
      if (!silent) {
        console.error('Failed to fetch profile:', error)
      }
      return false
    }
  }

  const login = async (credentials: LoginRequest) => {
    try {
      setIsLoading(true)
      setError(null)
      setIsNavigating(true) // Prevent auth redirects during login
      
      const response = await apiClient.login(credentials)
      setUser(response.user)

      // Try to load profile
      await refreshProfile()

      router.push('/dashboard')
    } catch (error) {
      if (error instanceof ApiError) {
        setError(error.message)
      } else {
        setError('Login failed. Please try again.')
      }
      throw error
    } finally {
      setIsLoading(false)
      setIsNavigating(false) // Re-enable auth redirects
    }
  }

  const signup = async (credentials: SignupRequest) => {
    try {
      setIsLoading(true)
      setError(null)
      setIsNavigating(true) // Prevent auth redirects during signup
      
      const response = await apiClient.signup(credentials)
      setUser(response.user)

      // Profile won't exist yet for new users
      setProfile(null)

      router.push('/dashboard')
    } catch (error) {
      if (error instanceof ApiError) {
        setError(error.message)
      } else {
        setError('Signup failed. Please try again.')
      }
      throw error
    } finally {
      setIsLoading(false)
      setIsNavigating(false) // Re-enable auth redirects
    }
  }

  const logout = async () => {
    try {
      setIsNavigating(true) // Prevent auth redirects during logout
      // Clear state first to immediately update UI
      setUser(null)
      setProfile(null)
      setError(null)
      // Clear CSRF token
      apiClient.setCsrfToken(null)
      // Call logout endpoint to clear server session
      await apiClient.logout()
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      // Ensure state is cleared even if logout fails
      setUser(null)
      setProfile(null)
      setError(null)
      apiClient.setCsrfToken(null)
      setIsNavigating(false) // Re-enable auth redirects
      router.push('/')
    }
  }

  // Route change detection for navigation state
  useEffect(() => {
    let timeoutId: NodeJS.Timeout

    const handleRouteChangeStart = () => {
      // Use setTimeout to avoid scheduling updates during render
      timeoutId = setTimeout(() => setIsNavigating(true), 0)
    }
    
    const handleRouteChangeComplete = () => {
      // Clear any pending start timeout and set navigation to false
      if (timeoutId) clearTimeout(timeoutId)
      setTimeout(() => setIsNavigating(false), 100)
    }

    // Listen to route changes if available (Next.js specific)
    if (typeof window !== 'undefined' && window.history) {
      const originalPushState = window.history.pushState
      const originalReplaceState = window.history.replaceState

      window.history.pushState = function(...args) {
        handleRouteChangeStart()
        const result = originalPushState.apply(window.history, args)
        handleRouteChangeComplete()
        return result
      }

      window.history.replaceState = function(...args) {
        handleRouteChangeStart()
        const result = originalReplaceState.apply(window.history, args)
        handleRouteChangeComplete()
        return result
      }

      window.addEventListener('popstate', handleRouteChangeComplete)

      return () => {
        if (timeoutId) clearTimeout(timeoutId)
        window.history.pushState = originalPushState
        window.history.replaceState = originalReplaceState
        window.removeEventListener('popstate', handleRouteChangeComplete)
      }
    }
  }, [])

  useEffect(() => {
    const initializeAuth = async () => {
      console.log('🔄 Initializing authentication...')
      try {
        // First try to get CSRF token silently
        await apiClient.getCsrfTokenFromServer()
        console.log('✅ CSRF token obtained')
        // Then try to get current user silently - this will work if we have a valid cookie
        const userSuccess = await refreshUser(true) // silent mode
        console.log('🔍 User auth check:', userSuccess ? 'authenticated' : 'not authenticated')
        // Only fetch profile if user authentication succeeded
        if (userSuccess) {
          await refreshProfile(true) // silent mode
          console.log('👤 Profile loaded')
        }
      } catch (error) {
        // Handle CSRF token errors gracefully - user is not authenticated (expected on public pages)
        console.log('❌ Auth initialization error:', error)
        // Clear any stale state
        setUser(null)
        setProfile(null)
      } finally {
        console.log('✅ Auth initialization complete')
        setIsLoading(false)
      }
    }

    initializeAuth()
  }, [])

  const value: AuthContextType = {
    user,
    profile,
    isLoading,
    isAuthenticated: !!user,
    login,
    signup,
    logout,
    refreshUser,
    refreshProfile,
    error,
    clearError,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}