import { useState } from 'react'
import { clsx } from 'clsx'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/tauri'

type PoolStatus = 'ok' | 'degraded' | 'down' | 'unknown'

interface PoolHealthResult {
  url: string
  status: PoolStatus
  connected: boolean
  tls_verified: boolean | null
  latency_ms: number | null
  error: string | null
}

export function Pools() {
  const { coins } = useAppStore()
  const [healthResults, setHealthResults] = useState<Record<string, PoolHealthResult>>({})
  const [checking, setChecking] = useState<string | null>(null)
  const [checkingAll, setCheckingAll] = useState(false)

  const handleHealthCheck = async (url: string) => {
    setChecking(url)
    try {
      const result = await invoke<PoolHealthResult>('check_pool_health', { url })
      setHealthResults((prev) => ({ ...prev, [url]: result }))
    } catch (e) {
      setHealthResults((prev) => ({
        ...prev,
        [url]: { url, status: 'down', connected: false, tls_verified: null, latency_ms: null, error: String(e) },
      }))
    } finally {
      setChecking(null)
    }
  }

  const handleCheckAllForCoin = async (coinId: string) => {
    const coin = coins.find(c => c.id === coinId)
    if (!coin) return
    
    setCheckingAll(true)
    for (const pool of coin.default_pools) {
      await handleHealthCheck(pool.stratum_url)
    }
    setCheckingAll(false)
  }

  const getStatusColor = (status: PoolStatus) => {
    switch (status) {
      case 'ok': return 'bg-green-500'
      case 'degraded': return 'bg-yellow-500'
      case 'down': return 'bg-red-500'
      case 'unknown': return 'bg-gray-400'
    }
  }

  const getStatusLabel = (status: PoolStatus) => {
    switch (status) {
      case 'ok': return 'OK'
      case 'degraded': return 'Slow'
      case 'down': return 'Down'
      case 'unknown': return '?'
    }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <header>
        <h1 className="text-2xl font-semibold">Pools</h1>
        <p className="mt-1 text-sm text-[var(--text-secondary)]">Test mining pool connections before starting</p>
      </header>

      {coins.filter(c => c.cpu_mineable).map((coin) => (
        <section key={coin.id} className="rounded-xl border border-[var(--border)] bg-surface-elevated p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-base font-medium">{coin.name} ({coin.symbol})</h2>
            <button
              onClick={() => handleCheckAllForCoin(coin.id)}
              disabled={checkingAll || checking !== null}
              className="text-xs px-2.5 py-1 rounded bg-accent/10 text-accent hover:bg-accent/20 disabled:opacity-50 transition-colors"
            >
              {checkingAll ? 'Checking...' : 'Check All'}
            </button>
          </div>
          <div className="space-y-2">
            {coin.default_pools.map((pool) => {
              const health = healthResults[pool.stratum_url]
              const isChecking = checking === pool.stratum_url
              
              return (
                <PoolRow
                  key={pool.stratum_url}
                  name={pool.name}
                  url={pool.stratum_url}
                  region={pool.region}
                  tls={pool.tls}
                  health={health}
                  isChecking={isChecking}
                  onCheck={() => handleHealthCheck(pool.stratum_url)}
                  getStatusColor={getStatusColor}
                  getStatusLabel={getStatusLabel}
                />
              )
            })}
          </div>
        </section>
      ))}

      {/* Non-CPU mineable coins */}
      {coins.filter(c => !c.cpu_mineable).length > 0 && (
        <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-5 opacity-60">
          <h2 className="text-base font-medium mb-2">GPU/ASIC Only Coins</h2>
          <p className="text-xs text-[var(--text-secondary)]">
            {coins.filter(c => !c.cpu_mineable).map(c => c.symbol).join(', ')} - requires external hardware
          </p>
        </section>
      )}
    </div>
  )
}


interface PoolRowProps {
  name: string
  url: string
  region: string
  tls: boolean
  health?: {
    status: 'ok' | 'degraded' | 'down' | 'unknown'
    connected: boolean
    tls_verified: boolean | null
    latency_ms: number | null
    error: string | null
  }
  isChecking: boolean
  onCheck: () => void
  getStatusColor: (status: 'ok' | 'degraded' | 'down' | 'unknown') => string
  getStatusLabel: (status: 'ok' | 'degraded' | 'down' | 'unknown') => string
}

function PoolRow({ name, url, region, tls, health, isChecking, onCheck, getStatusColor, getStatusLabel }: PoolRowProps) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-[var(--border)] bg-surface p-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{name}</span>
          <span className="rounded bg-[var(--border)] px-1.5 py-0.5 text-xs">{region}</span>
          {tls && (
            <span className="rounded bg-green-500/10 px-1.5 py-0.5 text-xs text-green-600 dark:text-green-400">TLS</span>
          )}
        </div>
        <p className="mt-0.5 truncate font-mono text-xs text-[var(--text-secondary)]">{url}</p>
        {health?.error && (
          <p className="mt-0.5 text-xs text-red-500 truncate" title={health.error}>{health.error}</p>
        )}
      </div>
      
      <div className="ml-3 flex items-center gap-2">
        {health && (
          <div className="flex items-center gap-1.5">
            <span className={clsx('h-2 w-2 rounded-full', getStatusColor(health.status))} />
            <span className="text-xs text-[var(--text-secondary)]">
              {getStatusLabel(health.status)}
              {health.latency_ms && ` ${health.latency_ms}ms`}
            </span>
          </div>
        )}
        <button
          onClick={onCheck}
          disabled={isChecking}
          className="rounded border border-[var(--border)] px-2 py-1 text-xs transition-colors hover:bg-[var(--border)] disabled:opacity-50"
          title="Test connection"
        >
          {isChecking ? '...' : '↻'}
        </button>
      </div>
    </div>
  )
}
