const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1'

export interface User {
  id: string
  email: string
  is_active: boolean
  is_verified: boolean
  totp_enabled: boolean
  created_at: string
}

export interface UserProfile {
  id: string
  user_id: string
  name: string
  email: string
  phone?: string
  location?: string
  bio?: string
  join_date: string
  avatar_url?: string
  is_verified: boolean
  created_at: string
  updated_at: string
}

export interface AuthResponse {
  user: User
  csrf_token: string
}

export interface LoginRequest {
  email: string
  password: string
}

export interface SignupRequest {
  email: string
  password: string
}

export interface ChangePasswordRequest {
  current_password: string
  new_password: string
}

export interface CreateProfileRequest {
  name: string
  email: string
  phone?: string
  location?: string
  bio?: string
  avatar_url?: string
}

export interface UpdateProfileRequest {
  name?: string
  email?: string
  phone?: string
  location?: string
  bio?: string
  avatar_url?: string
}

export interface Setup2FAResponse {
  secret: string
  qr_code: string
  manual_entry_key: string
}

export interface Setup2FARequest {
  code: string
}

export interface Verify2FARequest {
  code: string
}

export interface Disable2FARequest {
  password: string
}

export interface TwoFactorStatus {
  enabled: boolean
  has_secret: boolean
}

export interface UserSession {
  id: string
  device_info: string
  ip_address: string
  location?: string
  user_agent: string
  is_current: boolean
  last_activity: string
  created_at: string
  platform: string
  browser: string
}

export interface SessionsResponse {
  sessions: UserSession[]
}

export interface ExchangeConnection {
  id: string
  user_id: string
  exchange_name: string
  display_name: string
  is_active: boolean
  last_sync?: string
  connection_status: 'pending' | 'connected' | 'error'
  last_error?: string
  created_at: string
  updated_at: string
}

export interface CreateExchangeConnectionRequest {
  exchange_name: string
  display_name: string
  api_key: string
  api_secret: string
  password: string
}

export interface UpdateExchangeConnectionRequest {
  display_name?: string
  api_key?: string
  api_secret?: string
  password: string
}

// Removed WalletBalance interface - we only use live balance data from exchanges

// New exchange account types based on modular connector
export interface AssetBalance {
  asset: string
  free: string
  locked: string
  total: string
  usd_value?: string
  btc_value?: string
  wallet_type: 'spot' | 'margin' | 'futures' | 'futures_coin' | 'savings' | 'earn'
}

export interface SpotAccount {
  balances: AssetBalance[]
  total_usd_value?: string
  total_btc_value?: string
  maker_commission?: string
  taker_commission?: string
  can_trade: boolean
  can_withdraw: boolean
  can_deposit: boolean
  last_update_time: string
}

export interface MarginAccount {
  balances: AssetBalance[]
  total_asset_value: string
  total_liability_value: string
  total_net_value: string
  margin_level?: string
  margin_ratio?: string
  is_margin_enabled: boolean
  can_trade: boolean
  can_borrow: boolean
  last_update_time: string
}

export interface FuturesPosition {
  symbol: string
  position_side: 'long' | 'short' | 'both'
  position_amount: string
  entry_price: string
  mark_price: string
  unrealized_pnl: string
  realized_pnl: string
  margin_type: 'isolated' | 'cross'
  isolated_margin?: string
  leverage: number
  liquidation_price?: string
  margin_ratio?: string
  maintenance_margin: string
  initial_margin: string
  position_initial_margin: string
  open_order_initial_margin: string
  adl_quantile?: number
}

export interface FuturesAccount {
  account_type: 'usdm' | 'coinm'
  balances: AssetBalance[]
  positions: FuturesPosition[]
  total_wallet_balance: string
  total_unrealized_pnl: string
  total_margin_balance: string
  available_balance: string
  max_withdraw_amount: string
  total_initial_margin: string
  total_maintenance_margin: string
  margin_ratio?: string
  can_trade: boolean
  can_deposit: boolean
  can_withdraw: boolean
  last_update_time: string
}

export interface AccountBalances {
  spot?: SpotAccount
  margin?: MarginAccount
  futures_usdm?: FuturesAccount
  futures_coinm?: FuturesAccount
  total_usd_value: string
  total_btc_value: string
}

export interface AccountResponse {
  connection_id: string
  exchange_name: string
  display_name: string
  accounts: AccountBalances
  last_update: string
}

export interface SpotAccountResponse {
  connection_id: string
  exchange_name: string
  account: SpotAccount
}

export interface MarginAccountResponse {
  connection_id: string
  exchange_name: string
  account: MarginAccount
}

export interface FuturesAccountResponse {
  connection_id: string
  exchange_name: string
  account: FuturesAccount
}

export interface ExchangeConnectionsResponse {
  connections: ExchangeConnection[]
  message?: string
}

export interface LiveBalancesResponse {
  balances: {
    exchange_connection_id: string
    exchange_name: string
    display_name: string
    total_usd_value: string
    total_btc_value: string
    accounts: AccountBalances
    last_updated: string
    is_live: boolean
  }[]
  total_usd_value: string
  is_live: boolean
  last_updated: string
}

// DCA Strategy Types - Backend Compatible
export interface DCAFrequency {
  Hourly?: number
  Daily?: number
  Weekly?: number
  Monthly?: number
  Custom?: number
}

export interface DCARSIConfig {
  period: number
  oversold_threshold: number
  overbought_threshold: number
  oversold_multiplier: number
  overbought_multiplier: number
  normal_multiplier: number
}

export interface VolatilityConfig {
  period: number
  low_threshold: number
  high_threshold: number
  low_volatility_multiplier: number
  high_volatility_multiplier: number
  normal_multiplier: number
}

export interface SentimentConfig {
  fear_greed_threshold?: number
  social_sentiment_threshold?: number
  news_sentiment_threshold?: number
  bearish_multiplier: number
  bullish_multiplier: number
}

export interface DynamicFactors {
  rsi_weight: number
  volatility_weight: number
  sentiment_weight: number
  trend_weight: number
  max_multiplier: number
  min_multiplier: number
}

export interface DipBuyingLevel {
  price_drop_percentage: number
  amount_multiplier: number
  max_triggers?: number
}

export interface DCAFilters {
  allowed_hours?: number[]
  allowed_weekdays?: number[]
  min_interval_minutes?: number
  max_executions_per_day?: number
  min_volume_threshold?: number
  max_spread_percentage?: number
  max_price_deviation_percentage?: number
}

