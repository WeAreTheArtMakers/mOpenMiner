import { useState } from 'react'
import { clsx } from 'clsx'
import { useAppStore, type Profile, type PerformancePreset } from '@/stores/app'

export function Profiles() {
  const { profiles, coins, saveProfile, deleteProfile, startMining, status, setPage } = useAppStore()
  const [showForm, setShowForm] = useState(false)
  const [editingProfile, setEditingProfile] = useState<Profile | null>(null)
  
  // Form state
  const [name, setName] = useState('')
  const [selectedCoin, setSelectedCoin] = useState('')
  const [selectedPool, setSelectedPool] = useState('')
  const [wallet, setWallet] = useState('')
  const [worker, setWorker] = useState('')
  const [preset, setPreset] = useState<PerformancePreset>('balanced')

  const selectedCoinData = coins.find(c => c.id === selectedCoin)

  const resetForm = () => {
    setName('')
    setSelectedCoin('')
    setSelectedPool('')
    setWallet('')
    setWorker('')
    setPreset('balanced')
    setEditingProfile(null)
    setShowForm(false)
  }

  const handleSave = async () => {
    if (!name || !selectedCoin || !selectedPool || !wallet) return
    
    await saveProfile({
      name,
      coin: selectedCoin,
      pool: selectedPool,
      wallet,
      worker: worker || 'worker',
      threads: 0, // Auto
      preset,
    })
    resetForm()
  }

  const handleLoad = (profile: Profile) => {
    if (status.isRunning) {
      alert('Stop mining first before loading a profile')
      return
    }
    
    const coinData = coins.find(c => c.id === profile.coin)
    const algorithm = coinData?.algorithm || ''
    
    startMining({
      coin: profile.coin,
      pool: profile.pool,
      wallet: profile.wallet,
      worker: profile.worker,
      threads: profile.threads,
      preset: profile.preset,
      algorithm,
      tryAnyway: false,
    })
    
    setPage('dashboard')
  }

  const handleEdit = (profile: Profile) => {
    setEditingProfile(profile)
    setName(profile.name)
    setSelectedCoin(profile.coin)
    setSelectedPool(profile.pool)
    setWallet(profile.wallet)
    setWorker(profile.worker)
    setPreset(profile.preset)
    setShowForm(true)
  }

  const handleDelete = async (profile: Profile) => {
    if (confirm(`Delete profile "${profile.name}"?`)) {
      await deleteProfile(profile.id)
    }
  }

  const presetLabels: Record<PerformancePreset, string> = {
    eco: 'Eco (~25%)',
    balanced: 'Balanced (~50%)',
    max: 'Max (~75%)',
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Profiles</h1>
          <p className="mt-1 text-[var(--text-secondary)]">Save and load mining configurations</p>
        </div>
        {!showForm && (
          <button
            onClick={() => setShowForm(true)}
            className="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover"
          >
            + New Profile
          </button>
        )}
      </header>

      {/* New/Edit Profile Form */}
      {showForm && (
        <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
          <h2 className="text-lg font-medium mb-4">
            {editingProfile ? 'Edit Profile' : 'Create New Profile'}
          </h2>
          
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="sm:col-span-2">
              <label className="block text-sm font-medium mb-1">Profile Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My XMR Config"
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Coin</label>
              <select
                value={selectedCoin}
                onChange={(e) => { setSelectedCoin(e.target.value); setSelectedPool('') }}
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm"
              >
                <option value="">Select coin...</option>
                {coins.map(c => (
                  <option key={c.id} value={c.id}>{c.name} ({c.symbol})</option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Pool</label>
              <select
                value={selectedPool}
                onChange={(e) => setSelectedPool(e.target.value)}
                disabled={!selectedCoin}
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm disabled:opacity-50"
              >
                <option value="">Select pool...</option>
                {selectedCoinData?.default_pools.map(p => (
                  <option key={p.stratum_url} value={p.stratum_url}>{p.name} ({p.region})</option>
                ))}
              </select>
            </div>

            <div className="sm:col-span-2">
              <label className="block text-sm font-medium mb-1">Wallet Address</label>
              <input
                type="text"
                value={wallet}
                onChange={(e) => setWallet(e.target.value)}
                placeholder="Your wallet address"
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 font-mono text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Worker Name</label>
              <input
                type="text"
                value={worker}
                onChange={(e) => setWorker(e.target.value)}
                placeholder="my-mac"
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">Performance</label>
              <select
                value={preset}
                onChange={(e) => setPreset(e.target.value as PerformancePreset)}
                className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm"
              >
                <option value="eco">Eco (~25% CPU)</option>
                <option value="balanced">Balanced (~50% CPU)</option>
                <option value="max">Max (~75% CPU)</option>
              </select>
            </div>
          </div>

          <div className="flex gap-3 mt-6">
            <button
              onClick={handleSave}
              disabled={!name || !selectedCoin || !selectedPool || !wallet}
              className="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Save Profile
            </button>
            <button
              onClick={resetForm}
              className="rounded-lg bg-surface px-4 py-2 text-sm font-medium hover:bg-[var(--border)]"
            >
              Cancel
            </button>
          </div>
        </section>
      )}

      {/* Saved Profiles */}
      {profiles.length === 0 && !showForm ? (
        <div className="rounded-xl border border-dashed border-[var(--border)] p-12 text-center">
          <div className="text-4xl mb-4">📋</div>
          <p className="text-[var(--text-secondary)]">No profiles saved yet.</p>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">
            Create a profile to quickly start mining with saved settings.
          </p>
          <button
            onClick={() => setShowForm(true)}
            className="mt-4 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent-hover"
          >
            Create First Profile
          </button>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2">
          {profiles.map((profile) => {
            const coinData = coins.find(c => c.id === profile.coin)
            return (
              <div
                key={profile.id}
                className="rounded-xl border border-[var(--border)] bg-surface-elevated p-4 hover:border-accent/50 transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div>
                    <h3 className="font-medium">{profile.name}</h3>
                    <p className="mt-1 text-sm text-[var(--text-secondary)]">
                      {coinData?.symbol || profile.coin.toUpperCase()} • {presetLabels[profile.preset]}
                    </p>
                  </div>
                  <span className="text-xs px-2 py-1 rounded bg-surface text-[var(--text-secondary)]">
                    {coinData?.algorithm || 'unknown'}
                  </span>
                </div>
                
                <div className="mt-3 space-y-1">
                  <p className="text-xs text-[var(--text-secondary)]">
                    Pool: {profile.pool.split('/')[2]?.split(':')[0] || profile.pool}
                  </p>
                  <p className="truncate font-mono text-xs text-[var(--text-secondary)]">
                    Wallet: {profile.wallet.slice(0, 12)}...{profile.wallet.slice(-8)}
                  </p>
                </div>

                <div className="flex gap-2 mt-4">
                  <button
                    onClick={() => handleLoad(profile)}
                    disabled={status.isRunning}
                    className={clsx(
                      "flex-1 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                      status.isRunning
                        ? "bg-surface text-[var(--text-secondary)] cursor-not-allowed"
                        : "bg-green-500/10 text-green-500 hover:bg-green-500/20"
                    )}
                  >
                    ▶ Start Mining
                  </button>
                  <button
                    onClick={() => handleEdit(profile)}
                    className="rounded-lg bg-surface px-3 py-2 text-sm hover:bg-[var(--border)]"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDelete(profile)}
                    className="rounded-lg bg-red-500/10 px-3 py-2 text-sm text-red-500 hover:bg-red-500/20"
                    title="Delete profile"
                  >
                    🗑
                  </button>
                </div>
              </div>
            )
          })}
        </div>
      )}

      {/* Info Box */}
      <section className="rounded-xl border border-blue-500/30 bg-blue-500/5 p-4">
        <h3 className="font-medium text-blue-500 mb-2">💡 How Mining Payments Work</h3>
        <div className="text-sm text-[var(--text-secondary)] space-y-2">
          <p>
            When you mine, your shares are sent to the pool. The pool collects shares from all miners
            and distributes rewards based on contribution.
          </p>
          <p>
            <strong>Payments are made by the pool</strong>, not this app. Most pools have a minimum
            payout threshold (e.g., 0.01 XMR). Once reached, the pool automatically sends coins to your wallet.
          </p>
          <p>
            Check your pool's website to see your balance and payment history.
          </p>
        </div>
      </section>
    </div>
  )
}
