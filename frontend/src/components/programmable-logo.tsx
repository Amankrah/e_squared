"use client"

import { cn } from "@/lib/utils"

interface ProgrammableLogoProps {
  size?: "sm" | "md" | "lg"
  showText?: boolean
  className?: string
}

export function ProgrammableLogo({
  size = "md",
  showText = true,
  className
}: ProgrammableLogoProps) {
  const sizes = {
    sm: { container: "w-10 h-10", svg: "w-8 h-8", text: "text-sm" },
    md: { container: "w-14 h-14", svg: "w-12 h-12", text: "text-xl" },
    lg: { container: "w-20 h-20", svg: "w-16 h-16", text: "text-2xl" }
  }

  return (
    <div className={cn("flex items-center space-x-3 group", className)}>
      {/* Animated Logo Container */}
      <div className={cn(
        "relative flex items-center justify-center border-2 rounded-2xl transition-all duration-500 group-hover:scale-110 group-hover:rotate-3",
        "border-purple-500/30 bg-gradient-to-br from-purple-500/20 via-indigo-500/10 to-purple-500/5",
        "backdrop-blur-xl group-hover:border-purple-400/50 group-hover:shadow-lg group-hover:shadow-purple-500/25",
        "animate-in zoom-in-0 duration-700",
        sizes[size].container
      )}>
        {/* Background Glow Effect */}
        <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 to-indigo-500/10 rounded-2xl blur-sm opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>

        {/* Main SVG Logo */}
        <svg
          className={cn("relative z-10", sizes[size].svg)}
          viewBox="0 0 100 100"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          {/* Gradient Definitions */}
          <defs>
            <linearGradient id="eGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="currentColor" className="text-white" />
              <stop offset="50%" stopColor="currentColor" className="text-purple-200" />
              <stop offset="100%" stopColor="currentColor" className="text-purple-300" />
            </linearGradient>
            <linearGradient id="superscript2Gradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="currentColor" className="text-purple-400" />
              <stop offset="100%" stopColor="currentColor" className="text-pink-400" />
            </linearGradient>
            <linearGradient id="orbitGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="currentColor" className="text-emerald-400" />
              <stop offset="100%" stopColor="currentColor" className="text-teal-400" />
            </linearGradient>
            <linearGradient id="innerGlow" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="currentColor" className="text-purple-500/30" />
              <stop offset="100%" stopColor="currentColor" className="text-pink-500/30" />
            </linearGradient>
          </defs>

          {/* Creative "E" Letter with Modern Design */}
          <g className="animate-pulse-slow">
            {/* Main vertical line with slight curve */}
            <path
              d="M20 25 Q18 22 20 20 L20 80 Q18 82 20 85"
              stroke="url(#eGradient)"
              strokeWidth="4.5"
              strokeLinecap="round"
              fill="none"
              className="drop-shadow-lg"
            />

            {/* Top horizontal line with dynamic end */}
            <path
              d="M20 25 L45 25 Q50 25 52 22 Q50 20 45 20 L20 20"
              stroke="url(#eGradient)"
              strokeWidth="4"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="drop-shadow-lg"
            />

            {/* Middle horizontal line (shorter and stylized) */}
            <path
              d="M20 50 L38 50 Q42 50 44 47 Q42 45 38 45 L20 45"
              stroke="url(#eGradient)"
              strokeWidth="3.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="drop-shadow-lg"
            />

            {/* Bottom horizontal line with dynamic end */}
            <path
              d="M20 80 L48 80 Q53 80 55 77 Q53 75 48 75 L20 75"
              stroke="url(#eGradient)"
              strokeWidth="4"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="drop-shadow-lg"
            />

            {/* Inner glow effect for the E */}
            <path
              d="M20 25 L20 80 M20 25 L45 25 M20 50 L38 50 M20 80 L48 80"
              stroke="url(#innerGlow)"
              strokeWidth="8"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="opacity-30 blur-sm"
            />
          </g>

          {/* Superscript "2" - Clean and Mathematical */}
          <g transform="translate(55, 20)" className="animate-pulse-slow" style={{ animationDelay: "0.3s" }}>
            {/* Mathematical "2" with curves */}
            <path
              d="M2 8 Q2 4 6 4 Q10 4 10 8 Q10 10 8 12 L2 18 L10 18"
              stroke="url(#superscript2Gradient)"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="drop-shadow-md"
            />

            {/* Subtle glow for the 2 */}
            <path
              d="M2 8 Q2 4 6 4 Q10 4 10 8 Q10 10 8 12 L2 18 L10 18"
              stroke="url(#superscript2Gradient)"
              strokeWidth="5"
              strokeLinecap="round"
              strokeLinejoin="round"
              fill="none"
              className="opacity-20 blur-sm"
            />
          </g>

          {/* Orbiting Particles around the 2 */}
          <g className="animate-spin-slow origin-center" style={{ transformOrigin: "60px 28px" }}>
            <circle
              cx="68"
              cy="25"
              r="2.5"
              fill="url(#orbitGradient)"
              className="drop-shadow-sm"
            >
              <animate
                attributeName="r"
                values="1.5;3;1.5"
                dur="2.5s"
                repeatCount="indefinite"
              />
              <animate
                attributeName="opacity"
                values="0.5;1;0.5"
                dur="2.5s"
                repeatCount="indefinite"
              />
            </circle>
          </g>

          {/* Second orbiting particle (counter-rotating) */}
          <g className="animate-spin-slow-reverse origin-center" style={{ transformOrigin: "60px 28px" }}>
            <circle
              cx="52"
              cy="31"
              r="1.5"
              fill="url(#orbitGradient)"
              className="drop-shadow-sm opacity-70"
            >
              <animate
                attributeName="r"
                values="1;2.5;1"
                dur="3s"
                repeatCount="indefinite"
              />
            </circle>
          </g>

          {/* Energy Flow Lines */}
          <g className="opacity-0 group-hover:opacity-70 transition-opacity duration-700">
            {/* From E to 2 */}
            <path
              d="M45 35 Q52 30 58 28"
              stroke="currentColor"
              strokeWidth="1.5"
              fill="none"
              className="text-purple-400/50"
              strokeDasharray="4,6"
            >
              <animate
                attributeName="stroke-dashoffset"
                values="0;10"
                dur="1.5s"
                repeatCount="indefinite"
              />
            </path>

            {/* Mathematical connection */}
            <path
              d="M48 60 Q55 45 62 35"
              stroke="currentColor"
              strokeWidth="1"
              fill="none"
              className="text-emerald-400/40"
              strokeDasharray="2,4"
            >
              <animate
                attributeName="stroke-dashoffset"
                values="0;6"
                dur="2s"
                repeatCount="indefinite"
              />
            </path>
          </g>

          {/* Subtle orbit path indicator */}
          <circle
            cx="60"
            cy="28"
            r="12"
            stroke="currentColor"
            strokeWidth="0.3"
            fill="none"
            className="text-purple-400/15 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
            strokeDasharray="3,6"
          />
        </svg>

        {/* Floating Accent Indicator */}
        <div className="absolute -top-1 -right-1 w-4 h-4 bg-gradient-to-br from-emerald-500/80 to-teal-500/80 rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all duration-500 group-hover:scale-110">
          <div className="w-1.5 h-1.5 bg-white rounded-full animate-pulse"></div>
        </div>

        {/* Animated Border Effect */}
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-purple-500/20 via-pink-500/20 to-purple-500/20 opacity-0 group-hover:opacity-100 transition-opacity duration-500 blur-sm"></div>
      </div>

      {/* Company Name */}
      {showText && (
        <div className="hidden sm:flex flex-col">
          <span className={cn(
            "font-black bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent leading-tight",
            sizes[size].text
          )}>
            E-Squared
          </span>
          <span className={cn(
            "font-medium text-purple-200/80 -mt-1",
            size === "sm" ? "text-xs" : size === "md" ? "text-sm" : "text-base"
          )}>
            Trading
          </span>
        </div>
      )}
    </div>
  )
}