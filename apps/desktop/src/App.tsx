import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/stores/app'
import { useTrayEvents } from '@/hooks/useTrayEvents'
import { ConsentDialog } from '@/components/ConsentDialog'
import { CrashRecoveryDialog } from '@/components/CrashRecoveryDialog'
import { Sidebar } from '@/components/Sidebar'
import { Dashboard } from '@/pages/Dashboard'
import { Earnings } from '@/pages/Earnings'
import { Profiles } from '@/pages/Profiles'
import { Pools } from '@/pages/Pools'
import { Logs } from '@/pages/Logs'
import { Settings } from '@/pages/Settings'
import { About } from '@/pages/About'

export default function App() {
  const { 
    hasConsent, 
    currentPage, 
    initializeApp, 
    crashRecovery, 
    clearCrashRecovery, 
    startMining,
    currentPreset,
    appendLog,
  } = useAppStore()

  // Handle tray menu events
  useTrayEvents()

  useEffect(() => {
    initializeApp()
  }, [initializeApp])

  // Listen for miner log events
  useEffect(() => {
    const unlisten = listen<string>('miner-log', (event) => {
      appendLog(event.payload)
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [appendLog])

  const renderPage = () => {
    switch (currentPage) {
      case 'dashboard': return <Dashboard />
      case 'earnings': return <Earnings />
      case 'profiles': return <Profiles />
      case 'pools': return <Pools />
      case 'logs': return <Logs />
      case 'settings': return <Settings />
      case 'about': return <About />
      default: return <Dashboard />
    }
  }

  const handleCrashRecoveryRestart = () => {
    if (crashRecovery?.last_session) {
      const session = crashRecovery.last_session
      const { coins } = useAppStore.getState()
      const coinData = coins.find(c => c.id === session.coin)
      const algorithm = coinData?.algorithm || session.coin
      
      startMining({
        coin: session.coin,
        pool: session.pool,
        wallet: session.wallet,
        worker: session.worker,
        threads: 0, // Auto
        preset: currentPreset,
        algorithm,
        tryAnyway: false,
      })
    }
    clearCrashRecovery()
  }

  return (
    <div className="flex h-screen bg-surface">
      {/* Consent dialog - blocks everything */}
      {!hasConsent && <ConsentDialog />}
      
      {/* Crash recovery dialog - shows after consent, before normal UI */}
      {hasConsent && crashRecovery?.had_unclean_shutdown && crashRecovery.last_session && (
        <CrashRecoveryDialog
          lastSession={crashRecovery.last_session}
          onDismiss={clearCrashRecovery}
          onRestart={handleCrashRecoveryRestart}
        />
      )}
      
      <Sidebar />
      <main className="flex-1 overflow-auto p-6">
        {renderPage()}
      </main>
    </div>
  )
}
