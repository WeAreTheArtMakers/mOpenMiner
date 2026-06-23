import { useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { useAppStore } from '@/stores/app'

interface AddPoolDialogProps {
  coinId: string
  coinName: string
  onClose: () => void
}

export function AddPoolDialog({ coinId, coinName, onClose }: AddPoolDialogProps) {
  const [name, setName] = useState('')
  const [stratumUrl, setStratumUrl] = useState('')
  const [tls, setTls] = useState(false)
  const [region, setRegion] = useState('global')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setSubmitting(true)

    // Basic validation
    if (!name.trim()) {
      setError('Pool name is required')
      setSubmitting(false)
      return
    }
    if (!stratumUrl.trim()) {
      setError('Stratum URL is required')
      setSubmitting(false)
      return
    }
    if (!stratumUrl.startsWith('stratum+')) {
      setError('URL must start with stratum+tcp:// or stratum+ssl://')
      setSubmitting(false)
      return
    }

    try {
      const pool = {
        name: name.trim(),
        stratum_url: stratumUrl.trim(),
        tls,
        region: region.trim() || 'global',
      }

      await invoke('add_pool_to_coin', {
        coinId,
        pool,
      })

      // Reload coins in store to refresh UI
      const { loadCoins } = useAppStore.getState()
      await loadCoins()

      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-xl border border-[var(--border)] bg-[var(--surface)] p-6 shadow-xl">
        <h2 className="text-lg font-semibold mb-1">Add Pool to {coinName}</h2>
        <p className="text-xs text-[var(--text-secondary)] mb-4">
          Coin ID: {coinId}
        </p>

        <form onSubmit={handleSubmit} className="space-y-3">
          <div>
            <label className="block text-xs font-medium mb-1">Pool Name *</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. My Pool"
              className="w-full rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-accent"
            />
          </div>

          <div>
            <label className="block text-xs font-medium mb-1">Stratum URL *</label>
            <input
              type="text"
              value={stratumUrl}
              onChange={(e) => setStratumUrl(e.target.value)}
              placeholder="e.g. stratum+tcp://pool.example.com:3333"
              className="w-full rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 py-2 text-sm font-mono focus:outline-none focus:ring-1 focus:ring-accent"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-medium mb-1">Region</label>
              <input
                type="text"
                value={region}
                onChange={(e) => setRegion(e.target.value)}
                placeholder="global"
                className="w-full rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-accent"
              />
            </div>
            <div className="flex items-end pb-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={tls}
                  onChange={(e) => setTls(e.target.checked)}
                  className="rounded border-[var(--border)] text-accent focus:ring-accent"
                />
                <span className="text-sm">TLS/SSL</span>
              </label>
            </div>
          </div>

          {error && (
            <div className="rounded-lg bg-red-500/10 border border-red-500/30 px-3 py-2 text-xs text-red-600 dark:text-red-400">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              disabled={submitting}
              className="rounded-lg border border-[var(--border)] px-4 py-2 text-sm transition-colors hover:bg-[var(--border)] disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="rounded-lg bg-accent px-4 py-2 text-sm text-white transition-colors hover:bg-accent/90 disabled:opacity-50"
            >
              {submitting ? 'Adding...' : 'Add Pool'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}