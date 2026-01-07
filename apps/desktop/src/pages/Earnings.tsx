import { useState, useEffect, useMemo } from 'react'
import { clsx } from 'clsx'
import { useAppStore } from '@/stores/app'

// Approximate network hashrates and block rewards for estimation
// These are rough estimates - real values change constantly
const COIN_DATA: Record<string, { 
  networkHashrate: number  // H/s
  blockReward: number
  blockTime: number  // seconds
  symbol: string
  priceUsd: number  // approximate
}> = {
  xmr: {
    networkHashrate: 2.5e9,  // ~2.5 GH/s
    blockReward: 0.6,
    blockTime: 120,
    symbol: 'XMR',
    priceUsd: 150,
  },
  vrsc: {
    networkHashrate: 50e12,  // ~50 TH/s (VerusHash)
    blockReward: 12,
    blockTime: 60,
    symbol: 'VRSC',
    priceUsd: 0.5,
  },
  rtm: {
    networkHashrate: 15e9,  // ~15 GH/s
    blockReward: 2500,
    blockTime: 120,
    symbol: 'RTM',
    priceUsd: 0.001,
  },
}

interface MiningSession {
  id: string
  coin: string
  pool: string
  startedAt: number
  endedAt: number | null
  totalShares: number
  hashrate: number
}

