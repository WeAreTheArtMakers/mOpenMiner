import { useState, useRef, useEffect, useCallback } from 'react'
import { clsx } from 'clsx'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/stores/app'
import { useSessionsStore, useActiveSessions, type LogEntry } from '@/stores/sessions'

type LogLevel = 'all' | 'info' | 'warn' | 'error'

export function Logs() {
  const { logs: legacyLogs } = useAppStore()
  const { getSessionLogs } = useSessionsStore()
  const activeSessions = useActiveSessions()
  
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const [sessionLogs, setSessionLogs] = useState<LogEntry[]>([])
  const [filter, setFilter] = useState<LogLevel>('all')
  const [autoScroll, setAutoScroll] = useState(true)
  const [isLoading, setIsLoading] = useState(false)
  
  const containerRef = useRef<HTMLDivElement>(null)
  const isUserScrolling = useRef(false)

  // Load initial logs when session changes
  useEffect(() => {
    if (!selectedSessionId) {
      setSessionLogs([])
      return
    }

    setIsLoading(true)
    getSessionLogs(selectedSessionId, undefined, 200).then((response) => {
      if (response) {
        setSessionLogs(response.lines)
      }
      setIsLoading(false)
    })
  }, [selectedSessionId, getSessionLogs])

  // Listen for live log events
  useEffect(() => {
    if (!selectedSessionId) return

    const unlisten = listen<{ session_id: string; line: string }>('session://log', (event) => {
      if (event.payload.session_id === selectedSessionId) {
        setSessionLogs((prev) => {
          const newLogs = [...prev, { 
            timestamp: Date.now(), 
            line: event.payload.line 
          }]
          // Keep last 500 lines
          return newLogs.slice(-500)
        })
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [selectedSessionId])

  // Auto-scroll behavior
  useEffect(() => {
    if (autoScroll && containerRef.current && !isUserScrolling.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, [sessionLogs, legacyLogs, autoScroll])

  // Detect user scroll
  const handleScroll = useCallback(() => {
    if (!containerRef.current) return
    
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50
    
    if (!isAtBottom) {
      isUserScrolling.current = true
      setAutoScroll(false)
    } else {
      isUserScrolling.current = false
    }
  }, [])

  const jumpToLatest = useCallback(() => {
    setAutoScroll(true)
    isUserScrolling.current = false
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, [])

  // Get logs to display (session or legacy)
  const displayLogs = selectedSessionId 
    ? sessionLogs.map((l) => l.line)
    : legacyLogs

  const filteredLogs = displayLogs.filter((log) => {
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
    if (lower.includes('accepted') || lower.includes('yay')) return 'text-green-500'
    if (lower.includes('hashrate') || lower.includes('h/s')) return 'text-blue-400'
    return 'text-[var(--text-secondary)]'
  }

  return (
    <div className="flex h-full flex-col space-y-4">
      <header className="flex items-center justify-between gap-4">
        <div className="flex-1">
          <h1 className="text-2xl font-semibold">Logs</h1>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">
            {selectedSessionId 
              ? `Session: ${activeSessions.find(s => s.id === selectedSessionId)?.config.symbol || 'Unknown'}`
              : 'All miner output'
            }
          </p>
        </div>

        {/* Session Picker */}
        {activeSessions.length > 0 && (
          <select
            value={selectedSessionId || ''}
            onChange={(e) => setSelectedSessionId(e.target.value || null)}
            className="rounded-lg border border-[var(--border)] bg-surface px-3 py-2 text-sm"
            aria-label="Select session"
          >
            <option value="">All Sessions</option>
            {activeSessions.map((session) => (
              <option key={session.id} value={session.id}>
                {session.config.symbol} ({session.stats.status})
              </option>
            ))}
          </select>
        )}
        
        {/* Filter */}
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

      {/* Log Container */}
      <div className="relative flex-1">
        <div
          ref={containerRef}
          onScroll={handleScroll}
          className="absolute inset-0 overflow-auto rounded-xl border border-[var(--border)] bg-surface-elevated p-4 font-mono text-xs"
          role="log"
          aria-live="polite"
          aria-label="Mining logs"
        >
          {isLoading ? (
            <p className="text-[var(--text-secondary)]">Loading logs...</p>
          ) : filteredLogs.length === 0 ? (
            <p className="text-[var(--text-secondary)]">
              {selectedSessionId 
                ? 'No logs for this session yet.'
                : 'No logs yet. Start mining to see output.'
              }
            </p>
          ) : (
            filteredLogs.map((log, i) => (
              <div key={i} className={clsx('py-0.5 break-all', getLogColor(log))}>
                {log}
              </div>
            ))
          )}
        </div>

        {/* Jump to Latest Button */}
        {!autoScroll && (
          <button
            onClick={jumpToLatest}
            className="absolute bottom-4 right-4 flex items-center gap-2 rounded-lg bg-accent px-3 py-2 text-sm font-medium text-white shadow-lg transition-all hover:bg-accent-hover"
            aria-label="Jump to latest logs"
          >
            <ArrowDownIcon />
            Jump to Latest
          </button>
        )}
      </div>

      {/* Stats Bar */}
      <div className="flex items-center justify-between text-xs text-[var(--text-secondary)]">
        <span>{filteredLogs.length} lines</span>
        <span>
          {autoScroll ? '● Auto-scrolling' : '○ Scroll paused'}
        </span>
      </div>
    </div>
  )
}

function ArrowDownIcon() {
  return (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
    </svg>
  )
}
