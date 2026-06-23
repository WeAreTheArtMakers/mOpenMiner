import { useAppStore } from '@/stores/app'

export function StopButton() {
  const { status, stopMining } = useAppStore()

  if (!status.isRunning) return null

  return (
    <button
      onClick={stopMining}
      className="fixed right-6 top-6 z-40 flex items-center gap-2 rounded-lg bg-danger px-6 py-3 font-semibold text-white shadow-lg transition-all hover:bg-danger-hover hover:scale-105 focus-visible:ring-2 focus-visible:ring-danger focus-visible:ring-offset-2"
      aria-label="Stop mining immediately"
    >
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
      </svg>
      STOP
    </button>
  )
}
