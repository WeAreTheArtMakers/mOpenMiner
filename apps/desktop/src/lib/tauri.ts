import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export { invoke }

export async function subscribeToLogs(callback: (log: string) => void) {
  return listen<string>('miner-log', (event) => {
    callback(event.payload)
  })
}

export async function subscribeToStats(callback: (stats: MinerStats) => void) {
  return listen<MinerStats>('miner-stats', (event) => {
    callback(event.payload)
  })
}

export interface MinerStats {
  hashrate: number
  avgHashrate: number
  acceptedShares: number
  rejectedShares: number
  uptime: number
}

export interface PoolHealthResult {
  url: string
  connected: boolean
  latencyMs: number | null
  error: string | null
}

export async function checkPoolHealth(url: string): Promise<PoolHealthResult> {
  return invoke('check_pool_health', { url })
}
