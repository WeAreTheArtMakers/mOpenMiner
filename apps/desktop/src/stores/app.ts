import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/tauri'

export type Page = 'dashboard' | 'earnings' | 'profiles' | 'pools' | 'logs' | 'settings' | 'about'
export type MinerState = 'stopped' | 'starting' | 'running' | 'stopping' | 'error'
export type PerformancePreset = 'eco' | 'balanced' | 'max'

export interface MiningStatus {
  state: MinerState
  isRunning: boolean
  coin: string | null
  pool: string | null
  worker: string | null
  hashrate: number
  avgHashrate: number
  acceptedShares: number
  rejectedShares: number
  uptime: number
  activeMiner: string
  warning: string | null
}

export interface HashratePoint {
  timestamp: number
  hashrate: number
}

export interface CoinDefinition {
  id: string
  name: string
  symbol: string
  algorithm: string
  recommended_miner: string
  cpu_mineable: boolean
  default_pools: PoolConfig[]
  notes: string | null
  trusted: boolean
}

export interface PoolConfig {
  name: string
  stratum_url: string
  tls: boolean
  region: string
}

export interface Profile {
  id: string
  name: string
  coin: string
  pool: string
  wallet: string
  worker: string
  threads: number
  preset: PerformancePreset
}

export interface CrashRecoveryState {
  had_unclean_shutdown: boolean
  last_session: {
    coin: string
    pool: string
    wallet: string
    worker: string
  } | null
}

interface AppState {
  hasConsent: boolean
  currentPage: Page
  theme: 'light' | 'dark' | 'system'
  status: MiningStatus
  coins: CoinDefinition[]
  profiles: Profile[]
  logs: string[]
  customBinaryPath: string | null
  crashRecovery: CrashRecoveryState | null
  currentPreset: PerformancePreset
  hashrateHistory: HashratePoint[]
  
  // Actions
  setConsent: (consent: boolean) => void
  setPage: (page: Page) => void
  setTheme: (theme: 'light' | 'dark' | 'system') => void
  setPreset: (preset: PerformancePreset) => void
  setCustomBinaryPath: (path: string | null) => void
  initializeApp: () => Promise<void>
  startMining: (config: {
    coin: string
    pool: string
    wallet: string
    worker: string
    threads: number
    preset: PerformancePreset
    algorithm: string
    tryAnyway: boolean
  }) => Promise<void>
  stopMining: () => Promise<void>
  refreshStatus: () => Promise<void>
  loadCoins: () => Promise<void>
  saveProfile: (profile: Omit<Profile, 'id'>) => Promise<void>
  deleteProfile: (profileId: string) => Promise<void>
  loadProfiles: () => Promise<void>
  appendLog: (log: string) => void
  exportDiagnostics: (maskWallets: boolean) => Promise<string>
  clearCrashRecovery: () => void
}

const defaultStatus: MiningStatus = {
  state: 'stopped',
  isRunning: false,
  coin: null,
  pool: null,
  worker: null,
  hashrate: 0,
  avgHashrate: 0,
  acceptedShares: 0,
  rejectedShares: 0,
  uptime: 0,
  activeMiner: '',
  warning: null,
}