export interface DCAConfig {
  base_amount: number
  frequency: DCAFrequency
  strategy_type: 'Simple' | 'RSIBased' | 'VolatilityBased' | 'Dynamic' | 'DipBuying' | 'SentimentBased'
  rsi_config?: DCARSIConfig
  volatility_config?: VolatilityConfig
  sentiment_config?: SentimentConfig
  dynamic_factors?: DynamicFactors
  dip_levels?: DipBuyingLevel[]
  reference_price?: number
  reference_period_days?: number
  max_single_amount?: number
  min_single_amount?: number
  max_position_size?: number
  pause_on_high_volatility: boolean
  volatility_pause_threshold?: number
  pause_on_bear_market: boolean
  bear_market_threshold?: number
  filters: DCAFilters
}

export interface DCAStrategy {
  id: string
  user_id: string
  name: string
  asset_symbol: string
  status: string
  config: DCAConfig
  total_invested: string
  total_purchased: string
  average_buy_price?: string
  current_profit_loss?: string
  profit_loss_percentage?: string
  last_execution_at?: string
  next_execution_at?: string
  recent_executions: DCAExecution[]
  created_at: string
  updated_at: string
}

export interface DCAExecution {
  id: string
  strategy_id: string
  execution_type: string
  trigger_reason: string
  amount_usd: string
  amount_asset?: string
  price_at_execution?: string
  fear_greed_index?: number
  market_volatility?: string
  order_status: string
  execution_timestamp: string
  error_message?: string
}

export interface CreateDCAStrategyRequest {
  name: string
  asset_symbol: string
  config: DCAConfig
}

export interface DCAStrategiesResponse {
  strategies: DCAStrategy[]
  total_allocation: string
  total_invested: string
  total_profit_loss: string
  active_strategies: number
}

// Base Strategy Types
export interface BaseStrategy {
  id: string
  user_id: string
  name: string
  asset_symbol: string
  status: string
  total_invested: string
  total_purchased: string
  average_buy_price?: string
  current_profit_loss?: string
  profit_loss_percentage?: string
  last_execution_at?: string
  next_execution_at?: string
  created_at: string
  updated_at: string
}

export interface BaseExecution {
  id: string
  strategy_id: string
  execution_type: string
  trigger_reason: string
  amount_usd: string
  amount_asset?: string
  price_at_execution?: string
  order_status: string
  execution_timestamp: string
  error_message?: string
}

export type StrategyType = 'dca' | 'grid_trading' | 'sma_crossover' | 'rsi' | 'macd'

// Grid Trading Strategy Types
export interface GridTradingConfig {
  grid_count: number
  range_percentage: string  // Percentage range from current price (e.g., "10" for ±10%)
  investment_amount: string
  stop_loss_percentage?: string
  take_profit_percentage?: string
  rebalance_threshold?: string
}

export interface GridTradingStrategy extends BaseStrategy {
  config: GridTradingConfig
  recent_executions: GridTradingExecution[]
}

export interface GridTradingExecution extends BaseExecution {
  grid_level?: number
  buy_price?: string
  sell_price?: string
}

export interface CreateGridTradingStrategyRequest {
  name: string
  asset_symbol: string
  config: GridTradingConfig
}

export interface GridTradingStrategiesResponse {
  strategies: GridTradingStrategy[]
  total_allocation: string
  total_invested: string
  total_profit_loss: string
  active_strategies: number
}

// SMA Crossover Strategy Types
export interface SMACrossoverConfig {
  short_period: number
  long_period: number
  investment_amount: string
  stop_loss_percentage?: string
  take_profit_percentage?: string
  confirmation_period?: number
}

export interface SMACrossoverStrategy extends BaseStrategy {
  config: SMACrossoverConfig
  recent_executions: SMACrossoverExecution[]
}

export interface SMACrossoverExecution extends BaseExecution {
  short_sma?: string
  long_sma?: string
  signal_type?: string
}

export interface CreateSMACrossoverStrategyRequest {
  name: string
  asset_symbol: string
  config: SMACrossoverConfig
}

export interface SMACrossoverStrategiesResponse {
  strategies: SMACrossoverStrategy[]
  total_allocation: string
  total_invested: string
  total_profit_loss: string
  active_strategies: number
}

// RSI Strategy Types
export interface RSIConfig {
  rsi_period: number
  oversold_threshold: number
  overbought_threshold: number
  investment_amount: string
  stop_loss_percentage?: string
  take_profit_percentage?: string
}

export interface RSIStrategy extends BaseStrategy {
  config: RSIConfig
  recent_executions: RSIExecution[]
}

export interface RSIExecution extends BaseExecution {
  rsi_value?: number
  signal_strength?: string
}

export interface CreateRSIStrategyRequest {
  name: string
  asset_symbol: string
  config: RSIConfig
}

export interface RSIStrategiesResponse {
  strategies: RSIStrategy[]
  total_allocation: string
  total_invested: string
  total_profit_loss: string
  active_strategies: number
}

// MACD Strategy Types
export interface MACDConfig {
  fast_period: number
  slow_period: number
  signal_period: number
  investment_amount: string
  stop_loss_percentage?: string
  take_profit_percentage?: string
}

export interface MACDStrategy extends BaseStrategy {
  config: MACDConfig
  recent_executions: MACDExecution[]
}

export interface MACDExecution extends BaseExecution {
  macd_value?: string
  signal_value?: string
  histogram?: string
}

export interface CreateMACDStrategyRequest {
  name: string
  asset_symbol: string
  config: MACDConfig
}

export interface MACDStrategiesResponse {
  strategies: MACDStrategy[]
  total_allocation: string
  total_invested: string
  total_profit_loss: string
  active_strategies: number
}

// Unified Strategy Types
export type Strategy = DCAStrategy | GridTradingStrategy | SMACrossoverStrategy | RSIStrategy | MACDStrategy
export type StrategyConfig = DCAConfig | GridTradingConfig | SMACrossoverConfig | RSIConfig | MACDConfig
export type CreateStrategyRequest = CreateDCAStrategyRequest | CreateGridTradingStrategyRequest | CreateSMACrossoverStrategyRequest | CreateRSIStrategyRequest | CreateMACDStrategyRequest
export type StrategiesResponse = DCAStrategiesResponse | GridTradingStrategiesResponse | SMACrossoverStrategiesResponse | RSIStrategiesResponse | MACDStrategiesResponse

// Strategy Info for UI
export interface StrategyInfo {
  type: StrategyType
  name: string
  description: string
  icon: string
  color: string
  riskLevel: 'Low' | 'Medium' | 'High'
  timeHorizon: string
  features: string[]
  minInvestment: number
}

// Backtesting Types
export interface BacktestRequest {
  strategy_type: StrategyType
  asset_symbol: string
  start_date: string
  end_date: string
  initial_capital: number
  config: StrategyConfig
  interval?: string
}

// Backend-compatible backtest request
export interface BackendBacktestRequest {
  symbol: string
  interval: string
  start_date: string
  end_date: string
  initial_balance: number
  strategy_name: string
  strategy_parameters?: any
  stop_loss_percentage?: number
  take_profit_percentage?: number
}

