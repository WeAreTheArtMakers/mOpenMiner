import { useAppStore } from '@/stores/app'

export function Profiles() {
  const { profiles } = useAppStore()

  return (
    <div className="mx-auto max-w-4xl space-y-8">
      <header>
        <h1 className="text-2xl font-semibold">Profiles</h1>
        <p className="mt-1 text-[var(--text-secondary)]">Save and load mining configurations</p>
      </header>

      {profiles.length === 0 ? (
        <div className="rounded-xl border border-dashed border-[var(--border)] p-12 text-center">
          <p className="text-[var(--text-secondary)]">No profiles saved yet.</p>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">
            Configure mining on the Dashboard and save as a profile.
          </p>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2">
          {profiles.map((profile) => (
            <div
              key={profile.id}
              className="rounded-xl border border-[var(--border)] bg-surface-elevated p-4"
            >
              <h3 className="font-medium">{profile.name}</h3>
              <p className="mt-1 text-sm text-[var(--text-secondary)]">
                {profile.coin.toUpperCase()} • {profile.threads} threads
              </p>
              <p className="mt-2 truncate font-mono text-xs text-[var(--text-secondary)]">
                {profile.wallet}
              </p>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