export function Earnings() {
  const { status, coins } = useAppStore()
  const [sessions, setSessions] = useState<MiningSession[]>([])
  const [selectedPeriod, setSelectedPeriod] = useState<'day' | 'week' | 'month'>('day')

  // Load sessions from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('mining_sessions')
    if (saved) {
      setSessions(JSON.parse(saved))
    }
  }, [])

  // Track current session
  useEffect(() => {
    if (status.isRunning && status.coin) {
      const currentSession = sessions.find(s => s.endedAt === null)
      if (!currentSession) {
        // Start new session
        const newSession: MiningSession = {
          id: crypto.randomUUID(),
          coin: status.coin,
          pool: status.pool || '',
          startedAt: Date.now(),
          endedAt: null,
          totalShares: status.acceptedShares,
          hashrate: status.hashrate,
        }
        const updated = [...sessions, newSession]
        setSessions(updated)
        localStorage.setItem('mining_sessions', JSON.stringify(updated))
      } else {
        // Update current session
        const updated = sessions.map(s => 
          s.id === currentSession.id 
            ? { ...s, totalShares: status.acceptedShares, hashrate: status.avgHashrate || status.hashrate }
            : s
        )
        setSessions(updated)
        localStorage.setItem('mining_sessions', JSON.stringify(updated))
      }
    } else {
      // End current session
      const currentSession = sessions.find(s => s.endedAt === null)
      if (currentSession) {
        const updated = sessions.map(s =>
          s.id === currentSession.id
            ? { ...s, endedAt: Date.now() }
            : s
        )
        setSessions(updated)
        localStorage.setItem('mining_sessions', JSON.stringify(updated))
      }
    }
  }, [status.isRunning, status.acceptedShares])

  // Calculate estimated earnings
  const estimatedEarnings = useMemo(() => {
    if (!status.isRunning || !status.coin || status.hashrate === 0) {
      return null
    }

    const coinData = COIN_DATA[status.coin.toLowerCase()]
    if (!coinData) {
      return null
    }

    // Your share of network hashrate
    const shareOfNetwork = status.hashrate / coinData.networkHashrate
    
    // Blocks per day
    const blocksPerDay = (24 * 60 * 60) / coinData.blockTime
    
    // Daily coin earnings
    const dailyCoins = shareOfNetwork * blocksPerDay * coinData.blockReward
    
    // USD value
    const dailyUsd = dailyCoins * coinData.priceUsd

    return {
      hourly: { coins: dailyCoins / 24, usd: dailyUsd / 24 },
      daily: { coins: dailyCoins, usd: dailyUsd },
      weekly: { coins: dailyCoins * 7, usd: dailyUsd * 7 },
      monthly: { coins: dailyCoins * 30, usd: dailyUsd * 30 },
      symbol: coinData.symbol,
    }
  }, [status.isRunning, status.coin, status.hashrate])

  // Calculate totals
  const totals = useMemo(() => {
    const now = Date.now()
    const periodMs = selectedPeriod === 'day' ? 86400000 : selectedPeriod === 'week' ? 604800000 : 2592000000
    const cutoff = now - periodMs

    const periodSessions = sessions.filter(s => s.startedAt >= cutoff)
    
    const totalTime = periodSessions.reduce((acc, s) => {
      const end = s.endedAt || now
      return acc + (end - s.startedAt)
    }, 0)

    const totalShares = periodSessions.reduce((acc, s) => acc + s.totalShares, 0)
    
    const byCoin = periodSessions.reduce((acc, s) => {
      if (!acc[s.coin]) {
        acc[s.coin] = { time: 0, shares: 0 }
      }
      const end = s.endedAt || now
      acc[s.coin].time += end - s.startedAt
      acc[s.coin].shares += s.totalShares
      return acc
    }, {} as Record<string, { time: number; shares: number }>)

    return { totalTime, totalShares, byCoin, sessionCount: periodSessions.length }
  }, [sessions, selectedPeriod])

  const formatDuration = (ms: number) => {
    const hours = Math.floor(ms / 3600000)
    const minutes = Math.floor((ms % 3600000) / 60000)
    if (hours > 0) {
      return `${hours}h ${minutes}m`
    }
    return `${minutes}m`
  }

  const formatNumber = (n: number, decimals = 2) => {
    if (n < 0.0001) return '< 0.0001'
    if (n < 1) return n.toFixed(decimals + 2)
    return n.toFixed(decimals)
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <h1 className="text-2xl font-bold">Earnings</h1>

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
            <EarningsCard 
              label="Hourly" 
              coins={formatNumber(estimatedEarnings.hourly.coins, 6)} 
              usd={estimatedEarnings.hourly.usd}
              symbol={estimatedEarnings.symbol}
            />
            <EarningsCard 
              label="Daily" 
              coins={formatNumber(estimatedEarnings.daily.coins, 4)} 
              usd={estimatedEarnings.daily.usd}
              symbol={estimatedEarnings.symbol}
            />
            <EarningsCard 
              label="Weekly" 
              coins={formatNumber(estimatedEarnings.weekly.coins, 4)} 
              usd={estimatedEarnings.weekly.usd}
              symbol={estimatedEarnings.symbol}
            />
            <EarningsCard 
              label="Monthly" 
              coins={formatNumber(estimatedEarnings.monthly.coins, 2)} 
              usd={estimatedEarnings.monthly.usd}
              symbol={estimatedEarnings.symbol}
            />
          </div>

          <p className="mt-4 text-xs text-[var(--text-secondary)]">
            ⚠️ Estimates are approximate and based on current network conditions. Actual earnings may vary.
          </p>
        </section>
      )}

      {!estimatedEarnings && !status.isRunning && (
        <div className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6 text-center">
          <p className="text-[var(--text-secondary)]">Start mining to see estimated earnings</p>
        </div>
      )}

      {/* Mining History */}
      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Mining History</h2>
          <div className="flex gap-1">
            {(['day', 'week', 'month'] as const).map((period) => (
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
                {period === 'day' ? '24h' : period === 'week' ? '7d' : '30d'}
              </button>
            ))}
          </div>
        </div>

        {/* Summary Stats */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Total Time</p>
            <p className="text-xl font-mono font-bold mt-1">{formatDuration(totals.totalTime)}</p>
          </div>
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Total Shares</p>
            <p className="text-xl font-mono font-bold mt-1 text-green-500">{totals.totalShares}</p>
          </div>
          <div className="rounded-lg bg-surface p-4 text-center">
            <p className="text-xs text-[var(--text-secondary)] uppercase">Sessions</p>
            <p className="text-xl font-mono font-bold mt-1">{totals.sessionCount}</p>
          </div>
        </div>

        {/* By Coin Breakdown */}
        {Object.keys(totals.byCoin).length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-[var(--text-secondary)] mb-3">By Coin</h3>
            <div className="space-y-2">
              {Object.entries(totals.byCoin).map(([coin, data]) => {
                const coinInfo = coins.find(c => c.id === coin)
                return (
                  <div key={coin} className="flex items-center justify-between p-3 rounded-lg bg-surface">
                    <div className="flex items-center gap-3">
                      <span className="text-lg font-bold">{coinInfo?.symbol || coin.toUpperCase()}</span>
                      <span className="text-sm text-[var(--text-secondary)]">{coinInfo?.name}</span>
                    </div>
                    <div className="flex items-center gap-6 text-sm">
                      <span className="text-[var(--text-secondary)]">{formatDuration(data.time)}</span>
                      <span className="text-green-500 font-mono">{data.shares} shares</span>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {Object.keys(totals.byCoin).length === 0 && (
          <p className="text-center text-[var(--text-secondary)] py-4">
            No mining sessions in this period
          </p>
        )}
      </section>

      {/* Recent Sessions */}
      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="text-lg font-semibold mb-4">Recent Sessions</h2>
        
        {sessions.length > 0 ? (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {sessions.slice(-10).reverse().map((session) => {
              const coinInfo = coins.find(c => c.id === session.coin)
              const duration = (session.endedAt || Date.now()) - session.startedAt
              const isActive = session.endedAt === null
              
              return (
                <div 
                  key={session.id} 
                  className={clsx(
                    'flex items-center justify-between p-3 rounded-lg',
                    isActive ? 'bg-green-500/10 border border-green-500/30' : 'bg-surface'
                  )}
                >
                  <div className="flex items-center gap-3">
                    {isActive && (
                      <span className="relative flex h-2 w-2">
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
                        <span className="relative inline-flex h-2 w-2 rounded-full bg-green-500" />
                      </span>
                    )}
                    <span className="font-medium">{coinInfo?.symbol || session.coin.toUpperCase()}</span>
                    <span className="text-xs text-[var(--text-secondary)]">
                      {new Date(session.startedAt).toLocaleString()}
                    </span>
                  </div>
                  <div className="flex items-center gap-4 text-sm">
                    <span className="text-[var(--text-secondary)]">{formatDuration(duration)}</span>
                    <span className="text-green-500 font-mono">{session.totalShares} shares</span>
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <p className="text-center text-[var(--text-secondary)] py-4">
            No mining sessions yet
          </p>
        )}

        {sessions.length > 0 && (
          <button
            onClick={() => {
              if (confirm('Clear all mining history?')) {
                setSessions([])
                localStorage.removeItem('mining_sessions')
              }
            }}
            className="mt-4 text-xs text-red-500 hover:underline"
          >
            Clear History
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
