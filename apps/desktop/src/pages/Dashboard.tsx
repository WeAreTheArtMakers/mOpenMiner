import { useState, useEffect, useCallback, useMemo } from 'react'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/tauri'
import { useAppStore, type PerformancePreset, type MinerState, type HashratePoint } from '@/stores/app'
import { useSessionsStore, useActiveSessions, useTotalHashrate } from '@/stores/sessions'
import { SessionCard } from '@/components/SessionCard'

interface BudgetStatus {
  effective_cores: number
  budget_threads: number
  total_requested: number
  is_overcommitted: boolean
  overcommit_ratio: number
  suggested_per_session: number
}

const stateLabels: Record<MinerState, string> = {
  stopped: 'STOPPED',
  starting: 'STARTING...',
  running: 'RUNNING',
  stopping: 'STOPPING...',
  error: 'ERROR',
}

export function Dashboard() {
  const { status, coins, hasConsent, startMining, stopMining, refreshStatus, logs, hashrateHistory } = useAppStore()
  const { 
    hydrate: hydrateSessions, 
    stopSession, 
    suspendSession, 
    resumeSession, 
    stopAll,
    refreshStats,
    setupEventListeners,
  } = useSessionsStore()
  
  const activeSessions = useActiveSessions()
  const totalHashrate = useTotalHashrate()
  const hasActiveSessions = activeSessions.length > 0
  
  const [selectedCoin, setSelectedCoin] = useState('')
  const [selectedPool, setSelectedPool] = useState('')
  const [wallet, setWallet] = useState('')
  const [worker, setWorker] = useState('')
  const [threads] = useState(0)
  const [preset, setPreset] = useState<PerformancePreset>('balanced')
  const [reconnectCountdown] = useState<number | null>(null)
  const [budgetStatus, setBudgetStatus] = useState<BudgetStatus | null>(null)
  const [lastAcceptedShares, setLastAcceptedShares] = useState(0)

  const selectedCoinData = coins.find((c) => c.id === selectedCoin)
  const isExternalMiner = selectedCoinData?.recommended_miner === 'external-asic' || selectedCoinData?.recommended_miner === 'external-gpu'
  const isCpuMineable = selectedCoinData?.cpu_mineable === true
  const isTransitioning = status.state === 'starting' || status.state === 'stopping'
  // Allow starting for all coins, but show warning for non-CPU coins
  const canStart = Boolean(selectedCoin && selectedPool && wallet && !isTransitioning && status.state === 'stopped')

  // Hydrate sessions on mount and setup event listeners
  useEffect(() => {
    hydrateSessions()
    let unlisteners: (() => void)[] = []
    setupEventListeners().then((fns) => {
      unlisteners = fns
    })
    return () => {
      unlisteners.forEach((fn) => fn())
    }
  }, [hydrateSessions, setupEventListeners])

  // Refresh budget status when sessions change
  useEffect(() => {
    invoke<BudgetStatus>('get_budget_status').then(setBudgetStatus).catch(console.error)
  }, [activeSessions.length])

  // Play sound when share is accepted
  useEffect(() => {
    if (status.acceptedShares > lastAcceptedShares && lastAcceptedShares > 0) {
      // New share accepted - play subtle sound
      invoke('play_notification_sound', { sound: 'success' }).catch(() => {})
    }
    setLastAcceptedShares(status.acceptedShares)
  }, [status.acceptedShares, lastAcceptedShares])

  // Auto-refresh stats when running (legacy + sessions)
  useEffect(() => {
    if (status.isRunning || hasActiveSessions) {
      const interval = setInterval(() => {
        refreshStatus()
        refreshStats()
      }, 2000)
      return () => clearInterval(interval)
    }
  }, [status.isRunning, hasActiveSessions, refreshStatus, refreshStats])

  // Last 10 logs for quick view
  const recentLogs = useMemo(() => logs.slice(-10), [logs])

  const handleStart = useCallback(() => {
    if (!canStart) return
    const algorithm = selectedCoinData?.algorithm || ''
    const tryAnyway = isExternalMiner || !isCpuMineable
    startMining({ 
      coin: selectedCoin, 
      pool: selectedPool, 
      wallet, 
      worker, 
      threads, 
      preset,
      algorithm,
      tryAnyway,
    })
  }, [canStart, selectedCoin, selectedPool, wallet, worker, threads, preset, startMining, selectedCoinData, isExternalMiner, isCpuMineable])

  const formatUptime = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = seconds % 60
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }

  if (!hasConsent) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-[var(--text-secondary)]">Please accept the consent dialog to continue.</p>
      </div>
    )
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      {/* Global Stop Button - Always visible when running */}
      {(status.isRunning || hasActiveSessions) && (
        <div className="flex items-center justify-between p-3 rounded-lg bg-surface-elevated border border-[var(--border)]">
          <div className="flex items-center gap-3">
            <span className="relative flex h-2.5 w-2.5">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
              <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-green-500" />
            </span>
            <span className="text-sm font-medium">
              {status.coin?.toUpperCase() || 'Mining'} → {status.pool?.split('/')[2]?.split(':')[0] || 'pool'}
            </span>
            {status.activeMiner && (
              <span className="text-xs px-1.5 py-0.5 rounded bg-surface text-[var(--text-secondary)]">
                {status.activeMiner}
              </span>
            )}
            {/* Connection status badge */}
            {status.acceptedShares > 0 ? (
              <span className="text-xs px-1.5 py-0.5 rounded bg-green-500/10 text-green-500">
                Connected
              </span>
            ) : status.uptime > 5 ? (
              <span className="text-xs px-1.5 py-0.5 rounded bg-yellow-500/10 text-yellow-500">
                Connecting...
              </span>
            ) : null}
          </div>
          <button
            onClick={() => {
              stopMining()
              stopAll()
            }}
            className="flex items-center gap-1.5 rounded-md bg-red-500/10 px-3 py-1.5 text-sm font-medium text-red-500 hover:bg-red-500/20 transition-colors"
            aria-label="Stop all mining"
          >
            <StopIcon className="h-3.5 w-3.5" />
            Stop
          </button>
        </div>
      )}

      {/* Active Sessions Summary */}
      {hasActiveSessions && (
        <div className="rounded-xl border border-green-500/30 bg-green-500/5 p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <span className="relative flex h-3 w-3">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
                <span className="relative inline-flex h-3 w-3 rounded-full bg-green-500" />
              </span>
              <span className="font-semibold">
                {activeSessions.length} Active Session{activeSessions.length > 1 ? 's' : ''}
              </span>
            </div>
            <span className="font-mono text-sm text-[var(--text-secondary)]">
              Total: {totalHashrate > 0 ? `${totalHashrate.toFixed(1)} H/s` : '—'}
            </span>
          </div>
        </div>
      )}

      {/* CPU Overcommit Warning */}
      {budgetStatus?.is_overcommitted && (
        <div className="rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-3 text-sm">
          <div className="flex items-center gap-2 text-yellow-600 dark:text-yellow-400">
            <svg className="h-4 w-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
            </svg>
            <span className="font-medium">CPU Overcommitted</span>
          </div>
          <p className="mt-1 text-xs text-[var(--text-secondary)]">
            Using {budgetStatus.total_requested} threads ({Math.round(budgetStatus.overcommit_ratio * 100)}% of budget).
            Consider reducing threads or stopping a session.
          </p>
        </div>
      )}

      {/* Session Cards */}
      {activeSessions.length > 0 && (
        <section aria-label="Active mining sessions">
          <h2 className="text-sm font-medium text-[var(--text-secondary)] mb-3">Running Sessions</h2>
          <div className="grid gap-4 sm:grid-cols-2">
            {activeSessions.map((session) => (
              <SessionCard
                key={session.id}
                session={session}
                onStop={stopSession}
                onSuspend={suspendSession}
                onResume={resumeSession}
              />
            ))}
          </div>
        </section>
      )}

      {/* Legacy Status Banner (for backward compatibility) */}
      {!hasActiveSessions && (
        <div className={clsx(
          'flex items-center justify-between rounded-xl p-4',
          status.state === 'running' && 'bg-green-500/10 border border-green-500/30',
          status.state === 'stopped' && 'bg-surface-elevated border border-[var(--border)]',
          status.state === 'error' && 'bg-red-500/10 border border-red-500/30',
          isTransitioning && 'bg-yellow-500/10 border border-yellow-500/30'
        )}>
          <div className="flex items-center gap-3">
            <StatusIndicator state={status.state} />
            <div>
              <span className="text-lg font-semibold">{stateLabels[status.state]}</span>
              {status.isRunning && status.activeMiner && (
                <span className="ml-2 text-xs text-[var(--text-secondary)] bg-surface px-2 py-0.5 rounded">
                  {status.activeMiner}
                </span>
              )}
            </div>
          </div>
          {status.isRunning && status.coin && (
            <span className="text-sm text-[var(--text-secondary)]">
              {status.coin.toUpperCase()} → {status.pool?.split('/')[2]?.split(':')[0] || 'pool'}
            </span>
          )}
        </div>
      )}

      {/* Warning Banner for non-practical mining */}
      {status.isRunning && status.warning && (
        <div className="rounded-lg bg-yellow-500/10 border border-yellow-500/30 p-3 text-sm text-yellow-600 dark:text-yellow-400">
          ⚠️ {status.warning}
        </div>
      )}

      {/* Error Banner */}
      {status.state === 'error' && status.warning && (
        <div className="rounded-lg bg-red-500/10 border border-red-500/30 p-4 text-sm text-red-600 dark:text-red-400">
          <div className="font-medium mb-1">Failed to start mining</div>
          <div className="text-xs opacity-90">{status.warning}</div>
          {status.warning.includes('cpuminer-opt') && (
            <div className="mt-2 text-xs opacity-75">
              See <span className="font-mono">docs/MINERS.md</span> for installation instructions.
            </div>
          )}
        </div>
      )}

      {/* KPI Cards - 3 main metrics */}
      {status.isRunning && (
        <div className="grid grid-cols-3 gap-4">
          <KPICard label="Hashrate" value={`${status.hashrate.toFixed(1)} H/s`} />
          <KPICard label="Accepted" value={status.acceptedShares.toString()} accent="green" />
          <KPICard label="Rejected" value={status.rejectedShares.toString()} accent={status.rejectedShares > 0 ? "red" : undefined} />
        </div>
      )}

      {/* Secondary stats row */}
      {status.isRunning && (
        <div className="grid grid-cols-3 gap-4">
          <KPICard label="Avg Hashrate" value={status.avgHashrate > 0 ? `${status.avgHashrate.toFixed(1)} H/s` : '—'} small />
          <KPICard label="Uptime" value={formatUptime(status.uptime)} small />
          <KPICard label="Efficiency" value={status.acceptedShares > 0 ? `${((status.acceptedShares / (status.acceptedShares + status.rejectedShares)) * 100).toFixed(1)}%` : '—'} small />
        </div>
      )}

      {/* Hashrate Chart */}
      {status.isRunning && hashrateHistory.length > 1 && (
        <HashrateChart data={hashrateHistory} />
      )}

      {/* Reconnect countdown */}
      {reconnectCountdown !== null && (
        <div className="rounded-lg bg-yellow-500/10 p-3 text-center text-sm text-yellow-600 dark:text-yellow-400">
          Reconnecting in {reconnectCountdown}s...
        </div>
      )}

      {/* Configuration */}
      <ConfigSection
        coins={coins}
        selectedCoin={selectedCoin}
        setSelectedCoin={setSelectedCoin}
        selectedPool={selectedPool}
        setSelectedPool={setSelectedPool}
        wallet={wallet}
        setWallet={setWallet}
        worker={worker}
        setWorker={setWorker}
        preset={preset}
        setPreset={setPreset}
        isRunning={status.isRunning}
        isTransitioning={isTransitioning}
        isExternalMiner={isExternalMiner}
        selectedCoinData={selectedCoinData}
        canStart={canStart}
        onStart={handleStart}
      />

      {/* Recent Logs */}
      {status.isRunning && recentLogs.length > 0 && (
        <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-4">
          <div className="mb-2 flex items-center justify-between">
            <h3 className="text-sm font-medium text-[var(--text-secondary)]">Recent Activity</h3>
            <button 
              onClick={() => useAppStore.getState().setPage('logs')}
              className="text-xs text-accent hover:underline"
            >
              Open Logs →
            </button>
          </div>
          <div className="space-y-1 font-mono text-xs">
            {recentLogs.map((log, i) => (
              <div key={i} className={clsx(
                'truncate',
                log.toLowerCase().includes('error') && 'text-red-500',
                log.toLowerCase().includes('warn') && 'text-yellow-500',
                log.toLowerCase().includes('accepted') && 'text-green-500',
                !log.toLowerCase().includes('error') && !log.toLowerCase().includes('warn') && !log.toLowerCase().includes('accepted') && 'text-[var(--text-secondary)]'
              )}>
                {log}
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  )
}

function StatusIndicator({ state }: { state: MinerState }) {
  if (state === 'running') {
    return (
      <span className="relative flex h-3 w-3">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
        <span className="relative inline-flex h-3 w-3 rounded-full bg-green-500" />
      </span>
    )
  }
  if (state === 'starting' || state === 'stopping') {
    return <span className="h-3 w-3 rounded-full bg-yellow-500 animate-pulse" />
  }
  if (state === 'error') {
    return <span className="h-3 w-3 rounded-full bg-red-500" />
  }
  return <span className="h-3 w-3 rounded-full bg-gray-400" />
}

function KPICard({ label, value, accent, small }: { label: string; value: string; accent?: 'green' | 'red'; small?: boolean }) {
  return (
    <div className={clsx(
      'rounded-xl border border-[var(--border)] bg-surface-elevated text-center',
      small ? 'p-3' : 'p-4'
    )}>
      <p className={clsx(
        'font-medium uppercase tracking-wider text-[var(--text-secondary)]',
        small ? 'text-[10px]' : 'text-xs'
      )}>{label}</p>
      <p className={clsx(
        'mt-1 font-mono font-bold',
        small ? 'text-lg' : 'text-2xl',
        accent === 'green' && 'text-green-500',
        accent === 'red' && 'text-red-500'
      )}>
        {value}
      </p>
    </div>
  )
}

function StopIcon({ className }: { className?: string }) {
  return (
    <svg className={className || "h-5 w-5"} fill="currentColor" viewBox="0 0 24 24">
      <rect x="6" y="6" width="12" height="12" rx="1" />
    </svg>
  )
}


interface ConfigSectionProps {
  coins: any[]
  selectedCoin: string
  setSelectedCoin: (v: string) => void
  selectedPool: string
  setSelectedPool: (v: string) => void
  wallet: string
  setWallet: (v: string) => void
  worker: string
  setWorker: (v: string) => void
  preset: PerformancePreset
  setPreset: (v: PerformancePreset) => void
  isRunning: boolean
  isTransitioning: boolean
  isExternalMiner: boolean
  selectedCoinData: any
  canStart: boolean
  onStart: () => void
}

function ConfigSection(props: ConfigSectionProps) {
  const {
    coins, selectedCoin, setSelectedCoin, selectedPool, setSelectedPool,
    wallet, setWallet, worker, setWorker, preset, setPreset,
    isRunning, isTransitioning, isExternalMiner, selectedCoinData, canStart, onStart
  } = props

  const presetDescriptions: Record<PerformancePreset, string> = {
    eco: 'Low power (~25% CPU)',
    balanced: 'Moderate (~50% CPU)',
    max: 'Maximum (~75% CPU)',
  }

  return (
    <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
      <h2 className="mb-4 text-lg font-medium">Configuration</h2>
      
      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label htmlFor="coin" className="mb-1 block text-sm font-medium">Coin</label>
          <select
            id="coin"
            value={selectedCoin}
            onChange={(e) => { setSelectedCoin(e.target.value); setSelectedPool('') }}
            disabled={isRunning || isTransitioning}
            className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm disabled:opacity-50"
          >
            <option value="">Select...</option>
            {coins.map((c) => <option key={c.id} value={c.id}>{c.name} ({c.symbol})</option>)}
          </select>
        </div>

        <div>
          <label htmlFor="pool" className="mb-1 block text-sm font-medium">Pool</label>
          <select
            id="pool"
            value={selectedPool}
            onChange={(e) => setSelectedPool(e.target.value)}
            disabled={isRunning || isTransitioning || !selectedCoin}
            className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm disabled:opacity-50"
          >
            <option value="">Select...</option>
            {selectedCoinData?.default_pools.map((p: any) => (
              <option key={p.stratum_url} value={p.stratum_url}>{p.name} ({p.region})</option>
            ))}
          </select>
        </div>

        <div className="sm:col-span-2">
          <label htmlFor="wallet" className="mb-1 block text-sm font-medium">Wallet Address</label>
          <input
            id="wallet"
            type="text"
            value={wallet}
            onChange={(e) => setWallet(e.target.value)}
            disabled={isRunning || isTransitioning}
            placeholder="Your wallet address"
            className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 font-mono text-sm disabled:opacity-50"
          />
        </div>

        <div>
          <label htmlFor="worker" className="mb-1 block text-sm font-medium">Worker Name</label>
          <input
            id="worker"
            type="text"
            value={worker}
            onChange={(e) => setWorker(e.target.value)}
            disabled={isRunning || isTransitioning}
            placeholder="my-mac"
            className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm disabled:opacity-50"
          />
        </div>

        {!isExternalMiner && (
          <div>
            <label htmlFor="preset" className="mb-1 block text-sm font-medium">Performance</label>
            <select
              id="preset"
              value={preset}
              onChange={(e) => setPreset(e.target.value as PerformancePreset)}
              disabled={isRunning || isTransitioning}
              className="w-full rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm disabled:opacity-50"
            >
              <option value="eco">Eco</option>
              <option value="balanced">Balanced</option>
              <option value="max">Max</option>
            </select>
            <p className="mt-1 text-xs text-[var(--text-secondary)]">{presetDescriptions[preset]}</p>
          </div>
        )}
      </div>

      {selectedCoinData?.notes && (
        <div className="mt-4 rounded-lg bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
          {selectedCoinData.notes}
        </div>
      )}

      {isExternalMiner && selectedCoinData && (
        <ExternalMinerGuide coin={selectedCoinData} pool={selectedPool} worker={worker} wallet={wallet} />
      )}

      {!isRunning && (
        <button
          onClick={onStart}
          disabled={!canStart}
          className={clsx(
            "mt-4 w-full rounded-lg py-3 text-lg font-semibold text-white disabled:cursor-not-allowed disabled:opacity-50",
            isExternalMiner 
              ? "bg-yellow-600 hover:bg-yellow-700" 
              : "bg-accent hover:bg-accent-hover"
          )}
        >
          {isTransitioning ? 'Please wait...' : isExternalMiner ? '⚠️ Try Mining Anyway' : 'Start Mining'}
        </button>
      )}

      {isExternalMiner && !isRunning && (
        <p className="mt-2 text-center text-xs text-yellow-600 dark:text-yellow-400">
          Warning: {selectedCoinData?.symbol} is not optimized for CPU mining. Very low hashrate expected.
        </p>
      )}
    </section>
  )
}

function ExternalMinerGuide({ coin, pool, worker, wallet }: { coin: any; pool: string; worker: string; wallet: string }) {
  const copyConfig = () => {
    const config = `Pool: ${pool || '(select above)'}
Wallet: ${wallet || '(enter above)'}
Worker: ${worker || 'worker'}
Algorithm: ${coin.algorithm}`
    navigator.clipboard.writeText(config)
  }

  const isGpu = coin.recommended_miner === 'external-gpu'
  const hardwareType = isGpu ? 'GPU' : 'ASIC'
  const minerSuggestions: Record<string, string> = {
    'kawpow': 'T-Rex, NBMiner, or TeamRedMiner',
    'etchash': 'T-Rex, lolMiner, or TeamRedMiner',
    'kheavyhash': 'lolMiner or BzMiner',
    'autolykos2': 'lolMiner or Nanominer',
    'equihash': 'EWBF, lolMiner, or miniZ',
    'zelHash': 'lolMiner or miniZ',
    'scrypt': 'Antminer L7 or similar ASIC',
    'sha256': 'Antminer S19 or similar ASIC',
  }

  return (
    <div className="mt-4 rounded-lg border border-blue-500/30 bg-blue-500/5 p-4">
      <h3 className="font-medium text-blue-600 dark:text-blue-400">
        External {hardwareType} Miner Mode ({coin.symbol})
      </h3>
      <p className="mt-2 text-sm text-[var(--text-secondary)]">
        {coin.symbol} uses <strong>{coin.algorithm}</strong> algorithm and requires {hardwareType} hardware.
        This app cannot mine {coin.symbol} directly.
      </p>
      
      <div className="mt-3 rounded bg-surface p-3">
        <p className="text-xs font-medium text-[var(--text-secondary)] mb-2">Configure your external miner with:</p>
        <div className="font-mono text-xs space-y-1">
          <p><span className="text-[var(--text-secondary)]">Pool:</span> {pool || '(select above)'}</p>
          <p><span className="text-[var(--text-secondary)]">Worker:</span> {worker || 'worker'}</p>
          <p><span className="text-[var(--text-secondary)]">Algorithm:</span> {coin.algorithm}</p>
        </div>
      </div>

      {minerSuggestions[coin.algorithm] && (
        <p className="mt-3 text-xs text-[var(--text-secondary)]">
          <strong>Suggested miners:</strong> {minerSuggestions[coin.algorithm]}
        </p>
      )}

      <button 
        onClick={copyConfig} 
        className="mt-3 rounded bg-blue-500/10 px-3 py-1.5 text-xs font-medium text-blue-600 hover:bg-blue-500/20 dark:text-blue-400"
      >
        Copy Pool Config
      </button>
    </div>
  )
}

function HashrateChart({ data }: { data: HashratePoint[] }) {
  if (data.length < 2) return null
  
  const maxHashrate = Math.max(...data.map(d => d.hashrate))
  const minHashrate = Math.min(...data.map(d => d.hashrate))
  const range = maxHashrate - minHashrate || 1
  
  // SVG dimensions
  const width = 100
  const height = 40
  const padding = 2
  
  // Generate path
  const points = data.map((d, i) => {
    const x = padding + (i / (data.length - 1)) * (width - 2 * padding)
    const y = height - padding - ((d.hashrate - minHashrate) / range) * (height - 2 * padding)
    return `${x},${y}`
  })
  
  const pathD = `M ${points.join(' L ')}`
  
  // Time range
  const startTime = new Date(data[0].timestamp)
  const endTime = new Date(data[data.length - 1].timestamp)
  const durationMins = Math.round((endTime.getTime() - startTime.getTime()) / 60000)
  
  return (
    <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-4">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-medium text-[var(--text-secondary)]">Hashrate History</h3>
        <span className="text-xs text-[var(--text-secondary)]">Last {durationMins} min</span>
      </div>
      
      <div className="relative">
        <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-20" preserveAspectRatio="none">
          {/* Grid lines */}
          <line x1={padding} y1={height/2} x2={width-padding} y2={height/2} stroke="var(--border)" strokeWidth="0.5" strokeDasharray="2,2" />
          
          {/* Area fill */}
          <path
            d={`${pathD} L ${width - padding},${height - padding} L ${padding},${height - padding} Z`}
            fill="url(#hashrate-gradient)"
            opacity="0.3"
          />
          
          {/* Line */}
          <path
            d={pathD}
            fill="none"
            stroke="#22c55e"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
          
          {/* Gradient definition */}
          <defs>
            <linearGradient id="hashrate-gradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#22c55e" />
              <stop offset="100%" stopColor="#22c55e" stopOpacity="0" />
            </linearGradient>
          </defs>
        </svg>
        
        {/* Labels */}
        <div className="flex justify-between text-xs text-[var(--text-secondary)] mt-1">
          <span>{minHashrate.toFixed(0)} H/s</span>
          <span>{maxHashrate.toFixed(0)} H/s</span>
        </div>
      </div>
    </section>
  )
}
