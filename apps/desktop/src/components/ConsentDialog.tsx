import { useAppStore } from '@/stores/app'

export function ConsentDialog() {
  const setConsent = useAppStore((s) => s.setConsent)

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="consent-title"
    >
      <div className="mx-4 max-w-md rounded-2xl bg-surface-elevated p-8 shadow-2xl">
        <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-accent/10">
          <svg className="h-8 w-8 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        
        <h2 id="consent-title" className="mb-3 text-xl font-semibold">
          Mining Consent Required
        </h2>
        
        <p className="mb-6 text-[var(--text-secondary)] leading-relaxed">
          This software can perform cryptocurrency mining on your device. Mining will{' '}
          <strong className="text-[var(--text-primary)]">only run with your explicit permission</strong>{' '}
          and can be stopped at any time with a single click.
        </p>
        
        <ul className="mb-6 space-y-2 text-sm text-[var(--text-secondary)]">
          <li className="flex items-center gap-2">
            <svg className="h-4 w-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            Mining is disabled by default
          </li>
          <li className="flex items-center gap-2">
            <svg className="h-4 w-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            One-click stop always available
          </li>
          <li className="flex items-center gap-2">
            <svg className="h-4 w-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            No background or hidden processes
          </li>
        </ul>
        
        <div className="flex gap-3">
          <button
            onClick={() => setConsent(true)}
            className="flex-1 rounded-lg bg-accent px-4 py-3 font-medium text-white transition-colors hover:bg-accent-hover focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
          >
            I Understand & Agree
          </button>
        </div>
        
        <p className="mt-4 text-center text-xs text-[var(--text-secondary)]">
          You can revoke consent anytime in Settings
        </p>
      </div>
    </div>
  )
}
