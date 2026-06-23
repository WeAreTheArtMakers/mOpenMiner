import { memo } from 'react'
import { clsx } from 'clsx'
import type { SessionSummary, SessionStatus } from '@/stores/sessions'

interface SessionCardProps {
  session: SessionSummary
  onStop: (id: string) => void
  onSuspend: (id: string) => void
  onResume: (id: string) => void
}

const statusLabels: Record<SessionStatus, string> = {
  stopped: 'STOPPED',
  starting: 'STARTING',
  running: 'RUNNING',
  suspended: 'PAUSED',
  stopping: 'STOPPING',
  error: 'ERROR',
}

const statusColors: Record<SessionStatus, string> = {
  stopped: 'bg-gray-500',
  starting: 'bg-yellow-500',
  running: 'bg-green-500',
  suspended: 'bg-blue-500',
  stopping: 'bg-yellow-500',
  error: 'bg-red-500',
}

function formatHashrate(hs: number): string {
  if (hs >= 1e9) return `${(hs / 1e9).toFixed(2)} GH/s`
  if (hs >= 1e6) return `${(hs / 1e6).toFixed(2)} MH/s`
  if (hs >= 1e3) return `${(hs / 1e3).toFixed(2)} kH/s`
  return `${hs.toFixed(1)} H/s`
}

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}

export const SessionCard = memo(function SessionCard({ 
  session, 
  onStop, 
  onSuspend, 
  onResume 
}: SessionCardProps) {
  const { id, config, stats } = session
  const isActive = stats.status === 'running' || stats.status === 'suspended'
  const isTransitioning = stats.status === 'starting' || stats.status === 'stopping'

  const poolHost = config.pool_url.split('/')[2]?.split(':')[0] || config.pool_url

  return (
    <article
      className={clsx(
        'rounded-xl border p-4 transition-all',
        stats.status === 'running' && 'border-green-500/30 bg-green-500/5',
        stats.status === 'suspended' && 'border-blue-500/30 bg-blue-500/5',
        stats.status === 'error' && 'border-red-500/30 bg-red-500/5',
        stats.status === 'stopped' && 'border-[var(--border)] bg-surface-elevated',
        isTransitioning && 'border-yellow-500/30 bg-yellow-500/5'
      )}
      aria-label={`${config.symbol} mining session`}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="text-lg font-bold">{config.symbol}</span>
          <span className="text-xs px-2 py-0.5 rounded bg-surface text-[var(--text-secondary)]">
            {config.miner_kind}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span className={clsx('w-2 h-2 rounded-full', statusColors[stats.status])} />
          <span className="text-xs font-medium">{statusLabels[stats.status]}</span>
        </div>
      </div>

      {/* Pool & Worker */}
      <div className="text-xs text-[var(--text-secondary)] mb-3 truncate">
        {poolHost} • {config.worker}
      </div>

      {/* Stats Grid */}
      {isActive && (
        <div className="grid grid-cols-3 gap-2 mb-3">
          <div className="text-center">
            <p className="text-xs text-[var(--text-secondary)]">Hashrate</p>
            <p 
              className={clsx(
                'font-mono text-sm font-semibold',
                stats.telemetry_confidence === 'low' && 'text-yellow-500',
                stats.telemetry_confidence === 'unknown' && 'text-[var(--text-secondary)]'
              )}
              title={stats.telemetry_reason || undefined}
            >
              {stats.hashrate_current > 0 ? formatHashrate(stats.hashrate_current) : '—'}
              {stats.telemetry_confidence === 'low' && (
                <span className="ml-1 text-xs" title={stats.telemetry_reason}>⚠</span>
              )}
            </p>
          </div>
          <div className="text-center">
            <p className="text-xs text-[var(--text-secondary)]">Shares</p>
            <p className="font-mono text-sm font-semibold text-green-500">
              {stats.accepted}
              {stats.rejected > 0 && (
                <span className="text-red-500">/{stats.rejected}</span>
              )}
            </p>
          </div>
          <div className="text-center">
            <p className="text-xs text-[var(--text-secondary)]">Uptime</p>
            <p className="font-mono text-sm">{formatUptime(stats.uptime_secs)}</p>
          </div>
        </div>
      )}

      {/* Connection state badge */}
      {isActive && stats.connection_state && stats.connection_state !== 'unknown' && (
        <div className="flex items-center gap-1 mb-2">
          <span className={clsx(
            'text-xs px-1.5 py-0.5 rounded',
            stats.connection_state === 'authorized' && 'bg-green-500/10 text-green-500',
            stats.connection_state === 'connected' && 'bg-blue-500/10 text-blue-500',
            stats.connection_state === 'connecting' && 'bg-yellow-500/10 text-yellow-500'
          )}>
            {stats.connection_state}
          </span>
        </div>
      )}

      {/* Telemetry warning */}
      {isActive && stats.telemetry_confidence === 'low' && stats.hashrate_current === 0 && (
        <p className="text-xs text-yellow-500 mb-2" title={stats.telemetry_reason}>
          ⚠️ {stats.telemetry_reason || 'Hashrate unknown (log parsing limited)'}
        </p>
      )}

      {/* Error message */}
      {stats.last_error && (
        <p className="text-xs text-red-500 mb-2 truncate" title={stats.last_error}>
          {stats.last_error}
        </p>
      )}

      {/* Actions */}
      <div className="flex gap-2">
        {stats.status === 'running' && (
          <>
            <button
              onClick={() => onSuspend(id)}
              className="flex-1 px-3 py-1.5 text-xs font-medium rounded bg-blue-500/10 text-blue-500 hover:bg-blue-500/20 transition-colors"
              aria-label={`Pause ${config.symbol} mining`}
            >
              Pause
            </button>
            <button
              onClick={() => onStop(id)}
              className="flex-1 px-3 py-1.5 text-xs font-medium rounded bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-colors"
              aria-label={`Stop ${config.symbol} mining`}
            >
              Stop
            </button>
          </>
        )}
        {stats.status === 'suspended' && (
          <>
            <button
              onClick={() => onResume(id)}
              className="flex-1 px-3 py-1.5 text-xs font-medium rounded bg-green-500/10 text-green-500 hover:bg-green-500/20 transition-colors"
              aria-label={`Resume ${config.symbol} mining`}
            >
              Resume
            </button>
            <button
              onClick={() => onStop(id)}
              className="flex-1 px-3 py-1.5 text-xs font-medium rounded bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-colors"
              aria-label={`Stop ${config.symbol} mining`}
            >
              Stop
            </button>
          </>
        )}
        {isTransitioning && (
          <span className="flex-1 text-center text-xs text-[var(--text-secondary)] py-1.5">
            Please wait...
          </span>
        )}
      </div>
    </article>
  )
})