export const useAppStore = create<AppState>((set, get) => ({
  hasConsent: false,
  currentPage: 'dashboard',
  theme: 'dark',
  status: defaultStatus,
  coins: [],
  profiles: [],
  logs: [],
  customBinaryPath: null,
  crashRecovery: null,
  currentPreset: 'balanced',
  hashrateHistory: [],

  setConsent: (consent) => {
    set({ hasConsent: consent })
    invoke('set_consent', { consent }).catch(console.error)
  },

  setPage: (page) => set({ currentPage: page }),

  setTheme: (theme) => {
    set({ theme })
    document.documentElement.classList.toggle('dark', theme === 'dark')
    invoke('set_theme', { theme }).catch(console.error)
  },

  setPreset: (preset) => {
    set({ currentPreset: preset })
  },

  setCustomBinaryPath: (path) => {
    set({ customBinaryPath: path })
    invoke('set_custom_binary_path', { path }).catch(console.error)
  },

  initializeApp: async () => {
    try {
      const [consent, theme, customPath, crashRecovery] = await Promise.all([
        invoke<boolean>('get_consent'),
        invoke<string>('get_theme'),
        invoke<string | null>('get_custom_binary_path'),
        invoke<CrashRecoveryState>('get_crash_recovery_state'),
      ])
      
      set({
        hasConsent: consent,
        theme: theme as 'light' | 'dark' | 'system',
        customBinaryPath: customPath,
        crashRecovery: crashRecovery.had_unclean_shutdown ? crashRecovery : null,
      })
      
      document.documentElement.classList.toggle('dark', theme === 'dark')
      await get().loadCoins()
      await get().loadProfiles()
    } catch (e) {
      console.error('Failed to initialize:', e)
    }
  },

  startMining: async (config) => {
    const state = get()
    if (!state.hasConsent) return
    if (state.status.state !== 'stopped' && state.status.state !== 'error') return

    set((s) => ({ status: { ...s.status, state: 'starting' } }))

    try {
      await invoke('start_mining', { config })
      set((s) => ({
        status: {
          ...s.status,
          state: 'running',
          isRunning: true,
          coin: config.coin,
          pool: config.pool,
          worker: config.worker,
        },
        currentPreset: config.preset,
      }))
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e)
      console.error('Failed to start mining:', errorMsg)
      set((s) => ({ 
        status: { 
          ...s.status, 
          state: 'error',
          warning: errorMsg,
        } 
      }))
    }
  },

  stopMining: async () => {
    const state = get()
    if (state.status.state !== 'running') return

    set((s) => ({ status: { ...s.status, state: 'stopping' } }))

    try {
      await invoke('stop_mining')
      set({ status: defaultStatus, hashrateHistory: [] })
    } catch (e) {
      console.error('Failed to stop mining:', e)
    }
  },

  refreshStatus: async () => {
    try {
      const raw = await invoke<{
        state: string
        is_running: boolean
        coin: string | null
        pool: string | null
        wallet: string | null
        worker: string | null
        hashrate: number
        avg_hashrate: number
        accepted_shares: number
        rejected_shares: number
        uptime: number
        active_miner: string
        warning: string | null
        started_at: number
        algorithm: string | null
      }>('get_status')
      
      // Map snake_case to camelCase
      const status: MiningStatus = {
        state: raw.state as MinerState,
        isRunning: raw.is_running,
        coin: raw.coin,
        pool: raw.pool,
        worker: raw.worker,
        hashrate: raw.hashrate,
        avgHashrate: raw.avg_hashrate,
        acceptedShares: raw.accepted_shares,
        rejectedShares: raw.rejected_shares,
        uptime: raw.uptime,
        activeMiner: raw.active_miner,
        warning: raw.warning,
      }
      
      // Update hashrate history (keep last 60 points = ~1 hour at 1 min intervals)
      set((state) => {
        const now = Date.now()
        const newHistory = [...state.hashrateHistory]
        
        // Only add if mining and hashrate > 0
        if (raw.is_running && raw.hashrate > 0) {
          // Add new point if last point is > 30 seconds old
          const lastPoint = newHistory[newHistory.length - 1]
          if (!lastPoint || now - lastPoint.timestamp > 30000) {
            newHistory.push({ timestamp: now, hashrate: raw.hashrate })
          }
          // Keep only last 120 points (1 hour at 30s intervals)
          while (newHistory.length > 120) {
            newHistory.shift()
          }
        }
        
        return { status, hashrateHistory: newHistory }
      })
    } catch (e) {
      console.error('Failed to get status:', e)
    }
  },

  loadCoins: async () => {
    try {
      const coins = await invoke<CoinDefinition[]>('list_coins')
      set({ coins })
    } catch (e) {
      console.error('Failed to load coins:', e)
    }
  },

  saveProfile: async (profile) => {
    try {
      const id = crypto.randomUUID()
      await invoke('save_profile', { profile: { ...profile, id } })
      await get().loadProfiles()
    } catch (e) {
      console.error('Failed to save profile:', e)
    }
  },

  deleteProfile: async (profileId) => {
    try {
      await invoke('delete_profile', { profileId })
      await get().loadProfiles()
    } catch (e) {
      console.error('Failed to delete profile:', e)
    }
  },

  loadProfiles: async () => {
    try {
      const profiles = await invoke<Profile[]>('list_profiles')
      set({ profiles })
    } catch (e) {
      console.error('Failed to load profiles:', e)
    }
  },

  appendLog: (log) => {
    set((state) => ({
      logs: [...state.logs.slice(-1999), log],
    }))
  },

  exportDiagnostics: async (maskWallets) => {
    try {
      return await invoke<string>('export_diagnostics', { maskWallets })
    } catch (e) {
      console.error('Failed to export diagnostics:', e)
      return ''
    }
  },

  clearCrashRecovery: () => {
    set({ crashRecovery: null })
    invoke('clear_crash_recovery').catch(console.error)
  },
}))
