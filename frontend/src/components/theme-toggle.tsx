"use client"

import * as React from "react"
import { Moon, Sun, Monitor } from "lucide-react"
import { useTheme } from "next-themes"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

export function ThemeToggle() {
  const { setTheme, theme } = useTheme()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-10 w-10 rounded-xl border-0 bg-transparent hover:bg-white/10 text-gray-200 hover:text-white transition-all duration-300 backdrop-blur-sm focus:ring-2 focus:ring-purple-400/50"
        >
          <Sun className="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all duration-300 dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all duration-300 dark:rotate-0 dark:scale-100" />
          <span className="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="border-2 border-[rgba(147,51,234,0.3)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-2xl min-w-[160px]"
      >
        <DropdownMenuItem
          onClick={() => setTheme("light")}
          className="flex items-center space-x-3 px-4 py-3 text-gray-200 hover:text-white hover:bg-white/10 focus:bg-white/10 rounded-lg m-1 transition-all duration-200 cursor-pointer"
        >
          <Sun className="h-4 w-4 text-amber-400" />
          <span className="font-medium">Light</span>
          {theme === "light" && (
            <div className="ml-auto w-2 h-2 bg-purple-400 rounded-full"></div>
          )}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => setTheme("dark")}
          className="flex items-center space-x-3 px-4 py-3 text-gray-200 hover:text-white hover:bg-white/10 focus:bg-white/10 rounded-lg m-1 transition-all duration-200 cursor-pointer"
        >
          <Moon className="h-4 w-4 text-blue-400" />
          <span className="font-medium">Dark</span>
          {theme === "dark" && (
            <div className="ml-auto w-2 h-2 bg-purple-400 rounded-full"></div>
          )}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => setTheme("system")}
          className="flex items-center space-x-3 px-4 py-3 text-gray-200 hover:text-white hover:bg-white/10 focus:bg-white/10 rounded-lg m-1 transition-all duration-200 cursor-pointer"
        >
          <Monitor className="h-4 w-4 text-purple-400" />
          <span className="font-medium">System</span>
          {theme === "system" && (
            <div className="ml-auto w-2 h-2 bg-purple-400 rounded-full"></div>
          )}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}