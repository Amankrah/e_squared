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

// DCA Strategy Types
export interface DCAStrategy {
  id: string
  user_id: string
  name: string
  asset_symbol: string
  total_allocation: string
  base_tranche_size: string
  status: string
  strategy_type: string
  sentiment_multiplier: boolean
  volatility_adjustment: boolean
  fear_greed_threshold_buy: number
  fear_greed_threshold_sell: number
  max_tranche_percentage: string
  min_tranche_percentage: string
  dca_interval_hours: number
  target_zones?: string[]
  stop_loss_percentage?: string
  take_profit_percentage?: string
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
  total_allocation: number
  base_tranche_percentage: number
  strategy_type: string
  sentiment_multiplier: boolean
  volatility_adjustment: boolean
  fear_greed_threshold_buy: number
  fear_greed_threshold_sell: number
  dca_interval_hours: number
  target_zones?: number[]
  stop_loss_percentage?: number
  take_profit_percentage?: number
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
  lower_price: string
  upper_price: string
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
export type StrategyConfig = GridTradingConfig | SMACrossoverConfig | RSIConfig | MACDConfig
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
}

export interface BacktestResult {
  id: string
  strategy_type: StrategyType
  asset_symbol: string
  start_date: string
  end_date: string
  initial_capital: string
  final_capital: string
  total_return: string
  total_return_percentage: string
  max_drawdown: string
  max_drawdown_percentage: string
  sharpe_ratio: string
  sortino_ratio: string
  win_rate: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  average_win: string
  average_loss: string
  profit_factor: string
  volatility: string
  daily_returns: DailyReturn[]
  trades: BacktestTrade[]
  metrics: BacktestMetrics
  created_at: string
}

export interface DailyReturn {
  date: string
  portfolio_value: string
  daily_return: string
  cumulative_return: string
  drawdown: string
}

export interface BacktestTrade {
  entry_date: string
  exit_date: string
  entry_price: string
  exit_price: string
  quantity: string
  side: 'buy' | 'sell'
  pnl: string
  pnl_percentage: string
  duration_hours: number
  signal_reason: string
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

export interface BacktestRequest {
  symbol: string
  start_date: string
  end_date: string
  interval: string
  total_allocation: number
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
    return this.request<DCAStrategy>('/dca/strategies', {
      method: 'POST',
      body: JSON.stringify(data),
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
    return this.request<GridTradingStrategy>('/grid-trading/strategies', {
      method: 'POST',
      body: JSON.stringify(data),
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
    return this.request<SMACrossoverStrategy>('/sma-crossover/strategies', {
      method: 'POST',
      body: JSON.stringify(data),
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
    return this.request<RSIStrategy>('/rsi/strategies', {
      method: 'POST',
      body: JSON.stringify(data),
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
    return this.request<MACDStrategy>('/macd/strategies', {
      method: 'POST',
      body: JSON.stringify(data),
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

  // Unified Strategy Management
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

  // Backtesting endpoints
  async runBacktest(data: BacktestRequest): Promise<BacktestResult> {
    return this.request<BacktestResult>('/backtesting/run', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async getBacktestResult(backtestId: string): Promise<BacktestResult> {
    return this.request<BacktestResult>(`/backtesting/results/${backtestId}`)
  }

  async getUserBacktests(): Promise<BacktestResult[]> {
    return this.request<BacktestResult[]>('/backtesting/results')
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


  // Backtesting endpoints
  async runBacktest(templateId: string, request: BacktestRequest): Promise<BacktestResults> {
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