import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/stores/app'

/**
 * Hook to handle tray menu events from Rust backend.
 * Tray is created in Rust (single source of truth), events are handled here.
 */
export function useTrayEvents() {
  const { startMining, stopMining, setPage } = useAppStore()

  useEffect(() => {
    const unlisten = listen<string>('tray-action', (event) => {
      const action = event.payload

      switch (action) {
        case 'stop':
          stopMining()
          break

        case 'start':
          // Can't start from tray without config - navigate to dashboard
          setPage('dashboard')
          break

        case 'navigate:dashboard':
          setPage('dashboard')
          break

        case 'navigate:logs':
          setPage('logs')
          break

        case 'quit':
          // Stop mining first, then quit
          stopMining().then(() => {
            // Tauri will handle the actual quit
            import('@tauri-apps/api/process').then(({ exit }) => exit(0))
          })
          break

        default:
          if (action.startsWith('preset:')) {
            const preset = action.replace('preset:', '')
            // Store preset preference - will be used on next start
            console.log('Preset changed to:', preset)
          }
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [startMining, stopMining, setPage])
}
