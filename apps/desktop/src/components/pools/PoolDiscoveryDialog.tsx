import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { useAppStore } from '@/stores/app'

interface DiscoveredPool {
  name: string
  url: string
  tls: boolean
}

interface PoolDiscoveryDialogProps {
  coinId: string
  coinName: string
  symbol: string
  onClose: () => void
}

export function PoolDiscoveryDialog({ coinId, coinName, symbol, onClose }: PoolDiscoveryDialogProps) {
  const [pools, setPools] = useState<DiscoveredPool[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [importing, setImporting] = useState<string | null>(null)
  const [importResults, setImportResults] = useState<Record<string, 'success' | 'duplicate' | 'error'>>({})

  const handleDiscover = async () => {
    setLoading(true)
    setError(null)
    setPools([])
    setImportResults({})
    
    try {
      const result = await invoke<DiscoveredPool[]>('discover_pools', { symbol })
      setPools(result)
      if (result.length === 0) {
        setError(`No pools found for ${symbol}. The API may be unavailable.`)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleImportPool = async (pool: DiscoveredPool) => {
    setImporting(pool.url)
    try {
      // Map tls field correctly for the backend
      const poolConfig = {
        name: pool.name,
        stratum_url: pool.url,
        tls: pool.tls,
        region: 'global',
      }

      await invoke('add_pool_to_coin', {
        coinId,
        pool: poolConfig,
      })

      setImportResults((prev) => ({ ...prev, [pool.url]: 'success' }))
      
      // Refresh coins
      const { loadCoins } = useAppStore.getState()
      await loadCoins()
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : String(err)
      if (errMsg.includes('already exists')) {
        setImportResults((prev) => ({ ...prev, [pool.url]: 'duplicate' }))
      } else {
        setImportResults((prev) => ({ ...prev, [pool.url]: 'error' }))
        console.error('Import failed:', errMsg)
      }
    } finally {
      setImporting(null)
    }
  }

  const handleImportAll = async () => {
    for (const pool of pools) {
      if (importResults[pool.url]) continue // Skip already imported
      await handleImportPool(pool)
    }
  }

  // Auto-discover on mount
  useEffect(() => {
    handleDiscover()
  }, [])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-lg max-h-[80vh] rounded-xl border border-[var(--border)] bg-[var(--surface)] p-6 shadow-xl overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-lg font-semibold">Discover Pools</h2>
            <p className="text-xs text-[var(--text-secondary)]">
              {coinName} ({symbol}) - miningpoolstats.co.uk
            </p>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg border border-[var(--border)] px-2 py-1 text-xs hover:bg-[var(--border)]"
          >
            ✕
          </button>
        </div>

        {loading && (
          <div className="flex items-center justify-center py-8">
            <div className="h-6 w-6 animate-spin rounded-full border-2 border-accent border-t-transparent" />
            <span className="ml-2 text-sm text-[var(--text-secondary)]">Discovering pools...</span>
          </div>
        )}

        {error && !loading && (
          <div className="rounded-lg bg-yellow-500/10 border border-yellow-500/30 px-3 py-2 text-xs text-yellow-600 dark:text-yellow-400 mb-4">
            {error}
            <button
              onClick={handleDiscover}
              className="ml-2 underline hover:no-underline"
            >
              Retry
            </button>
          </div>
        )}

        {!loading && pools.length > 0 && (
          <>
            <div className="flex items-center justify-between mb-3">
              <span className="text-xs text-[var(--text-secondary)]">
                {pools.length} pool{pools.length !== 1 ? 's' : ''} found
              </span>
              <button
                onClick={handleImportAll}
                className="text-xs px-2.5 py-1 rounded bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
              >
                Import All
              </button>
            </div>

            <div className="space-y-2">
              {pools.map((pool) => {
                const status = importResults[pool.url]
                return (
                  <div
                    key={pool.url}
                    className="flex items-center justify-between rounded-lg border border-[var(--border)] bg-surface p-3"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium">{pool.name}</span>
                        {pool.tls && (
                          <span className="rounded bg-green-500/10 px-1.5 py-0.5 text-xs text-green-600 dark:text-green-400">TLS</span>
                        )}
                      </div>
                      <p className="mt-0.5 truncate font-mono text-xs text-[var(--text-secondary)]">{pool.url}</p>
                    </div>
                    
                    <div className="ml-3">
                      {status === 'success' && (
                        <span className="text-xs text-green-500">✓ Imported</span>
                      )}
                      {status === 'duplicate' && (
                        <span className="text-xs text-yellow-500">Duplicate</span>
                      )}
                      {status === 'error' && (
                        <span className="text-xs text-red-500">Failed</span>
                      )}
                      {!status && (
                        <button
                          onClick={() => handleImportPool(pool)}
                          disabled={importing === pool.url}
                          className="rounded border border-[var(--border)] px-2 py-1 text-xs transition-colors hover:bg-[var(--border)] disabled:opacity-50"
                        >
                          {importing === pool.url ? '...' : 'Import'}
                        </button>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          </>
        )}

        <div className="flex justify-end mt-4">
          <button
            onClick={onClose}
            className="rounded-lg border border-[var(--border)] px-4 py-2 text-sm transition-colors hover:bg-[var(--border)]"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}