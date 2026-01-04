export function About() {
  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <header>
        <h1 className="text-2xl font-semibold">About</h1>
        <p className="mt-1 text-[var(--text-secondary)]">OpenMineDash v0.1.0</p>
      </header>

      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="mb-4 text-lg font-medium">What is OpenMineDash?</h2>
        <p className="text-[var(--text-secondary)] leading-relaxed">
          OpenMineDash is a transparent, open-source mining dashboard and manager for macOS.
          It orchestrates legitimate mining software with full user control and transparency.
        </p>
        <p className="mt-4 text-[var(--text-secondary)] leading-relaxed">
          Developed by <a href="https://wearetheartmakers.com" target="_blank" rel="noopener noreferrer" className="text-accent hover:underline font-medium">WATAM (We Are The Art Makers)</a>
        </p>
      </section>

      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="mb-4 text-lg font-medium">Security Commitment</h2>
        <ul className="space-y-2 text-sm text-[var(--text-secondary)]">
          <SecurityItem text="No hidden or background mining" />
          <SecurityItem text="Explicit user consent required" />
          <SecurityItem text="Binary checksum verification" />
          <SecurityItem text="No private keys stored" />
          <SecurityItem text="Default OFF, no auto-start" />
        </ul>
      </section>

      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="mb-4 text-lg font-medium">Built With</h2>
        <p className="text-sm text-[var(--text-secondary)] mb-3">
          This app uses the following open-source technologies:
        </p>
        <div className="flex flex-wrap gap-2 text-xs">
          <span className="rounded-full bg-surface px-3 py-1">Tauri (Desktop Framework)</span>
          <span className="rounded-full bg-surface px-3 py-1">React (UI)</span>
          <span className="rounded-full bg-surface px-3 py-1">Rust (Backend)</span>
          <span className="rounded-full bg-surface px-3 py-1">XMRig (Miner)</span>
        </div>
      </section>

      <section className="rounded-xl border border-accent/30 bg-accent/5 p-6">
        <h2 className="mb-4 text-lg font-medium">Created By</h2>
        <div className="flex items-center gap-4">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-accent/20 text-xl font-bold text-accent">
            W
          </div>
          <div>
            <p className="font-medium">WATAM</p>
            <a 
              href="https://wearetheartmakers.com" 
              target="_blank" 
              rel="noopener noreferrer" 
              className="text-sm text-accent hover:underline"
            >
              wearetheartmakers.com
            </a>
          </div>
        </div>
      </section>

      <section className="rounded-xl border border-yellow-500/30 bg-yellow-500/5 p-6">
        <h2 className="mb-4 text-lg font-medium text-yellow-600 dark:text-yellow-400">
          Important Disclaimers
        </h2>
        <ul className="space-y-3 text-sm text-[var(--text-secondary)]">
          <li className="flex items-start gap-2">
            <span className="mt-1 text-yellow-500">⚠</span>
            <span>This software is not investment advice. Cryptocurrency mining involves risks.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="mt-1 text-yellow-500">⚠</span>
            <span>BTC/LTC CPU mining is not practical. ASIC hardware is required for profitability.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="mt-1 text-yellow-500">⚠</span>
            <span>Mining is disabled by default. No auto-start, no background processes.</span>
          </li>
          <li className="flex items-start gap-2">
            <span className="mt-1 text-yellow-500">⚠</span>
            <span>You are responsible for electricity costs and hardware wear.</span>
          </li>
        </ul>
      </section>

      <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
        <h2 className="mb-4 text-lg font-medium">License</h2>
        <p className="text-sm text-[var(--text-secondary)]">
          MIT License © 2026 WATAM (We Are The Art Makers)
        </p>
      </section>
    </div>
  )
}

function SecurityItem({ text }: { text: string }) {
  return (
    <li className="flex items-start gap-2">
      <svg className="mt-0.5 h-4 w-4 flex-shrink-0 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
      </svg>
      {text}
    </li>
  )
}


