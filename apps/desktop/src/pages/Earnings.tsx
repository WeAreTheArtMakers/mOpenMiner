import { useState, useEffect, useMemo } from 'react'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/tauri'
import { useAppStore } from '@/stores/app'

// Approximate network hashrates and block rewards for estimation
const COIN_DATA: Record<string, { 
  networkHashrate: number
  blockReward: number
  blockTime: number
  symbol: string
  priceUsd: number
}> = {
  xmr: {
    networkHashrate: 2.5e9,
    blockReward: 0.6,
    blockTime: 120,
    symbol: 'XMR',
    priceUsd: 150,
  },
  vrsc: {
    networkHashrate: 50e12,
    blockReward: 12,
    blockTime: 60,
    symbol: 'VRSC',
    priceUsd: 0.5,
  },
  rtm: {
    networkHashrate: 15e9,
    blockReward: 2500,
    blockTime: 120,
    symbol: 'RTM',
    priceUsd: 0.001,
  },
}

// Pool info for tracking payments
const POOL_INFO: Record<string, {
  name: string
  minPayout: string
  dashboardUrl: (wallet: string) => string
}> = {
  'gulf.moneroocean.stream': {
    name: 'MoneroOcean',
    minPayout: '0.003 XMR',
    dashboardUrl: (wallet) => `https://moneroocean.stream/#/dashboard?addr=${wallet}`,
  },
  'pool.supportxmr.com': {
    name: 'SupportXMR',
    minPayout: '0.1 XMR',
    dashboardUrl: (wallet) => `https://supportxmr.com/#/dashboard?wallet=${wallet}`,
  },
  'xmr.nanopool.org': {
    name: 'Nanopool',
    minPayout: '0.1 XMR',
    dashboardUrl: (wallet) => `https://xmr.nanopool.org/account/${wallet}`,
  },
  'na.luckpool.net': {
    name: 'LuckPool',
    minPayout: '0.5 VRSC',
    dashboardUrl: (wallet) => `https://luckpool.net/verus/miner/${wallet}`,
  },
  'ap.luckpool.net': {
    name: 'LuckPool',
    minPayout: '0.5 VRSC',
    dashboardUrl: (wallet) => `https://luckpool.net/verus/miner/${wallet}`,
  },
  'eu.luckpool.net': {
    name: 'LuckPool',
    minPayout: '0.5 VRSC',
    dashboardUrl: (wallet) => `https://luckpool.net/verus/miner/${wallet}`,
  },
}

interface MiningRecord {
  id: string
  coin: string
  symbol: string
  pool: string
  wallet: string
  worker: string
  started_at: number
  ended_at: number
  duration_secs: number
  accepted_shares: number
  rejected_shares: number
  avg_hashrate: number
  algorithm: string
}

interface HistorySummary {
  total_sessions: number
  total_time_secs: number
  total_accepted_shares: number
  total_rejected_shares: number
  by_coin: CoinSummary[]
}

interface CoinSummary {
  coin: string
  symbol: string
  total_time_secs: number
  total_accepted: number
  total_rejected: number
  session_count: number
  wallets: string[]
}

interface PoolBalance {
  pool_name: string
  pending_balance: number
  total_paid: number
  min_payout: number
  symbol: string
  last_payment: number | null
  hashrate: number | null
}

// Supported pools for balance checking
const SUPPORTED_POOLS = [
  { id: 'gulf.moneroocean.stream', name: 'MoneroOcean', coin: 'XMR', minPayout: 0.003 },
  { id: 'pool.supportxmr.com', name: 'SupportXMR', coin: 'XMR', minPayout: 0.1 },
  { id: 'xmr.nanopool.org', name: 'Nanopool XMR', coin: 'XMR', minPayout: 0.1 },
]

