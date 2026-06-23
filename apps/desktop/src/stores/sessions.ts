import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/tauri'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

export type SessionStatus = 'stopped' | 'starting' | 'running' | 'suspended' | 'stopping' | 'error'
export type MinerKind = 'xmrig' | 'cpuminer-opt'
export type PerformancePreset = 'eco' | 'balanced' | 'max'

export interface SessionConfig {
  coin_id: string
  symbol: string
  algorithm: string
  miner_kind: MinerKind
  pool_url: string
  wallet: string
  worker: string
  preset: PerformancePreset
  threads_hint: number
  created_at: number
}

export interface SessionStats {
  status: SessionStatus
  hashrate_current: number
  hashrate_avg60: number
  accepted: number
  rejected: number
  difficulty: number
  last_share_time: number | null
  uptime_secs: number
  connected: boolean
  last_error: string | null
  stats_confidence: number
  telemetry_confidence: 'high' | 'medium' | 'low' | 'unknown'
  telemetry_reason: string
  connection_state: 'connecting' | 'connected' | 'subscribed' | 'authorized' | 'unknown'
  overcommitted: boolean
  overcommit_ratio: number
}

export interface SessionSummary {
  id: string
  config: SessionConfig
  stats: SessionStats
}

export interface SessionDetails {
  id: string
  config: SessionConfig
  stats: SessionStats
}

export interface LogEntry {
  timestamp: number
  line: string
}

export interface LogsResponse {
  session_id: string
  lines: LogEntry[]
  next_cursor: number | null
  has_more: boolean
}

interface SessionsState {
  sessions: Record<string, SessionSummary>
  activeSessionId: string | null
  isLoading: boolean
  error: string | null
  
  // Actions
  hydrate: () => Promise<void>
  startSession: (config: Omit<SessionConfig, 'miner_kind' | 'created_at'>) => Promise<string>
  stopSession: (sessionId: string) => Promise<void>
  suspendSession: (sessionId: string) => Promise<void>
  resumeSession: (sessionId: string) => Promise<void>
  stopAll: () => Promise<void>
  refreshStats: () => Promise<void>
  setActiveSession: (sessionId: string | null) => void
  getSessionLogs: (sessionId: string, cursor?: number, limit?: number) => Promise<LogsResponse | null>
  
  // Event handlers
  setupEventListeners: () => Promise<UnlistenFn[]>
}

export const useSessionsStore = create<SessionsState>((set, get) => ({
  sessions: {},
  activeSessionId: null,
  isLoading: false,
  error: null,

  hydrate: async () => {
    set({ isLoading: true, error: null })
    try {
      const sessions = await invoke<SessionSummary[]>('list_sessions')
      const sessionsMap: Record<string, SessionSummary> = {}
      for (const session of sessions) {
        sessionsMap[session.id] = session
      }
      set({ sessions: sessionsMap, isLoading: false })
    } catch (e) {
      set({ error: String(e), isLoading: false })
    }
  },

  startSession: async (config) => {
    set({ isLoading: true, error: null })
    try {
      const sessionId = await invoke<string>('start_session', { 
        config: {
          ...config,
          miner_kind: 'xmrig', // Will be determined by backend
          created_at: Date.now(),
        }
      })
      await get().hydrate()
      set({ activeSessionId: sessionId })
      return sessionId
    } catch (e) {
      set({ error: String(e), isLoading: false })
      throw e
    }
  },

  stopSession: async (sessionId) => {
    try {
      await invoke('stop_session', { sessionId })
      await get().hydrate()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  suspendSession: async (sessionId) => {
    try {
      await invoke('suspend_session', { sessionId })
      await get().hydrate()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  resumeSession: async (sessionId) => {
    try {
      await invoke('resume_session', { sessionId })
      await get().hydrate()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  stopAll: async () => {
    try {
      await invoke('stop_all_sessions')
      await get().hydrate()
    } catch (e) {
      set({ error: String(e) })
    }
  },

  refreshStats: async () => {
    try {
      const sessions = await invoke<SessionSummary[]>('refresh_session_stats')
      const sessionsMap: Record<string, SessionSummary> = {}
      for (const session of sessions) {
        sessionsMap[session.id] = session
      }
      set({ sessions: sessionsMap })
    } catch (e) {
      console.error('Failed to refresh stats:', e)
    }
  },

  setActiveSession: (sessionId) => {
    set({ activeSessionId: sessionId })
  },

  getSessionLogs: async (sessionId, cursor, limit) => {
    try {
      return await invoke<LogsResponse | null>('get_session_logs', { 
        sessionId, 
        cursor, 
        limit 
      })
    } catch (e) {
      console.error('Failed to get logs:', e)
      return null
    }
  },

  setupEventListeners: async () => {
    const unlisteners: UnlistenFn[] = []

    // Session created
    unlisteners.push(await listen('session://created', () => {
      get().hydrate()
    }))

    // Session updated (stats) - single session
    unlisteners.push(await listen<{ session_id: string }>('session://updated', () => {
      get().refreshStats()
    }))

    // Batch stats update (throttled 1Hz)
    unlisteners.push(await listen<{ sessions: SessionSummary[] }>('session://batch_updated', (event) => {
      const sessionsMap = { ...get().sessions }
      for (const session of event.payload.sessions) {
        sessionsMap[session.id] = session
      }
      set({ sessions: sessionsMap })
    }))

    // Log batch (batched in chunks of 20)
    unlisteners.push(await listen<{ session_id: string; lines: string[] }>('session://log_batch', () => {
      // Logs are fetched on-demand via getSessionLogs, this is for live updates
      // Could emit to a separate log store if needed
    }))

    // Session stopped
    unlisteners.push(await listen('session://stopped', () => {
      get().hydrate()
    }))

    // All sessions stopped
    unlisteners.push(await listen('session://all_stopped', () => {
      get().hydrate()
    }))

    return unlisteners
  },
}))

// Computed selectors
export const useActiveSessions = () => {
  const sessions = useSessionsStore((s) => s.sessions)
  return Object.values(sessions).filter(
    (s) => s.stats.status === 'running' || s.stats.status === 'suspended'
  )
}

export const useActiveSessionCount = () => {
  return useActiveSessions().length
}

export const useTotalHashrate = () => {
  const sessions = useActiveSessions()
  return sessions.reduce((sum, s) => sum + s.stats.hashrate_current, 0)
}
