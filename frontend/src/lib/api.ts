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

export interface WalletBalance {
  id: string
  exchange_connection_id: string
  wallet_type: 'spot' | 'margin' | 'futures' | 'funding' | 'earn' | 'options'
  asset_symbol: string
  free_balance: string
  locked_balance: string
  total_balance: string
  usd_value?: string
  last_updated: string
  created_at: string
}

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

export interface WalletBalancesResponse {
  balances: WalletBalance[]
  message?: string
}

export interface SyncBalancesResponse {
  message: string
  synced_balances: number
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

    const response = await fetch(url, {
      ...options,
      headers,
      credentials: 'include', // This ensures cookies are sent
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Unknown error' }))
      throw new ApiError(response.status, errorData.message || errorData.error || 'Request failed')
    }

    return response.json()
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
    return this.request<User>('/auth/me')
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
    await this.request<{ message: string }>('/auth/logout', {
      method: 'POST',
    })
    this.setCsrfToken(null)
  }

  async getCsrfTokenFromServer(): Promise<string> {
    const response = await this.request<{ csrf_token: string }>('/auth/csrf-token')
    this.setCsrfToken(response.csrf_token)
    return response.csrf_token
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

  async syncExchangeConnection(connectionId: string, password: string): Promise<any> {
    return this.request<any>(`/exchanges/connections/${connectionId}/sync`, {
      method: 'POST',
      body: JSON.stringify({ password }),
    })
  }

  async syncExchangeBalances(connectionId: string, password: string): Promise<any> {
    return this.request<any>(`/exchanges/connections/${connectionId}/sync`, {
      method: 'POST',
      body: JSON.stringify({ password })
    })
  }

  async getWalletBalances(connectionId: string): Promise<WalletBalancesResponse> {
    // This endpoint returns stored balances from database - kept for compatibility
    return this.request<WalletBalancesResponse>(`/exchanges/connections/${connectionId}/balances`)
  }

  async getLiveWalletBalances(connectionId: string, password: string): Promise<any> {
    return this.request<any>(`/exchanges/connections/${connectionId}/live-balances`, {
      method: 'POST',
      body: JSON.stringify({ password })
    })
  }

  async getAllUserBalances(): Promise<WalletBalancesResponse> {
    return this.request<WalletBalancesResponse>('/exchanges/balances')
  }

  async getAllLiveUserBalances(password: string): Promise<any> {
    return this.request<any>('/exchanges/live-balances', {
      method: 'POST',
      body: JSON.stringify({ password })
    })
  }

  // New method to get live balances for connections display
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

  // New exchange account endpoints
  async getAllExchangeAccounts(): Promise<AccountResponse[]> {
    return this.request<AccountResponse[]>('/exchange-accounts/all')
  }

  async getSpotAccount(connectionId: string): Promise<SpotAccountResponse> {
    return this.request<SpotAccountResponse>(`/exchange-accounts/spot?connection_id=${connectionId}`)
  }

  async getMarginAccount(connectionId: string): Promise<MarginAccountResponse> {
    return this.request<MarginAccountResponse>(`/exchange-accounts/margin?connection_id=${connectionId}`)
  }

  async getFuturesAccount(connectionId: string): Promise<FuturesAccountResponse> {
    return this.request<FuturesAccountResponse>(`/exchange-accounts/futures?connection_id=${connectionId}`)
  }

  async testExchangeConnection(connectionId: string): Promise<{
    connection_id: string
    exchange_name: string
    is_connected: boolean
    tested_at: string
  }> {
    return this.request(`/exchange-accounts/test?connection_id=${connectionId}`)
  }

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