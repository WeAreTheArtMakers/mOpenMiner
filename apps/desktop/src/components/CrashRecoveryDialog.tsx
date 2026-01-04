interface Props {
  lastSession: {
    coin: string
    pool: string
    wallet: string
    worker: string
  }
  onDismiss: () => void
  onRestart: () => void
}

export function CrashRecoveryDialog({ lastSession, onDismiss, onRestart }: Props) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="recovery-title"
    >
      <div className="mx-4 max-w-md rounded-2xl bg-surface-elevated p-8 shadow-2xl">
        <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-yellow-500/10">
          <svg className="h-8 w-8 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        
        <h2 id="recovery-title" className="mb-3 text-xl font-semibold">
          Previous Session Detected
        </h2>
        
        <p className="mb-4 text-[var(--text-secondary)] leading-relaxed">
          The app was closed while mining was active. Would you like to resume?
        </p>
        
        <div className="mb-6 rounded-lg bg-surface p-3 text-sm">
          <p><span className="text-[var(--text-secondary)]">Coin:</span> {lastSession.coin.toUpperCase()}</p>
          <p className="truncate"><span className="text-[var(--text-secondary)]">Pool:</span> {lastSession.pool}</p>
          <p><span className="text-[var(--text-secondary)]">Worker:</span> {lastSession.worker || 'default'}</p>
        </div>
        
        <div className="flex gap-3">
          <button
            onClick={onDismiss}
            className="flex-1 rounded-lg border border-[var(--border)] px-4 py-3 font-medium transition-colors hover:bg-[var(--border)]"
          >
            No, Start Fresh
          </button>
          <button
            onClick={onRestart}
            className="flex-1 rounded-lg bg-accent px-4 py-3 font-medium text-white transition-colors hover:bg-accent-hover"
          >
            Resume Mining
          </button>
        </div>
        
        <p className="mt-4 text-center text-xs text-[var(--text-secondary)]">
          Mining will not start automatically without your action
        </p>
      </div>
    </div>
  )
}