export function Earnings() {
  const { status, coins } = useAppStore()
  const [records, setRecords] = useState<MiningRecord[]>([])
  const [summary, setSummary] = useState<HistorySummary | null>(null)
  const [selectedPeriod, setSelectedPeriod] = useState<'day' | 'week' | 'month' | 'all'>('all')
  const [loading, setLoading] = useState(true)
  const [poolBalances, setPoolBalances] = useState<Record<string, PoolBalance | null>>({})
  const [loadingBalances, setLoadingBalances] = useState<Set<string>>(new Set())
  
  // Manual wallet input for balance checking
  const [manualPool, setManualPool] = useState('gulf.moneroocean.stream')
  const [manualWallet, setManualWallet] = useState('')

  // Load history from backend
  useEffect(() => {
    loadHistory()
  }, [])

  // Refresh when mining stops
  useEffect(() => {
    if (!status.isRunning) {
      loadHistory()
    }
  }, [status.isRunning])

  const loadHistory = async () => {
    try {
      const [historyRecords, historySummary] = await Promise.all([
        invoke<MiningRecord[]>('get_mining_history'),
        invoke<HistorySummary>('get_history_summary'),
      ])
      setRecords(historyRecords)
      setSummary(historySummary)
    } catch (e) {
      console.error('Failed to load mining history:', e)
    } finally {
      setLoading(false)
    }
  }

  // Filter records by period
  const filteredRecords = useMemo(() => {
    if (selectedPeriod === 'all') return records
    
    const now = Date.now() / 1000
    const cutoffs: Record<string, number> = {
      day: now - 86400,
      week: now - 604800,
      month: now - 2592000,
    }
    const cutoff = cutoffs[selectedPeriod] || 0
    return records.filter(r => r.started_at >= cutoff)
  }, [records, selectedPeriod])

  // Calculate totals for filtered period
  const periodTotals = useMemo(() => {
    return {
      sessions: filteredRecords.length,
      time: filteredRecords.reduce((acc, r) => acc + r.duration_secs, 0),
      accepted: filteredRecords.reduce((acc, r) => acc + r.accepted_shares, 0),
      rejected: filteredRecords.reduce((acc, r) => acc + r.rejected_shares, 0),
    }
  }, [filteredRecords])

  // Calculate estimated earnings for current session
  const estimatedEarnings = useMemo(() => {
    if (!status.isRunning || !status.coin || status.hashrate === 0) return null

    const coinData = COIN_DATA[status.coin.toLowerCase()]
    if (!coinData) return null

    const shareOfNetwork = status.hashrate / coinData.networkHashrate
    const blocksPerDay = (24 * 60 * 60) / coinData.blockTime
    const dailyCoins = shareOfNetwork * blocksPerDay * coinData.blockReward
    const dailyUsd = dailyCoins * coinData.priceUsd

    return {
      hourly: { coins: dailyCoins / 24, usd: dailyUsd / 24 },
      daily: { coins: dailyCoins, usd: dailyUsd },
      weekly: { coins: dailyCoins * 7, usd: dailyUsd * 7 },
      monthly: { coins: dailyCoins * 30, usd: dailyUsd * 30 },
      symbol: coinData.symbol,
    }
  }, [status.isRunning, status.coin, status.hashrate])

  // Get unique pools and wallets from history AND current session for tracking
  const poolTracking = useMemo(() => {
    const tracking: { pool: string; wallet: string; coin: string; poolInfo: typeof POOL_INFO[string] | null }[] = []
    const seen = new Set<string>()
    
    // Add from history
    for (const record of records) {
      const poolHost = record.pool.replace(/^stratum\+tcp:\/\//, '').replace(/^stratum\+ssl:\/\//, '').split(':')[0]
      const key = `${poolHost}:${record.wallet}`
      
      if (!seen.has(key) && record.wallet) {
        seen.add(key)
        tracking.push({
          pool: poolHost,
          wallet: record.wallet,
          coin: record.coin,
          poolInfo: POOL_INFO[poolHost] || null,
        })
      }
    }
    
    return tracking
  }, [records])

  // Fetch pool balance
  const fetchBalance = async (poolHost: string, wallet: string) => {
    const key = `${poolHost}:${wallet}`
    if (loadingBalances.has(key)) return
    
    setLoadingBalances(prev => new Set(prev).add(key))
    try {
      const balance = await invoke<PoolBalance>('fetch_pool_balance', { poolHost, wallet })
      setPoolBalances(prev => ({ ...prev, [key]: balance }))
    } catch (e) {
      console.error('Failed to fetch balance:', e)
      setPoolBalances(prev => ({ ...prev, [key]: null }))
    } finally {
      setLoadingBalances(prev => {
        const next = new Set(prev)
        next.delete(key)
        return next
      })
    }
  }

  const formatDuration = (secs: number) => {
    const hours = Math.floor(secs / 3600)
    const minutes = Math.floor((secs % 3600) / 60)
    if (hours > 0) return `${hours}h ${minutes}m`
    return `${minutes}m`
  }

  const formatNumber = (n: number, decimals = 2) => {
    if (n < 0.0001) return '< 0.0001'
    if (n < 1) return n.toFixed(decimals + 2)
    return n.toFixed(decimals)
  }

  const handleClearHistory = async () => {
    if (confirm('Clear all mining history? This cannot be undone.')) {
      await invoke('clear_mining_history')
      loadHistory()
    }
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-[var(--text-secondary)]">Loading history...</p>
      </div>
    )
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <h1 className="text-2xl font-bold">Earnings & History</h1>

      {/* Current Earnings Estimate */}
      {estimatedEarnings && (
        <section className="rounded-xl border border-green-500/30 bg-green-500/5 p-6">
          <h2 className="text-lg font-semibold text-green-500 mb-4">
            💰 Estimated Earnings ({estimatedEarnings.symbol})
          </h2>
          <p className="text-xs text-[var(--text-secondary)] mb-4">
            Based on current hashrate of {status.hashrate.toFixed(1)} H/s
          </p>
          
          <div className="grid grid-cols-4 gap-4">
            <EarningsCard label="Hourly" coins={formatNumber(estimatedEarnings.hourly.coins, 6)} usd={estimatedEarnings.hourly.usd} symbol={estimatedEarnings.symbol} />
            <EarningsCard label="Daily" coins={formatNumber(estimatedEarnings.daily.coins, 4)} usd={estimatedEarnings.daily.usd} symbol={estimatedEarnings.symbol} />
            <EarningsCard label="Weekly" coins={formatNumber(estimatedEarnings.weekly.coins, 4)} usd={estimatedEarnings.weekly.usd} symbol={estimatedEarnings.symbol} />
            <EarningsCard label="Monthly" coins={formatNumber(estimatedEarnings.monthly.coins, 2)} usd={estimatedEarnings.monthly.usd} symbol={estimatedEarnings.symbol} />
          </div>

          <p className="mt-4 text-xs text-[var(--text-secondary)]">
            ⚠️ Estimates are approximate and based on current network conditions.
          </p>
        </section>
      )}

      {!estimatedEarnings && !status.isRunning && (
        <div className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6 text-center">
          <p className="text-[var(--text-secondary)]">Start mining to see estimated earnings</p>
        </div>
      )}

      {/* Pool Payment Tracking - Always show */}
      <section className="rounded-xl border border-blue-500/30 bg-blue-500/5 p-6">
        <h2 className="text-lg font-semibold text-blue-500 mb-2">🔗 Check Pool Balance</h2>
        <p className="text-xs text-[var(--text-secondary)] mb-4">
          Enter your wallet address to check your pending balance at the pool.
        </p>
        
        {/* Manual balance check */}
        <div className="p-4 rounded-lg bg-surface mb-4">
          <div className="grid gap-3 sm:grid-cols-3">
            <div>
              <label className="block text-xs font-medium mb-1">Pool</label>
              <select
                value={manualPool}
                onChange={(e) => setManualPool(e.target.value)}
                className="w-full rounded-lg border border-[var(--border)] bg-surface-elevated px-3 py-2 text-sm"
              >
                {SUPPORTED_POOLS.map(p => (
                  <option key={p.id} value={p.id}>{p.name} ({p.coin})</option>
                ))}
              </select>
            </div>
            <div className="sm:col-span-2">
              <label className="block text-xs font-medium mb-1">Wallet Address</label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={manualWallet}
                  onChange={(e) => setManualWallet(e.target.value)}
                  placeholder="Enter your XMR wallet address"
                  className="flex-1 rounded-lg border border-[var(--border)] bg-surface-elevated px-3 py-2 text-sm font-mono"
                />
                <button
                  onClick={() => manualWallet && fetchBalance(manualPool, manualWallet)}
                  disabled={!manualWallet || loadingBalances.has(`${manualPool}:${manualWallet}`)}
                  className="px-4 py-2 rounded-lg bg-blue-500 text-white text-sm font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {loadingBalances.has(`${manualPool}:${manualWallet}`) ? '...' : 'Check'}
                </button>
              </div>
            </div>
          </div>
          
          {/* Show balance for manual check */}
          {manualWallet && poolBalances[`${manualPool}:${manualWallet}`] && (
            <div className="mt-4 pt-4 border-t border-[var(--border)]">
              {(() => {
                const balance = poolBalances[`${manualPool}:${manualWallet}`]!
                return (
                  <>
                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <p className="text-xs text-[var(--text-secondary)]">Pending Balance</p>
                        <p className="font-mono font-bold text-green-500 text-lg">
                          {balance.pending_balance.toFixed(6)} {balance.symbol}
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-[var(--text-secondary)]">Total Paid</p>
                        <p className="font-mono font-medium">
                          {balance.total_paid.toFixed(4)} {balance.symbol}
                        </p>
                      </div>
                      <div>
                        <p className="text-xs text-[var(--text-secondary)]">Min Payout</p>
                        <p className="font-mono text-sm">
                          {balance.min_payout} {balance.symbol}
                        </p>
                      </div>
                    </div>
                    
                    {/* Progress to payout */}
                    <div className="mt-3">
                      <div className="flex justify-between text-xs text-[var(--text-secondary)] mb-1">
                        <span>Progress to payout</span>
                        <span>{Math.min(100, (balance.pending_balance / balance.min_payout * 100)).toFixed(1)}%</span>
                      </div>
                      <div className="h-3 bg-[var(--border)] rounded-full overflow-hidden">
                        <div 
                          className="h-full bg-gradient-to-r from-blue-500 to-green-500 rounded-full transition-all"
                          style={{ width: `${Math.min(100, balance.pending_balance / balance.min_payout * 100)}%` }}
                        />
                      </div>
                      {balance.pending_balance >= balance.min_payout && (
                        <p className="mt-2 text-xs text-green-500 font-medium">
                          ✓ Ready for payout! Check pool dashboard for payment schedule.
                        </p>
                      )}
                    </div>
                  </>
                )
              })()}
            </div>
          )}
        </div>

        {/* Previously used pools from history */}
        {poolTracking.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-sm font-medium text-[var(--text-secondary)]">Previously Used Pools</h3>
            {poolTracking.map(({ pool, wallet, coin, poolInfo }, idx) => {
              const balanceKey = `${pool}:${wallet}`
              const balance = poolBalances[balanceKey]
              const isLoading = loadingBalances.has(balanceKey)
              
              return (
                <div key={idx} className="p-4 rounded-lg bg-surface">
                  <div className="flex items-center justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{poolInfo?.name || pool}</span>
                        <span className="text-xs px-2 py-0.5 rounded bg-blue-500/10 text-blue-500">
                          {coin.toUpperCase()}
                        </span>
                      </div>
                      <p className="text-xs text-[var(--text-secondary)] truncate mt-1">
                        {wallet.slice(0, 16)}...{wallet.slice(-8)}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      {poolInfo && (
                        <button
                          onClick={() => fetchBalance(pool, wallet)}
                          disabled={isLoading}
                          className="px-3 py-1.5 rounded-lg bg-blue-500/10 text-blue-500 text-xs font-medium hover:bg-blue-500/20 disabled:opacity-50"
                        >
                          {isLoading ? '...' : '↻ Refresh'}
                        </button>
                      )}
                      {poolInfo ? (
                        <a
                          href={poolInfo.dashboardUrl(wallet)}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="px-3 py-1.5 rounded-lg bg-blue-500 text-white text-xs font-medium hover:bg-blue-600 transition-colors"
                        >
                          Dashboard →
                        </a>
                      ) : (
                        <span className="text-xs text-[var(--text-secondary)]">
                          Check pool website
                        </span>
                      )}
                    </div>
                  </div>
                  
                  {/* Balance display */}
                  {balance && (
                    <div className="mt-3 pt-3 border-t border-[var(--border)]">
                      <div className="grid grid-cols-3 gap-4">
                        <div>
                          <p className="text-xs text-[var(--text-secondary)]">Pending Balance</p>
                          <p className="font-mono font-bold text-green-500">
                            {balance.pending_balance.toFixed(6)} {balance.symbol}
                          </p>
                        </div>
                        <div>
                          <p className="text-xs text-[var(--text-secondary)]">Total Paid</p>
                          <p className="font-mono font-medium">
                            {balance.total_paid.toFixed(4)} {balance.symbol}
                          </p>
                        </div>
                        <div>
                          <p className="text-xs text-[var(--text-secondary)]">Min Payout</p>
                          <p className="font-mono text-sm">
                            {balance.min_payout} {balance.symbol}
                          </p>
                        </div>
                      </div>
                      
                      {/* Progress to payout */}
                      {balance.pending_balance > 0 && (
                        <div className="mt-3">
                          <div className="flex justify-between text-xs text-[var(--text-secondary)] mb-1">
                            <span>Progress to payout</span>
                            <span>{Math.min(100, (balance.pending_balance / balance.min_payout * 100)).toFixed(1)}%</span>
                          </div>
                          <div className="h-2 bg-[var(--border)] rounded-full overflow-hidden">
                            <div 
                              className="h-full bg-green-500 rounded-full transition-all"
                              style={{ width: `${Math.min(100, balance.pending_balance / balance.min_payout * 100)}%` }}
                            />
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        )}
        
        <div className="mt-4 p-3 rounded-lg bg-surface">
          <p className="text-xs text-[var(--text-secondary)]">
            <strong>💡 How payments work:</strong> Your mining shares accumulate at the pool. 
            When your balance reaches the minimum payout threshold, the pool automatically 
            sends coins to your wallet. This can take hours to days depending on your hashrate.
          </p>
        </div>
      </section>

      {/* Mining History */}
      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Mining History</h2>
          <div className="flex gap-1">
            {(['day', 'week', 'month', 'all'] as const).map((period) => (
              <button
                key={period}
                onClick={() => setSelectedPeriod(period)}
                className={clsx(
                  'px-3 py-1 text-xs rounded-md transition-colors',
                  selectedPeriod === period
                    ? 'bg-accent text-white'
                    : 'bg-surface hover:bg-surface-elevated text-[var(--text-secondary)]'
                )}
              >
                {period === 'day' ? '24h' : period === 'week' ? '7d' : period === 'month' ? '30d' : 'All'}
              </button>
            ))}
          </div>
        </div>

        {/* Summary Stats */}
        <div className="grid grid-cols-4 gap-4 mb-6">
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Sessions</p>
            <p className="text-xl font-mono font-bold mt-1">{periodTotals.sessions}</p>
          </div>
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Total Time</p>
            <p className="text-xl font-mono font-bold mt-1">{formatDuration(periodTotals.time)}</p>
          </div>
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Accepted</p>
            <p className="text-xl font-mono font-bold mt-1 text-green-500">{periodTotals.accepted}</p>
          </div>
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Rejected</p>
            <p className="text-xl font-mono font-bold mt-1 text-red-500">{periodTotals.rejected}</p>
          </div>
        </div>

        {/* By Coin Breakdown */}
        {summary && summary.by_coin.length > 0 && (
          <div className="mb-6">
            <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">By Coin (All Time)</h3>
            <div className="space-y-2">
              {summary.by_coin.map((coin) => (
                <div key={coin.coin} className="flex items-center justify-between p-3 rounded-lg bg-surface">
                  <div className="flex items-center gap-3">
                    <span className="text-lg font-bold">{coin.symbol}</span>
                    <span className="text-xs text-[var(--text-secondary)]">{coin.session_count} sessions</span>
                  </div>
                  <div className="flex items-center gap-6 text-sm">
                    <span className="text-[var(--text-secondary)]">{formatDuration(coin.total_time_secs)}</span>
                    <span className="text-green-500 font-mono">{coin.total_accepted} shares</span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </section>

      {/* Recent Sessions */}
      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="text-lg font-semibold mb-4">Recent Sessions</h2>
        
        {filteredRecords.length > 0 ? (
          <div className="space-y-2 max-h-80 overflow-y-auto">
            {filteredRecords.slice().reverse().slice(0, 20).map((record) => {
              const coinInfo = coins.find(c => c.id === record.coin)
              
              return (
                <div key={record.id} className="flex items-center justify-between p-3 rounded-lg bg-surface">
                  <div className="flex items-center gap-3">
                    <span className="font-medium">{coinInfo?.symbol || record.symbol || record.coin.toUpperCase()}</span>
                    <span className="text-xs text-[var(--text-secondary)]">
                      {new Date(record.started_at * 1000).toLocaleString()}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-[var(--text-secondary)]">{formatDuration(record.duration_secs)}</span>
                    <span className="text-green-500 font-mono">{record.accepted_shares} shares</span>
                    {record.avg_hashrate > 0 && (
                      <span className="text-xs text-[var(--text-secondary)]">{record.avg_hashrate.toFixed(1)} H/s</span>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <p className="text-center text-[var(--text-secondary)] py-4">
            No mining sessions recorded yet
          </p>
        )}

        {records.length > 0 && (
          <button
            onClick={handleClearHistory}
            className="mt-4 text-xs text-red-500 hover:underline"
          >
            Clear All History
          </button>
        )}
      </section>
    </div>
  )
}

function EarningsCard({ label, coins, usd, symbol }: { label: string; coins: string; usd: number; symbol: string }) {
  return (
    <div className="rounded-lg bg-surface p-3 text-center">
      <p className="text-xs text-[var(--text-secondary)] uppercase">{label}</p>
      <p className="text-lg font-mono font-bold mt-1">{coins}</p>
      <p className="text-xs text-[var(--text-secondary)]">{symbol}</p>
      <p className="text-xs text-green-500 mt-1">${usd.toFixed(4)}</p>
    </div>
  )
}
