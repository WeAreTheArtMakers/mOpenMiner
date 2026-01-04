import { useState, useRef, useEffect } from 'react'
import { clsx } from 'clsx'
import { useAppStore } from '@/stores/app'

type LogLevel = 'all' | 'info' | 'warn' | 'error'

export function Logs() {
  const { logs } = useAppStore()
  const [filter, setFilter] = useState<LogLevel>('all')
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, [logs])

  const filteredLogs = logs.filter((log) => {
    if (filter === 'all') return true
    const lower = log.toLowerCase()
    if (filter === 'error') return lower.includes('error') || lower.includes('err')
    if (filter === 'warn') return lower.includes('warn') || lower.includes('warning')
    if (filter === 'info') return lower.includes('info')
    return true
  })

  const getLogColor = (log: string) => {
    const lower = log.toLowerCase()
    if (lower.includes('error') || lower.includes('err')) return 'text-red-500'
    if (lower.includes('warn')) return 'text-yellow-500'
    if (lower.includes('accepted')) return 'text-green-500'
    return 'text-[var(--text-secondary)]'
  }

  return (
    <div className="flex h-full flex-col space-y-4">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Logs</h1>
          <p className="mt-1 text-[var(--text-secondary)]">Real-time miner output</p>
        </div>
        
        <div className="flex gap-1 rounded-lg border border-[var(--border)] p-1">
          {(['all', 'info', 'warn', 'error'] as const).map((level) => (
            <button
              key={level}
              onClick={() => setFilter(level)}
              className={clsx(
                'rounded-md px-3 py-1 text-sm font-medium transition-colors',
                filter === level
                  ? 'bg-accent text-white'
                  : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              )}
            >
              {level.toUpperCase()}
            </button>
          ))}
        </div>
      </header>

      <div
        ref={containerRef}
        className="flex-1 overflow-auto rounded-xl border border-[var(--border)] bg-surface-elevated p-4 font-mono text-xs"
        role="log"
        aria-live="polite"
        aria-label="Mining logs"
      >
        {filteredLogs.length === 0 ? (
          <p className="text-[var(--text-secondary)]">No logs yet. Start mining to see output.</p>
        ) : (
          filteredLogs.map((log, i) => (
            <div key={i} className={clsx('py-0.5', getLogColor(log))}>
              {log}
            </div>
          ))
        )}
      </div>
    </div>
  )
}