// Database backtest result model (from history endpoint)
export interface BacktestResult {
  id: string
  name: string
  description?: string
  strategy_name: string
  strategy_type?: string  // Sub-strategy type (e.g., "Simple", "RSIBased", "VolatilityBased" for DCA)
  symbol: string
  interval: string
  start_date: string
  end_date: string
  initial_balance: string
  final_balance: string
  total_return: string
  total_return_percentage: string
  max_drawdown: string
  max_drawdown_percentage: string
  sharpe_ratio?: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  win_rate: string
  profit_factor?: string
  largest_win: string
  largest_loss: string
  average_win: string
  average_loss: string
  status: string
  error_message?: string
  execution_time_ms?: number
  created_at: string
  updated_at: string
}

export interface OpenPosition {
  timestamp: string
  price: string
  quantity: string
  total_value: string
  reason: string
}

// Backtest engine result (from run backtest endpoint)
export interface BacktestEngineResult {
  backtest_id: string
  config: any
  trades: BacktestTrade[]
  metrics: BacktestEngineMetrics
  performance_chart: PerformancePoint[]
  execution_time_ms: number
  open_positions?: OpenPosition[]
}

export interface BacktestEngineMetrics {
  total_return: string
  total_return_percentage: string
  annualized_return?: string
  sharpe_ratio?: string
  max_drawdown: string
  volatility: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  win_rate: string
  average_win: string
  average_loss: string
  profit_factor?: string
  final_portfolio_value: string
  benchmark_return?: string
  alpha?: string
  beta?: string
  total_invested: string
}

export interface PerformancePoint {
  timestamp: string
  portfolio_value: string
  drawdown: string
  cumulative_return: string
}

export interface BacktestResultDetail extends BacktestResult {
  strategy_parameters: any
  trades_data: any
  equity_curve: any
  drawdown_curve: any
}

export interface BacktestListResponse {
  results: BacktestResult[]
  pagination: {
    page: number
    limit: number
    total: number
    total_pages: number
  }
}

export interface BacktestListQuery {
  page?: number
  limit?: number
  strategy_name?: string
  symbol?: string
  status?: string
}

export interface DailyReturn {
  date: string
  portfolio_value: string
  daily_return: string
  cumulative_return: string
  drawdown: string
}

export interface BacktestTrade {
  timestamp: string
  trade_type: 'Buy' | 'Sell'
  price: string
  quantity: string
  total_value: string
  portfolio_value: string
  balance_remaining: string
  reason: string
  pnl?: string
  pnl_percentage?: string
  // Legacy fields for compatibility
  entry_date?: string
  exit_date?: string
  entry_price?: string
  exit_price?: string
  side?: 'buy' | 'sell'
  duration_hours?: number
  signal_reason?: string
}

export interface BacktestMetrics {
  total_days: number
  trading_days: number
  winning_days: number
  losing_days: number
  best_day: string
  worst_day: string
  longest_winning_streak: number
  longest_losing_streak: number
  max_consecutive_losses: number
  recovery_factor: string
  calmar_ratio: string
  information_ratio: string
}

export interface BacktestComparison {
  backtest_results: BacktestResult[]
  comparison_metrics: {
    strategy_types: StrategyType[]
    performance_summary: {
      [key in StrategyType]?: {
        total_return: string
        sharpe_ratio: string
        max_drawdown: string
        win_rate: string
      }
    }
  }
}

// Strategy Templates
export interface StrategyTemplate {
  id: string
  name: string
  description: string
  category: string
  risk_level: string
  recommended_allocation: {
    min_usd: string
    max_usd: string
    recommended_usd: string
  }
  time_horizon: string
  parameters: {
    strategy_type: string
    base_tranche_percentage: string
    sentiment_multiplier: boolean
    volatility_adjustment: boolean
    fear_greed_threshold_buy: number
    fear_greed_threshold_sell: number
    dca_interval_hours: number
    target_zones?: string[]
    stop_loss_percentage?: string
    take_profit_percentage?: string
    max_tranche_percentage: string
    min_tranche_percentage: string
  }
  features: string[]
  pros: string[]
  cons: string[]
  historical_performance?: {
    backtest_period: string
    total_return: string
    annualized_return: string
    max_drawdown: string
    win_rate: string
    sharpe_ratio: string
  }
  best_markets: string[]
  complexity: string
}

export interface TemplatesResponse {
  total_count: number
  templates: StrategyTemplate[]
}

export interface UserProfile {
  experience_level: string
  risk_tolerance: string
  investment_amount: number
  time_horizon: string
  primary_goals: string[]
  market_preference: string[]
}


export interface BacktestResults {
  config: any
  executions: any[]
  performance_metrics: {
    total_return: string
    total_return_percentage: string
    annualized_return: string
    max_drawdown: string
    max_drawdown_percentage: string
    sharpe_ratio: string
    win_rate: string
    total_trades: number
    profitable_trades: number
    volatility: string
  }
  comparison_metrics: {
    buy_and_hold_return: string
    buy_and_hold_return_percentage: string
    dca_vs_buy_hold_advantage: string
    regular_dca_return: string
    adaptive_vs_regular_advantage: string
  }
}

export interface TemplateComparison {
  request: any
  results: {
    template_id: string
    template_name: string
    results: BacktestResults
  }[]
  summary: {
    best_return_template: string
    best_sharpe_template: string
    lowest_risk_template: string
    most_consistent_template: string
    performance_ranking: {
      template_id: string
      template_name: string
      rank: number
      score: number
      strengths: string[]
      weaknesses: string[]
    }[]
  }
}

class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message)
    this.name = 'ApiError'
  }
}

