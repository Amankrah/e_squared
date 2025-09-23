"use client"

import { useState } from "react"
import { DashboardHeader } from "./dashboard-header"
import { DashboardSidebar } from "./dashboard-sidebar"
import { AuthGuard } from "@/components/auth/auth-guard"

interface DashboardLayoutProps {
  children: React.ReactNode
}

export function DashboardLayout({ children }: DashboardLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)

  const toggleSidebar = () => setSidebarOpen(!sidebarOpen)
  const closeSidebar = () => setSidebarOpen(false)

  return (
    <AuthGuard>
      <div className="h-screen flex flex-col overflow-hidden bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63]">
        {/* Sticky Header */}
        <DashboardHeader onMenuToggle={toggleSidebar} />

        {/* Main Layout Container */}
        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <DashboardSidebar isOpen={sidebarOpen} onClose={closeSidebar} />

          {/* Main Content Area */}
          <main className="flex-1 overflow-y-auto overflow-x-hidden">
            {/* Content Container with proper spacing */}
            <div className="p-6 lg:p-8 min-h-full">
              {/* Background Effects Container */}
              <div className="relative min-h-full">
                {/* Animated Background Elements */}
                <div className="absolute inset-0 -z-10 overflow-hidden">
                  <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/10 to-pink-400/10 rounded-full blur-3xl animate-pulse"></div>
                  <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/10 to-purple-400/10 rounded-full blur-3xl animate-pulse delay-1000"></div>
                  <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-indigo-400/10 to-blue-400/10 rounded-full blur-3xl animate-pulse delay-500"></div>
                </div>

                {/* Grid Pattern Overlay */}
                <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50 -z-10"></div>

                {/* Page Content */}
                <div className="relative z-10">
                  {children}
                </div>
              </div>
            </div>
          </main>
        </div>

        {/* Mobile Overlay */}
        {sidebarOpen && (
          <div
            className="fixed inset-0 z-40 bg-black bg-opacity-50 lg:hidden"
            onClick={closeSidebar}
          />
        )}
      </div>
    </AuthGuard>
  )
}