import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount)
}

export function formatNumber(amount: number, decimals: number = 4): string {
  if (amount === 0) return '0'

  if (Math.abs(amount) >= 1000000) {
    return (amount / 1000000).toFixed(2) + 'M'
  }

  if (Math.abs(amount) >= 1000) {
    return (amount / 1000).toFixed(2) + 'K'
  }

  return amount.toFixed(decimals)
}
