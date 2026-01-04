import { useState } from 'react'
import { clsx } from 'clsx'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/tauri'

type PoolStatus = 'ok' | 'degraded' | 'down'

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

  const getStatusColor = (status: PoolStatus) => {
    switch (status) {
      case 'ok': return 'bg-green-500'
      case 'degraded': return 'bg-yellow-500'
      case 'down': return 'bg-red-500'
    }
  }

  const getStatusLabel = (status: PoolStatus) => {
    switch (status) {
      case 'ok': return 'OK'
      case 'degraded': return 'Degraded'
      case 'down': return 'Down'
    }
  }

  return (
    <div className="mx-auto max-w-4xl space-y-8">
      <header>
        <h1 className="text-2xl font-semibold">Pools</h1>
        <p className="mt-1 text-[var(--text-secondary)]">View and test mining pool connections</p>
      </header>

      {coins.map((coin) => (
        <section key={coin.id} className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
          <h2 className="mb-4 text-lg font-medium">{coin.name} ({coin.symbol})</h2>
          <div className="space-y-3">
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
    </div>
  )
}


interface PoolRowProps {
  name: string
  url: string
  region: string
  tls: boolean
  health?: {
    status: 'ok' | 'degraded' | 'down'
    connected: boolean
    tls_verified: boolean | null
    latency_ms: number | null
    error: string | null
  }
  isChecking: boolean
  onCheck: () => void
  getStatusColor: (status: 'ok' | 'degraded' | 'down') => string
  getStatusLabel: (status: 'ok' | 'degraded' | 'down') => string
}

function PoolRow({ name, url, region, tls, health, isChecking, onCheck, getStatusColor, getStatusLabel }: PoolRowProps) {
  return (
    <div className="flex items-center justify-between rounded-lg border border-[var(--border)] bg-surface p-4">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="font-medium">{name}</span>
          <span className="rounded bg-[var(--border)] px-2 py-0.5 text-xs">{region}</span>
          {tls && (
            <span className="rounded bg-green-500/10 px-2 py-0.5 text-xs text-green-600 dark:text-green-400">TLS</span>
          )}
        </div>
        <p className="mt-1 truncate font-mono text-xs text-[var(--text-secondary)]">{url}</p>
        {health?.error && (
          <p className="mt-1 text-xs text-red-500">{health.error}</p>
        )}
      </div>
      
      <div className="ml-4 flex items-center gap-3">
        {health && (
          <div className="flex items-center gap-2">
            <span className={clsx('h-2 w-2 rounded-full', getStatusColor(health.status))} />
            <span className="text-xs text-[var(--text-secondary)]">
              {getStatusLabel(health.status)}
              {health.latency_ms && ` (${health.latency_ms}ms)`}
            </span>
          </div>
        )}
        <button
          onClick={onCheck}
          disabled={isChecking}
          className="rounded-lg border border-[var(--border)] px-3 py-1.5 text-sm transition-colors hover:bg-[var(--border)] disabled:opacity-50"
        >
          {isChecking ? 'Testing...' : 'Test'}
        </button>
      </div>
    </div>
  )
}
