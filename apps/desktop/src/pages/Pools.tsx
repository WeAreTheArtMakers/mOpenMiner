import { useState, useCallback } from 'react'
import { clsx } from 'clsx'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/tauri'
import { AddPoolDialog } from '@/components/pools/AddPoolDialog'
import { PoolDiscoveryDialog } from '@/components/pools/PoolDiscoveryDialog'

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
  const { coins, loadCoins } = useAppStore()
  const [healthResults, setHealthResults] = useState<Record<string, PoolHealthResult>>({})
  const [checking, setChecking] = useState<string | null>(null)
  const [checkingAll, setCheckingAll] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [addPoolForCoin, setAddPoolForCoin] = useState<{ id: string; name: string } | null>(null)
  const [discoverForCoin, setDiscoverForCoin] = useState<{ id: string; name: string; symbol: string } | null>(null)
  const [confirmRemove, setConfirmRemove] = useState<{ coinId: string; coinName: string; poolName: string; url: string } | null>(null)

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

  const handleRefreshCoins = async () => {
    setRefreshing(true)
    await loadCoins()
    setRefreshing(false)
  }

  const handleRemovePool = useCallback(async (coinId: string, stratumUrl: string) => {
    try {
      await invoke('remove_pool_from_coin', { coinId, stratumUrl })
      setConfirmRemove(null)
      // Clear health results for removed pool
      setHealthResults((prev) => {
        const next = { ...prev }
        delete next[stratumUrl]
        return next
      })
      // Reload coins
      await loadCoins()
    } catch (err) {
      console.error('Failed to remove pool:', err)
    }
  }, [loadCoins])

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
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Pools</h1>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">Test mining pool connections before starting</p>
        </div>
        <button
          onClick={handleRefreshCoins}
          disabled={refreshing}
          className="flex items-center gap-1.5 rounded-lg border border-[var(--border)] px-3 py-1.5 text-xs transition-colors hover:bg-[var(--border)] disabled:opacity-50"
          title="Reload pools from disk"
        >
          <span className={`inline-block ${refreshing ? 'animate-spin' : ''}`}>↻</span>
          {refreshing ? 'Reloading...' : 'Reload'}
        </button>
      </header>

      {coins.filter(c => c.cpu_mineable).map((coin) => (
        <section key={coin.id} className="rounded-xl border border-[var(--border)] bg-surface-elevated p-5">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-base font-medium">{coin.name} ({coin.symbol})</h2>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setDiscoverForCoin({ id: coin.id, name: coin.name, symbol: coin.symbol })}
                className="text-xs px-2.5 py-1 rounded bg-blue-500/10 text-blue-600 dark:text-blue-400 hover:bg-blue-500/20 transition-colors"
                title="Discover pools from miningpoolstats.co.uk"
              >
                🗺 Discover
              </button>
              <button
                onClick={() => setAddPoolForCoin({ id: coin.id, name: coin.name })}
                className="text-xs px-2.5 py-1 rounded bg-green-500/10 text-green-600 dark:text-green-400 hover:bg-green-500/20 transition-colors"
                title="Add new pool"
              >
                + Add Pool
              </button>
              <button
                onClick={() => handleCheckAllForCoin(coin.id)}
                disabled={checkingAll || checking !== null}
                className="text-xs px-2.5 py-1 rounded bg-accent/10 text-accent hover:bg-accent/20 disabled:opacity-50 transition-colors"
              >
                {checkingAll ? 'Checking...' : 'Check All'}
              </button>
            </div>
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
                  onRemove={(url) => setConfirmRemove({ coinId: coin.id, coinName: coin.name, poolName: pool.name, url })}
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
          <div className="flex items-center justify-between mb-2">
            <h2 className="text-base font-medium">GPU/ASIC Only Coins</h2>
          </div>
          <p className="text-xs text-[var(--text-secondary)]">
            {coins.filter(c => !c.cpu_mineable).map(c => c.symbol).join(', ')} - requires dedicated hardware
          </p>
        </section>
      )}

      {/* Add Pool Dialog */}
      {addPoolForCoin && (
        <AddPoolDialog
          coinId={addPoolForCoin.id}
          coinName={addPoolForCoin.name}
          onClose={() => setAddPoolForCoin(null)}
        />
      )}

      {/* Pool Discovery Dialog */}
      {discoverForCoin && (
        <PoolDiscoveryDialog
          coinId={discoverForCoin.id}
          coinName={discoverForCoin.name}
          symbol={discoverForCoin.symbol}
          onClose={() => setDiscoverForCoin(null)}
        />
      )}

      {/* Remove Pool Confirmation Dialog */}
      {confirmRemove && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-sm rounded-xl border border-[var(--border)] bg-[var(--surface)] p-6 shadow-xl">
            <h2 className="text-lg font-semibold mb-2">Remove Pool</h2>
            <p className="text-sm text-[var(--text-secondary)] mb-4">
              Are you sure you want to remove <strong>{confirmRemove.poolName}</strong> from <strong>{confirmRemove.coinName}</strong>?
              <br />
              <span className="font-mono text-xs">{confirmRemove.url}</span>
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setConfirmRemove(null)}
                className="rounded-lg border border-[var(--border)] px-4 py-2 text-sm transition-colors hover:bg-[var(--border)]"
              >
                Cancel
              </button>
              <button
                onClick={() => handleRemovePool(confirmRemove.coinId, confirmRemove.url)}
                className="rounded-lg bg-red-500 px-4 py-2 text-sm text-white transition-colors hover:bg-red-600"
              >
                Remove
              </button>
            </div>
          </div>
        </div>
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
  onRemove: (url: string) => void
  getStatusColor: (status: 'ok' | 'degraded' | 'down' | 'unknown') => string
  getStatusLabel: (status: 'ok' | 'degraded' | 'down' | 'unknown') => string
}

function PoolRow({ name, url, region, tls, health, isChecking, onCheck, onRemove, getStatusColor, getStatusLabel }: PoolRowProps) {
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
        <button
          onClick={() => onRemove(url)}
          className="rounded border border-red-500/30 px-2 py-1 text-xs text-red-500 transition-colors hover:bg-red-500/10"
          title="Remove pool"
        >
          ✕
        </button>
      </div>
    </div>
  )
}