class ApiClient {
  private baseUrl: string
  private csrfToken: string | null = null

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl
  }

  setCsrfToken(token: string | null) {
    this.csrfToken = token
  }

  getCsrfToken(): string | null {
    return this.csrfToken
  }

  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    }

    // Add CSRF token for non-GET requests
    if (options.method && options.method !== 'GET' && this.csrfToken) {
      (headers as Record<string, string>)['X-CSRF-Token'] = this.csrfToken
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        credentials: 'include', // This ensures cookies are sent
      })

      if (!response.ok) {
        // Handle authentication errors more specifically
        if (response.status === 401) {
          // Clear any stored tokens/state on unauthorized
          this.setCsrfToken(null)
        }
        
        const errorData = await response.json().catch(() => ({ 
          error: response.status === 401 ? 'Authentication required' : 'Unknown error' 
        }))
        throw new ApiError(response.status, errorData.message || errorData.error || 'Request failed')
      }

      return response.json()
    } catch (error) {
      // Handle network errors (CORS, fetch failures, etc.)
      if (error instanceof ApiError) {
        throw error
      }
      
      // Handle fetch/network errors
      if (error instanceof TypeError && error.message.includes('fetch')) {
        throw new ApiError(0, 'Network error - please check your connection')
      }
      
      // Handle other errors
      throw new ApiError(500, 'An unexpected error occurred')
    }
  }

  // Authentication endpoints
  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    })
    this.setCsrfToken(response.csrf_token)
    return response
  }

  async signup(credentials: SignupRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/signup', {
      method: 'POST',
      body: JSON.stringify(credentials),
    })
    this.setCsrfToken(response.csrf_token)
    return response
  }

  async getCurrentUser(): Promise<User> {
    const response = await this.request<{
      authenticated: boolean
      user: User | null
    }>('/auth/me')
    
    if (!response.authenticated || !response.user) {
      throw new ApiError(401, 'User not authenticated')
    }
    
    return response.user
  }

  async changePassword(data: ChangePasswordRequest): Promise<{ message: string }> {
    return this.request<{ message: string }>('/auth/change-password', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  // Profile endpoints
  async createProfile(data: CreateProfileRequest): Promise<UserProfile> {
    return this.request<UserProfile>('/profile', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getProfile(): Promise<UserProfile> {
    return this.request<UserProfile>('/profile')
  }

  async updateProfile(data: UpdateProfileRequest): Promise<UserProfile> {
    return this.request<UserProfile>('/profile', {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteProfile(): Promise<{ message: string }> {
    return this.request<{ message: string }>('/profile', {
      method: 'DELETE',
    })
  }

  async getPublicProfile(profileId: string): Promise<UserProfile> {
    return this.request<UserProfile>(`/public/profile/${profileId}`)
  }

  async logout(): Promise<void> {
    try {
      await this.request<{ message: string }>('/auth/logout', {
        method: 'POST',
      })
    } catch (error) {
      // Even if the logout endpoint fails (e.g., due to expired token),
      // we should still clear the local state
      console.error('Logout endpoint error:', error)
    } finally {
      // Always clear the CSRF token
      this.setCsrfToken(null)
    }
  }

  async getCsrfTokenFromServer(): Promise<string> {
    try {
      const response = await this.request<{ csrf_token: string }>('/auth/csrf-token')
      this.setCsrfToken(response.csrf_token)
      return response.csrf_token
    } catch (error) {
      // If CSRF token fetch fails, clear any existing token
      this.setCsrfToken(null)
      throw error
    }
  }

  // 2FA endpoints
  async setup2FA(): Promise<Setup2FAResponse> {
    return this.request<Setup2FAResponse>('/2fa/setup', {
      method: 'POST',
    })
  }

  async verifySetup2FA(data: Setup2FARequest): Promise<{ message: string }> {
    return this.request<{ message: string }>('/2fa/verify-setup', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async verify2FA(data: Verify2FARequest): Promise<{ message: string }> {
    return this.request<{ message: string }>('/2fa/verify', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async disable2FA(data: Disable2FARequest): Promise<{ message: string }> {
    return this.request<{ message: string }>('/2fa/disable', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async get2FAStatus(): Promise<TwoFactorStatus> {
    return this.request<TwoFactorStatus>('/2fa/status')
  }

  // Session management endpoints
  async getActiveSessions(): Promise<SessionsResponse> {
    return this.request<SessionsResponse>('/sessions')
  }

  async revokeSession(sessionId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/sessions/${sessionId}`, {
      method: 'DELETE',
    })
  }

  async revokeAllSessions(): Promise<{ message: string }> {
    return this.request<{ message: string }>('/sessions/revoke-all', {
      method: 'POST',
    })
  }

  // Exchange management endpoints
  async createExchangeConnection(data: CreateExchangeConnectionRequest): Promise<ExchangeConnection> {
    return this.request<ExchangeConnection>('/exchanges/connections', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getExchangeConnections(): Promise<ExchangeConnectionsResponse> {
    return this.request<ExchangeConnectionsResponse>('/exchanges/connections')
  }

  async updateExchangeConnection(connectionId: string, data: UpdateExchangeConnectionRequest): Promise<ExchangeConnection> {
    return this.request<ExchangeConnection>(`/exchanges/connections/${connectionId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteExchangeConnection(connectionId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/exchanges/connections/${connectionId}`, {
      method: 'DELETE',
    })
  }

  async syncExchangeConnection(connectionId: string, password: string): Promise<{
    connection_id: string
    exchange_name: string
    display_name: string
    total_usd_value: string
    total_btc_value: string
    accounts: AccountBalances
    last_updated: string
    is_live: boolean
  }> {
    return this.request(`/exchanges/connections/${connectionId}/sync`, {
      method: 'POST',
      body: JSON.stringify({ password }),
    })
  }

  async getLiveWalletBalances(connectionId: string, password: string): Promise<{
    exchange_connection_id: string
    exchange_name: string
    display_name: string
    total_usd_value: string
    total_btc_value: string
    accounts: AccountBalances
    last_updated: string
    is_live: boolean
  }> {
    return this.request(`/exchanges/connections/${connectionId}/live-balances`, {
      method: 'POST',
      body: JSON.stringify({ password })
    })
  }

  async getAllLiveUserBalances(password: string): Promise<LiveBalancesResponse> {
    return this.request<LiveBalancesResponse>('/exchanges/live-balances', {
      method: 'POST',
      body: JSON.stringify({ password })
    })
  }

  // Helper method to get connections with live balances
  async getConnectionsWithLiveBalances(password: string): Promise<{
    connections: ExchangeConnection[];
    live_balances: Record<string, any>;
  }> {
    const [connectionsResponse, liveBalancesResponse] = await Promise.all([
      this.getExchangeConnections(),
      this.getAllLiveUserBalances(password)
    ]);

    // Create a map of connection_id to live balance data
    const liveBalancesMap: Record<string, any> = {};
    if (liveBalancesResponse.balances) {
      for (const balance of liveBalancesResponse.balances) {
        liveBalancesMap[balance.exchange_connection_id] = balance;
      }
    }

    return {
      connections: connectionsResponse.connections,
      live_balances: liveBalancesMap
    };
  }

  // Strategy endpoints - Unified approach
  
  // DCA Strategy endpoints
  async getDCAStrategies(): Promise<DCAStrategiesResponse> {
    return this.request<DCAStrategiesResponse>('/dca/strategies')
  }

  async createDCAStrategy(data: CreateDCAStrategyRequest): Promise<DCAStrategy> {
    // Transform the data to match backend expectations
    const transformedData = {
      name: data.name,
      asset_symbol: data.asset_symbol,
      config_json: JSON.stringify(data.config)
    }

    return this.request<DCAStrategy>('/dca/strategies', {
      method: 'POST',
      body: JSON.stringify(transformedData),
    })
  }

  async getDCAStrategy(strategyId: string): Promise<DCAStrategy> {
    return this.request<DCAStrategy>(`/dca/strategies/${strategyId}`)
  }

  async updateDCAStrategy(strategyId: string, data: Partial<CreateDCAStrategyRequest>): Promise<DCAStrategy> {
    return this.request<DCAStrategy>(`/dca/strategies/${strategyId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteDCAStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/dca/strategies/${strategyId}`, {
      method: 'DELETE',
    })
  }

  // Grid Trading Strategy endpoints
  async getGridTradingStrategies(): Promise<GridTradingStrategiesResponse> {
    return this.request<GridTradingStrategiesResponse>('/grid-trading/strategies')
  }

  async createGridTradingStrategy(data: CreateGridTradingStrategyRequest): Promise<GridTradingStrategy> {
    // Transform frontend config to backend config
    const transformedData = {
      name: data.name,
      asset_symbol: data.asset_symbol,
      config: this.transformGridTradingConfig(data.config)
    }

    return this.request<GridTradingStrategy>('/grid-trading/strategies', {
      method: 'POST',
      body: JSON.stringify(transformedData),
    })
  }

  async getGridTradingStrategy(strategyId: string): Promise<GridTradingStrategy> {
    return this.request<GridTradingStrategy>(`/grid-trading/strategies/${strategyId}`)
  }

  async updateGridTradingStrategy(strategyId: string, data: Partial<CreateGridTradingStrategyRequest>): Promise<GridTradingStrategy> {
    return this.request<GridTradingStrategy>(`/grid-trading/strategies/${strategyId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteGridTradingStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/grid-trading/strategies/${strategyId}`, {
      method: 'DELETE',
    })
  }

  // SMA Crossover Strategy endpoints
  async getSMACrossoverStrategies(): Promise<SMACrossoverStrategiesResponse> {
    return this.request<SMACrossoverStrategiesResponse>('/sma-crossover/strategies')
  }

  async createSMACrossoverStrategy(data: CreateSMACrossoverStrategyRequest): Promise<SMACrossoverStrategy> {
    // Transform frontend config to backend config
    const transformedData = {
      name: data.name,
      asset_symbol: data.asset_symbol,
      config: this.transformSMACrossoverConfig(data.config)
    }

    return this.request<SMACrossoverStrategy>('/sma-crossover/strategies', {
      method: 'POST',
      body: JSON.stringify(transformedData),
    })
  }

  async getSMACrossoverStrategy(strategyId: string): Promise<SMACrossoverStrategy> {
    return this.request<SMACrossoverStrategy>(`/sma-crossover/strategies/${strategyId}`)
  }

  async updateSMACrossoverStrategy(strategyId: string, data: Partial<CreateSMACrossoverStrategyRequest>): Promise<SMACrossoverStrategy> {
    return this.request<SMACrossoverStrategy>(`/sma-crossover/strategies/${strategyId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteSMACrossoverStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/sma-crossover/strategies/${strategyId}`, {
      method: 'DELETE',
    })
  }

  // RSI Strategy endpoints
  async getRSIStrategies(): Promise<RSIStrategiesResponse> {
    return this.request<RSIStrategiesResponse>('/rsi/strategies')
  }

  async createRSIStrategy(data: CreateRSIStrategyRequest): Promise<RSIStrategy> {
    // Transform frontend config to backend config
    const transformedData = {
      name: data.name,
      asset_symbol: data.asset_symbol,
      config: this.transformRSIConfig(data.config)
    }

    return this.request<RSIStrategy>('/rsi/strategies', {
      method: 'POST',
      body: JSON.stringify(transformedData),
    })
  }

  async getRSIStrategy(strategyId: string): Promise<RSIStrategy> {
    return this.request<RSIStrategy>(`/rsi/strategies/${strategyId}`)
  }

  async updateRSIStrategy(strategyId: string, data: Partial<CreateRSIStrategyRequest>): Promise<RSIStrategy> {
    return this.request<RSIStrategy>(`/rsi/strategies/${strategyId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteRSIStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/rsi/strategies/${strategyId}`, {
      method: 'DELETE',
    })
  }

  // MACD Strategy endpoints
  async getMACDStrategies(): Promise<MACDStrategiesResponse> {
    return this.request<MACDStrategiesResponse>('/macd/strategies')
  }

  async createMACDStrategy(data: CreateMACDStrategyRequest): Promise<MACDStrategy> {
    // Transform frontend config to backend config
    const transformedData = {
      name: data.name,
      asset_symbol: data.asset_symbol,
      config: this.transformMACDConfig(data.config)
    }

    return this.request<MACDStrategy>('/macd/strategies', {
      method: 'POST',
      body: JSON.stringify(transformedData),
    })
  }

  async getMACDStrategy(strategyId: string): Promise<MACDStrategy> {
    return this.request<MACDStrategy>(`/macd/strategies/${strategyId}`)
  }

  async updateMACDStrategy(strategyId: string, data: Partial<CreateMACDStrategyRequest>): Promise<MACDStrategy> {
    return this.request<MACDStrategy>(`/macd/strategies/${strategyId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteMACDStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/macd/strategies/${strategyId}`, {
      method: 'DELETE',
    })
  }

  // Strategy Summary (lightweight endpoint)
  async getUserStrategySummary(): Promise<{
    authenticated: boolean
    strategy_types: Array<{
      strategy_type: string
      count: number
      has_active: boolean
    }>
    total_strategies: number
    total_active: number
  }> {
    return this.request('/auth/strategy-summary')
  }

  // Conditional Strategy Loading (only load specific strategy types)
  async getStrategiesByTypes(types: string[]): Promise<{
    dca?: DCAStrategiesResponse
    gridTrading?: GridTradingStrategiesResponse
    smaCrossover?: SMACrossoverStrategiesResponse
    rsi?: RSIStrategiesResponse
    macd?: MACDStrategiesResponse
  }> {
    const promises: Promise<any>[] = []
    const results: any = {}

    if (types.includes('dca')) {
      promises.push(
        this.getDCAStrategies()
          .then(data => { results.dca = data })
          .catch(() => { results.dca = { strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 } })
      )
    }

    if (types.includes('grid_trading')) {
      promises.push(
        this.getGridTradingStrategies()
          .then(data => { results.gridTrading = data })
          .catch(() => { results.gridTrading = { strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 } })
      )
    }

    if (types.includes('sma_crossover')) {
      promises.push(
        this.getSMACrossoverStrategies()
          .then(data => { results.smaCrossover = data })
          .catch(() => { results.smaCrossover = { strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 } })
      )
    }

    if (types.includes('rsi')) {
      promises.push(
        this.getRSIStrategies()
          .then(data => { results.rsi = data })
          .catch(() => { results.rsi = { strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 } })
      )
    }

    if (types.includes('macd')) {
      promises.push(
        this.getMACDStrategies()
          .then(data => { results.macd = data })
          .catch(() => { results.macd = { strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 } })
      )
    }

    await Promise.all(promises)
    return results
  }

  // Unified Strategy Management (keep for backward compatibility but discourage use)
  async getAllStrategies(): Promise<{
    dca: DCAStrategiesResponse
    gridTrading: GridTradingStrategiesResponse
    smaCrossover: SMACrossoverStrategiesResponse
    rsi: RSIStrategiesResponse
    macd: MACDStrategiesResponse
  }> {
    const [dca, gridTrading, smaCrossover, rsi, macd] = await Promise.all([
      this.getDCAStrategies().catch(() => ({ strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 })),
      this.getGridTradingStrategies().catch(() => ({ strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 })),
      this.getSMACrossoverStrategies().catch(() => ({ strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 })),
      this.getRSIStrategies().catch(() => ({ strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 })),
      this.getMACDStrategies().catch(() => ({ strategies: [], total_allocation: '0', total_invested: '0', total_profit_loss: '0', active_strategies: 0 }))
    ])

    return { dca, gridTrading, smaCrossover, rsi, macd }
  }

  // Configuration transformation helpers
  private transformDCAConfig(config: any): DCAConfig {
    // Handle different frequency formats
    let frequency: DCAFrequency
    if (config.frequency) {
      // Already in correct format
      frequency = config.frequency
    } else if (config.dca_interval_hours) {
      // Old format - convert to new format
      if (config.dca_interval_hours === 24) {
        frequency = { Daily: 1 }
      } else if (config.dca_interval_hours === 168) {
        frequency = { Weekly: 1 }
      } else if (config.dca_interval_hours < 24) {
        frequency = { Hourly: config.dca_interval_hours }
      } else {
        frequency = { Custom: config.dca_interval_hours * 60 } // Convert to minutes
      }
    } else {
      frequency = { Daily: 1 } // Default
    }

    // Map strategy type from frontend to backend format
    let strategyType: DCAConfig['strategy_type']
    switch (config.strategy_type) {
      case 'conservative':
      case 'simple':
      case 'Simple':
        strategyType = 'Simple'
        break
      case 'rsi_based':
      case 'aggressive':
      case 'RSIBased':
        strategyType = 'RSIBased'
        break
      case 'volatility_based':
      case 'VolatilityBased':
        strategyType = 'VolatilityBased'
        break
      case 'dynamic':
      case 'moderate':
      case 'Dynamic':
        strategyType = 'Dynamic'
        break
      case 'dip_buying':
      case 'DipBuying':
        strategyType = 'DipBuying'
        break
      case 'sentiment_based':
      case 'SentimentBased':
        strategyType = 'SentimentBased'
        break
      default:
        strategyType = 'Simple'
    }

    // Create backend-compatible DCA config
    const dcaConfig: DCAConfig = {
      base_amount: config.base_amount || config.total_allocation || 1000,
      frequency,
      strategy_type: strategyType,
      pause_on_high_volatility: config.pause_on_high_volatility || false,
      pause_on_bear_market: config.pause_on_bear_market || false,
      filters: {
        allowed_hours: config.allowed_hours,
        allowed_weekdays: config.allowed_weekdays,
        min_interval_minutes: config.min_interval_minutes,
        max_executions_per_day: config.max_executions_per_day,
        min_volume_threshold: config.min_volume_threshold,
        max_spread_percentage: config.max_spread_percentage,
        max_price_deviation_percentage: config.max_price_deviation_percentage
      }
    }

    // Add optional configurations based on strategy type and frontend config
    // Use existing config if already in correct format, otherwise build from old format
    if (config.sentiment_config) {
      dcaConfig.sentiment_config = config.sentiment_config
    } else if (config.sentiment_multiplier || config.fear_greed_threshold_buy || config.fear_greed_threshold_sell) {
      dcaConfig.sentiment_config = {
        fear_greed_threshold: config.fear_greed_threshold_buy,
        bearish_multiplier: config.fear_multiplier || 1.5,
        bullish_multiplier: config.greed_multiplier || 0.7
      }
    }

    if (config.volatility_config) {
      dcaConfig.volatility_config = config.volatility_config
      dcaConfig.volatility_pause_threshold = config.volatility_pause_threshold
    } else if (config.volatility_adjustment || strategyType === 'VolatilityBased' || strategyType === 'Dynamic') {
      dcaConfig.volatility_config = {
        period: config.volatility_period || 20,
        low_threshold: config.volatility_low_threshold || 10,
        high_threshold: config.volatility_high_threshold || 30,
        low_volatility_multiplier: config.low_volatility_multiplier || 0.8,
        high_volatility_multiplier: config.high_volatility_multiplier || 1.5,
        normal_multiplier: 1.0
      }
      dcaConfig.volatility_pause_threshold = config.volatility_pause_threshold
    }

    if (config.rsi_config) {
      dcaConfig.rsi_config = config.rsi_config
    } else if (strategyType === 'RSIBased' || strategyType === 'Dynamic') {
      dcaConfig.rsi_config = {
        period: config.rsi_period || 14,
        oversold_threshold: config.rsi_oversold || 30,
        overbought_threshold: config.rsi_overbought || 70,
        oversold_multiplier: config.rsi_oversold_multiplier || 2.0,
        overbought_multiplier: config.rsi_overbought_multiplier || 0.5,
        normal_multiplier: 1.0
      } as DCARSIConfig
    }

    if (config.dynamic_factors) {
      dcaConfig.dynamic_factors = config.dynamic_factors
    } else if (strategyType === 'Dynamic') {
      dcaConfig.dynamic_factors = {
        rsi_weight: config.rsi_weight || 0.3,
        volatility_weight: config.volatility_weight || 0.3,
        sentiment_weight: config.sentiment_weight || 0.2,
        trend_weight: config.trend_weight || 0.2,
        max_multiplier: config.max_multiplier || 3.0,
        min_multiplier: config.min_multiplier || 0.3
      }
    }

    if (config.dip_levels) {
      dcaConfig.dip_levels = config.dip_levels
      dcaConfig.reference_price = config.reference_price
      dcaConfig.reference_period_days = config.reference_period_days
    } else if (strategyType === 'DipBuying') {
      // Provide default dip levels if not specified
      dcaConfig.dip_levels = [
        { price_drop_percentage: 5, amount_multiplier: 1.5, max_triggers: undefined },
        { price_drop_percentage: 10, amount_multiplier: 2.0, max_triggers: undefined },
        { price_drop_percentage: 20, amount_multiplier: 3.0, max_triggers: undefined }
      ]
      dcaConfig.reference_price = config.reference_price
      dcaConfig.reference_period_days = config.reference_period_days || 30
    }

    // Risk management settings
    if (config.max_single_amount) dcaConfig.max_single_amount = config.max_single_amount
    if (config.min_single_amount) dcaConfig.min_single_amount = config.min_single_amount
    if (config.max_position_size) dcaConfig.max_position_size = config.max_position_size
    if (config.bear_market_threshold) dcaConfig.bear_market_threshold = config.bear_market_threshold

    return dcaConfig
  }

  private transformGridTradingConfig(config: any): any {
    const rangePercentage = parseFloat(config.range_percentage || '10')
    const gridLevels = config.grid_count || 10

    // Calculate spacing as: (total range) / (number of levels - 1)
    // Total range is 2x rangePercentage (upper + lower)
    // For example: 10% range means ±10%, so total range is 20%
    const totalRangePercentage = rangePercentage * 2
    const spacingPercentage = totalRangePercentage / (gridLevels - 1)

    return {
      grid_levels: gridLevels,
      total_investment: parseFloat(config.investment_amount || '1000'),
      spacing: {
        mode: 'Standard',
        fixed_spacing: spacingPercentage,
        dynamic_base_pct: null,
        volatility_factor: null,
        geometric_multiplier: null
      },
      bounds: {
        upper_bound: rangePercentage,
        lower_bound: rangePercentage,
        bounds_type: 'PercentageFromCenter',
        auto_adjust: true,
        use_support_resistance: false
      },
      risk_settings: {
        max_inventory: parseFloat(config.investment_amount || '1000') * 0.5,
        stop_loss_pct: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : null,
        take_profit_pct: config.take_profit_percentage ? parseFloat(config.take_profit_percentage) : null,
        max_drawdown_pct: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : 15.0,
        max_time_in_position: null,
        dynamic_adjustment: true,
        volatility_pause_threshold: null
      },
      min_order_size: 10.0,
      max_order_size: null,
      enable_rebalancing: true,
      rebalancing_interval: null, // Only rebalance when price exits bounds
      take_profit_threshold: config.take_profit_percentage ? parseFloat(config.take_profit_percentage) : null,
      stop_loss_threshold: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : null,
      market_making: {
        enabled: false,
        spread_pct: 0.2,
        inventory_target: 0.0,
        max_inventory_deviation: parseFloat(config.investment_amount || '1000') * 0.3,
        inventory_adjustment: true,
        inventory_skew_factor: 0.1
      }
    }
  }

  private transformRSIConfig(config: any): any {
    return {
      rsi_period: config.rsi_period || 14,
      overbought_level: config.overbought_threshold || 70,
      oversold_level: config.oversold_threshold || 30,
      enable_long: true,
      enable_short: false,
      position_sizing: {
        sizing_method: 'PortfolioPercentage',
        fixed_size: null,
        portfolio_percentage: 5.0,
        risk_per_trade: 2.0,
        max_position_size: parseFloat(config.investment_amount || '300'),
        min_position_size: 10.0
      },
      risk_management: {
        stop_loss_pct: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : 5.0,
        take_profit_pct: config.take_profit_percentage ? parseFloat(config.take_profit_percentage) : 10.0,
        max_drawdown_pct: 15.0,
        max_consecutive_losses: 3,
        cooldown_period: 30,
        rsi_stop_loss: 10.0,
        trailing_stop: {
          enabled: true,
          activation_threshold: 3.0,
          trailing_distance: 2.0,
          rsi_based_trailing: false,
          rsi_trailing_buffer: 5.0
        }
      },
      signal_filters: {
        min_volume: null,
        max_spread_pct: 0.5,
        sma_trend_confirmation: false,
        sma_trend_period: 50,
        min_rsi_change: 5.0,
        price_action_confirmation: false
      },
      divergence_config: {
        enabled: false,
        lookback_periods: 50,
        min_strength: 0.6,
        enable_regular_divergence: true,
        enable_hidden_divergence: false,
        swing_sensitivity: 1.5,
        min_swing_size: 2.0
      },
      exit_strategy: {
        strategy_type: 'RSIReversal',
        rsi_exit_levels: {
          long_exit_level: 70.0,
          short_exit_level: 30.0,
          centerline_exit: true
        },
        time_based_exit: null,
        profit_target_multiplier: 2.0,
        partial_exits: null
      },
      performance_config: {
        detailed_tracking: true,
        calculate_sharpe: true,
        risk_free_rate: 2.0,
        reporting_interval: 24,
        max_trade_history: 1000
      }
    }
  }

  private transformSMACrossoverConfig(config: any): any {
    return {
      fast_period: config.short_period || 20,
      slow_period: config.long_period || 50,
      position_size_pct: 5.0, // 5% of portfolio per trade
      risk_settings: {
        stop_loss_pct: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : 2.0,
        take_profit_pct: config.take_profit_percentage ? parseFloat(config.take_profit_percentage) : 4.0,
        max_position_pct: 10.0,
        min_signal_interval: 30,
        trailing_stop: false,
        trailing_stop_pct: null
      },
      filters: {
        min_volume: null,
        max_spread_pct: 0.5,
        rsi_overbought: 70.0,
        rsi_oversold: 30.0,
        macd_confirmation: false,
        min_sma_spread_pct: 0.1
      },
      enable_long: true,
      enable_short: false,
      use_market_orders: false,
      confirmation_indicators: {
        use_rsi: false,
        rsi_period: 14,
        use_macd: false,
        macd_fast: 12,
        macd_slow: 26,
        macd_signal: 9,
        use_volume: false,
        volume_period: 20,
        min_volume_multiplier: 1.5
      }
    }
  }

  private transformMACDConfig(config: any): any {
    return {
      fast_period: config.fast_period || 12,
      slow_period: config.slow_period || 26,
      signal_period: config.signal_period || 9,
      enable_long: true,
      enable_short: false,
      position_sizing: {
        sizing_method: 'PortfolioPercentage',
        fixed_size: null,
        portfolio_percentage: 5.0,
        risk_per_trade: 2.0,
        max_position_size: parseFloat(config.investment_amount || '400'),
        min_position_size: 10.0,
        scale_by_macd_strength: true
      },
      risk_management: {
        stop_loss_pct: config.stop_loss_percentage ? parseFloat(config.stop_loss_percentage) : 4.0,
        take_profit_pct: config.take_profit_percentage ? parseFloat(config.take_profit_percentage) : 12.0,
        max_drawdown_pct: 15.0,
        max_consecutive_losses: 3,
        cooldown_period: 30,
        macd_reversal_stop: true,
        histogram_stop_threshold: 0.0005,
        trailing_stop: {
          enabled: true,
          activation_threshold: 2.5,
          trailing_distance: 1.5,
          macd_based_trailing: true,
          macd_trailing_buffer: 0.0001
        }
      },
      signal_filters: {
        min_volume: null,
        max_spread_pct: 0.5,
        min_histogram_change: 0.0001,
        min_crossover_distance: 0.00001,
        sma_trend_confirmation: false,
        sma_trend_period: 50,
        price_action_confirmation: false,
        filter_consolidation: false
      },
      signal_config: {
        enable_signal_crossover: true,
        enable_zero_crossover: true,
        enable_divergence: false,
        enable_histogram: true,
        signal_strength: {
          min_strong_histogram: 0.0001,
          min_crossover_distance: 0.00005,
          min_momentum_change: 0.00001,
          require_histogram_acceleration: true
        },
        confirmation: {
          price_confirmation: true,
          price_confirmation_bars: 2,
          volume_confirmation: false,
          volume_increase_threshold: 1.2,
          trend_alignment: true,
          trend_sma_period: 50
        }
      },
      exit_strategy: {
        strategy_type: 'MACDReversal',
        macd_exit_conditions: {
          opposite_crossover: true,
          zero_line_exit: false,
          histogram_reversal: true,
          histogram_reversal_threshold: 0.0003,
          momentum_weakening: true,
          momentum_periods: 3
        },
        partial_exits: null,
        time_based_exit: null,
        profit_target_multiplier: 2.0
      },
      performance_config: {
        detailed_tracking: true,
        calculate_sharpe: true,
        risk_free_rate: 2.0,
        reporting_interval: 24,
        max_trade_history: 1000,
        track_macd_metrics: true
      }
    }
  }

  // Backtesting endpoints
  async runBacktest(data: BacktestRequest): Promise<BacktestEngineResult> {
    // Transform config based on strategy type
    let transformedConfig = data.config
    let stopLossPercentage: number | undefined
    let takeProfitPercentage: number | undefined

    switch (data.strategy_type) {
      case 'dca':
        transformedConfig = this.transformDCAConfig(data.config)
        // Extract stop loss and take profit from DCA config if present
        if ('stop_loss_percentage' in data.config && data.config.stop_loss_percentage) {
          stopLossPercentage = parseFloat(data.config.stop_loss_percentage.toString())
        }
        if ('take_profit_percentage' in data.config && data.config.take_profit_percentage) {
          takeProfitPercentage = parseFloat(data.config.take_profit_percentage.toString())
        }
        break
      case 'grid_trading':
        transformedConfig = this.transformGridTradingConfig(data.config)
        break
      case 'rsi':
        transformedConfig = this.transformRSIConfig(data.config)
        break
      case 'sma_crossover':
        transformedConfig = this.transformSMACrossoverConfig(data.config)
        break
      case 'macd':
        transformedConfig = this.transformMACDConfig(data.config)
        break
    }

    // Map frontend strategy types to backend strategy names
    const strategyNameMap: Record<StrategyType, string> = {
      'dca': 'dca_v2',
      'grid_trading': 'grid_trading_v2',
      'sma_crossover': 'sma_crossover_v2',
      'rsi': 'rsi_v1',
      'macd': 'macd_v1'
    }

    // Convert frontend format to backend format
    const backendRequest: BackendBacktestRequest = {
      symbol: data.asset_symbol,
      interval: data.interval || '1d', // Use provided interval or default to daily
      start_date: data.start_date + 'T00:00:00Z',
      end_date: data.end_date + 'T23:59:59Z',
      initial_balance: data.initial_capital,
      strategy_name: strategyNameMap[data.strategy_type] || data.strategy_type,
      strategy_parameters: transformedConfig,
      stop_loss_percentage: stopLossPercentage,
      take_profit_percentage: takeProfitPercentage
    }

    return this.request<BacktestEngineResult>('/backtesting/run', {
      method: 'POST',
      body: JSON.stringify(backendRequest),
    })
  }

  async getBacktestResult(backtestId: string): Promise<BacktestResultDetail> {
    return this.request<BacktestResultDetail>(`/backtesting/results/${backtestId}`)
  }

  async getUserBacktests(query?: BacktestListQuery): Promise<BacktestListResponse> {
    const searchParams = new URLSearchParams()
    if (query?.page) searchParams.append('page', query.page.toString())
    if (query?.limit) searchParams.append('limit', query.limit.toString())
    if (query?.strategy_name) searchParams.append('strategy_name', query.strategy_name)
    if (query?.symbol) searchParams.append('symbol', query.symbol)
    if (query?.status) searchParams.append('status', query.status)

    const url = `/backtesting/results${searchParams.toString() ? `?${searchParams.toString()}` : ''}`
    return this.request<BacktestListResponse>(url)
  }

  async deleteBacktest(backtestId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/backtesting/results/${backtestId}`, {
      method: 'DELETE',
    })
  }

  async compareStrategies(backtestIds: string[]): Promise<BacktestComparison> {
    return this.request<BacktestComparison>('/backtesting/compare', {
      method: 'POST',
      body: JSON.stringify({ backtest_ids: backtestIds }),
    })
  }

  async getAvailableAssets(): Promise<string[]> {
    return this.request<string[]>('/backtesting/assets')
  }

  async pauseDCAStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/dca/strategies/${strategyId}/pause`, {
      method: 'POST',
    })
  }

  async resumeDCAStrategy(strategyId: string): Promise<{ message: string }> {
    return this.request<{ message: string }>(`/dca/strategies/${strategyId}/resume`, {
      method: 'POST',
    })
  }

  // Strategy Template endpoints
  async getStrategyTemplates(): Promise<TemplatesResponse> {
    return this.request<TemplatesResponse>('/strategy-templates')
  }

  async getStrategyTemplate(templateId: string): Promise<StrategyTemplate> {
    return this.request<StrategyTemplate>(`/strategy-templates/${templateId}`)
  }

  async getRecommendedTemplates(profile: UserProfile): Promise<StrategyTemplate[]> {
    return this.request<StrategyTemplate[]>('/strategy-templates/recommend', {
      method: 'POST',
      body: JSON.stringify(profile),
    })
  }

  async validateStrategyParameters(templateId: string, parameters: any): Promise<{ valid: boolean; issues: string[] }> {
    return this.request<{ valid: boolean; issues: string[] }>(`/strategy-templates/${templateId}/validate`, {
      method: 'POST',
      body: JSON.stringify(parameters),
    })
  }


  // Template backtesting endpoints
  async runTemplateBacktest(templateId: string, request: BacktestRequest): Promise<BacktestResults> {
    return this.request<BacktestResults>(`/strategy-templates/${templateId}/backtest`, {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  async compareTemplates(templateIds: string[], request: BacktestRequest): Promise<TemplateComparison> {
    return this.request<TemplateComparison>('/strategy-templates/compare', {
      method: 'POST',
      body: JSON.stringify({
        template_ids: templateIds,
        ...request,
      }),
    })
  }

  async getBacktestHistory(templateId?: string): Promise<{
    backtests: Array<{
      id: string
      template_id: string
      template_name: string
      symbol: string
      start_date: string
      end_date: string
      total_return: string
      created_at: string
    }>
  }> {
    const endpoint = templateId
      ? `/strategy-templates/${templateId}/backtests`
      : '/strategy-templates/backtests'
    return this.request(endpoint)
  }
}

export const apiClient = new ApiClient(API_BASE_URL)
export { ApiError }