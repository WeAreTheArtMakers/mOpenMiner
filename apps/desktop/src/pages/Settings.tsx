import { useState, useEffect } from 'react'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/tauri'

interface NotificationSettings {
  enabled: boolean
  pool_down: boolean
  hashrate_drop: boolean
  hashrate_drop_threshold: number
  miner_crash: boolean
  remote_offline: boolean
  update_available: boolean
  quiet_hours_enabled: boolean
  quiet_hours_start: number
  quiet_hours_end: number
}

interface ThreadBudgetSettings {
  mode: 'off' | 'warn_only' | 'auto_distribute' | 'enforce_limit'
  preset: 'eco' | 'balanced' | 'max'
  max_concurrent_sessions: number
}

interface BudgetStatus {
  effective_cores: number
  budget_threads: number
  total_requested: number
  is_overcommitted: boolean
  overcommit_ratio: number
  suggested_per_session: number
}

export function Settings() {
  const { theme, setTheme, setConsent, customBinaryPath, setCustomBinaryPath, exportDiagnostics } = useAppStore()
  const [binaryPath, setBinaryPath] = useState(customBinaryPath || '')
  const [exportStatus, setExportStatus] = useState<string | null>(null)
  
  // Notification settings
  const [notifSettings, setNotifSettings] = useState<NotificationSettings>({
    enabled: false,
    pool_down: true,
    hashrate_drop: true,
    hashrate_drop_threshold: 30,
    miner_crash: true,
    remote_offline: false,
    update_available: true,
    quiet_hours_enabled: false,
    quiet_hours_start: 22,
    quiet_hours_end: 8,
  })

  // Thread budget settings
  const [budgetSettings, setBudgetSettings] = useState<ThreadBudgetSettings>({
    mode: 'warn_only',
    preset: 'balanced',
    max_concurrent_sessions: 3,
  })
  const [budgetStatus, setBudgetStatus] = useState<BudgetStatus | null>(null)

  useEffect(() => {
    invoke<NotificationSettings>('get_notification_settings').then(setNotifSettings).catch(console.error)
    invoke<ThreadBudgetSettings>('get_thread_budget_settings').then(setBudgetSettings).catch(console.error)
    invoke<BudgetStatus>('get_budget_status').then(setBudgetStatus).catch(console.error)
  }, [])

  const handleSaveBinaryPath = () => {
    setCustomBinaryPath(binaryPath || null)
  }

  const handleNotifChange = async (key: keyof NotificationSettings, value: boolean | number) => {
    const updated = { ...notifSettings, [key]: value }
    setNotifSettings(updated)
    await invoke('set_notification_settings', { settings: updated })
  }

  const handleBudgetChange = async (key: keyof ThreadBudgetSettings, value: string | number) => {
    const updated = { ...budgetSettings, [key]: value }
    setBudgetSettings(updated)
    await invoke('set_thread_budget_settings', { settings: updated })
    // Refresh status
    invoke<BudgetStatus>('get_budget_status').then(setBudgetStatus).catch(console.error)
  }

  const handleTestNotification = async () => {
    await invoke('send_test_notification')
  }

  const handleExportDiagnostics = async (maskWallets: boolean) => {
    setExportStatus('Exporting...')
    const data = await exportDiagnostics(maskWallets)
    if (data) {
      const blob = new Blob([data], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `openminedash-diagnostics-${Date.now()}.json`
      a.click()
      URL.revokeObjectURL(url)
      setExportStatus('Exported!')
    } else {
      setExportStatus('Failed')
    }
    setTimeout(() => setExportStatus(null), 3000)
  }

  return (
    <div className="mx-auto max-w-2xl space-y-6">
      <header>
        <h1 className="text-2xl font-semibold">Settings</h1>
      </header>

      {/* Appearance */}
      <Section title="Appearance">
        <Field label="Theme">
          <select
            value={theme}
            onChange={(e) => setTheme(e.target.value as 'light' | 'dark' | 'system')}
            className="input"
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>
          </select>
        </Field>
      </Section>

      {/* Notifications */}
      <Section title="Notifications">
        <Toggle
          label="Enable notifications"
          checked={notifSettings.enabled}
          onChange={(v) => handleNotifChange('enabled', v)}
        />
        
        {notifSettings.enabled && (
          <div className="mt-4 space-y-3 pl-4 border-l-2 border-[var(--border)]">
            <Toggle
              label="Pool connection lost"
              checked={notifSettings.pool_down}
              onChange={(v) => handleNotifChange('pool_down', v)}
            />
            <Toggle
              label="Hashrate drop"
              checked={notifSettings.hashrate_drop}
              onChange={(v) => handleNotifChange('hashrate_drop', v)}
            />
            {notifSettings.hashrate_drop && (
              <Field label="Drop threshold (%)">
                <input
                  type="number"
                  min={10}
                  max={90}
                  value={notifSettings.hashrate_drop_threshold}
                  onChange={(e) => handleNotifChange('hashrate_drop_threshold', Number(e.target.value))}
                  className="input w-24"
                />
              </Field>
            )}
            <Toggle
              label="Miner crash/stop"
              checked={notifSettings.miner_crash}
              onChange={(v) => handleNotifChange('miner_crash', v)}
            />
            <Toggle
              label="Remote miner offline"
              checked={notifSettings.remote_offline}
              onChange={(v) => handleNotifChange('remote_offline', v)}
            />
            <Toggle
              label="Update available"
              checked={notifSettings.update_available}
              onChange={(v) => handleNotifChange('update_available', v)}
            />
            
            <div className="pt-2">
              <Toggle
                label="Quiet hours"
                checked={notifSettings.quiet_hours_enabled}
                onChange={(v) => handleNotifChange('quiet_hours_enabled', v)}
              />
              {notifSettings.quiet_hours_enabled && (
                <div className="mt-2 flex items-center gap-2 text-sm">
                  <span>From</span>
                  <input
                    type="number"
                    min={0}
                    max={23}
                    value={notifSettings.quiet_hours_start}
                    onChange={(e) => handleNotifChange('quiet_hours_start', Number(e.target.value))}
                    className="input w-16"
                  />
                  <span>to</span>
                  <input
                    type="number"
                    min={0}
                    max={23}
                    value={notifSettings.quiet_hours_end}
                    onChange={(e) => handleNotifChange('quiet_hours_end', Number(e.target.value))}
                    className="input w-16"
                  />
                </div>
              )}
            </div>
            
            <button onClick={handleTestNotification} className="btn-secondary mt-2">
              Send Test Notification
            </button>
          </div>
        )}
      </Section>

      {/* Miner Binary */}
      <Section title="Miner Binary">
        <p className="text-sm text-[var(--text-secondary)] mb-3">
          Custom XMRig path for enterprise or custom builds.
        </p>
        <div className="flex gap-2">
          <input
            type="text"
            value={binaryPath}
            onChange={(e) => setBinaryPath(e.target.value)}
            placeholder="/path/to/xmrig"
            className="input flex-1 font-mono"
          />
          <button onClick={handleSaveBinaryPath} className="btn-primary">Save</button>
        </div>
      </Section>

      {/* Thread Budget */}
      <Section title="Thread Budget">
        <p className="text-sm text-[var(--text-secondary)] mb-3">
          Manage CPU thread allocation across multiple mining sessions.
        </p>
        
        {budgetStatus && (
          <div className="mb-4 p-3 rounded-lg bg-[var(--surface)] text-sm">
            <div className="flex justify-between mb-1">
              <span>CPU Cores:</span>
              <span className="font-mono">{budgetStatus.effective_cores}</span>
            </div>
            <div className="flex justify-between mb-1">
              <span>Budget ({budgetSettings.preset}):</span>
              <span className="font-mono">{budgetStatus.budget_threads} threads</span>
            </div>
            {budgetStatus.total_requested > 0 && (
              <div className="flex justify-between">
                <span>Currently Used:</span>
                <span className={`font-mono ${budgetStatus.is_overcommitted ? 'text-warning' : ''}`}>
                  {budgetStatus.total_requested} threads
                  {budgetStatus.is_overcommitted && ` (${Math.round(budgetStatus.overcommit_ratio * 100)}%)`}
                </span>
              </div>
            )}
          </div>
        )}

        <Field label="Budget Mode">
          <select
            value={budgetSettings.mode}
            onChange={(e) => handleBudgetChange('mode', e.target.value)}
            className="input"
          >
            <option value="off">Off - No management</option>
            <option value="warn_only">Warn Only - Detect overcommit (default)</option>
            <option value="auto_distribute">Auto Distribute - Suggest thread split</option>
            <option value="enforce_limit">Enforce Limit - Cap total threads</option>
          </select>
        </Field>

        <Field label="Budget Preset">
          <select
            value={budgetSettings.preset}
            onChange={(e) => handleBudgetChange('preset', e.target.value)}
            className="input mt-3"
          >
            <option value="eco">Eco - 50% of cores</option>
            <option value="balanced">Balanced - 80% of cores (default)</option>
            <option value="max">Max - 100% of cores</option>
          </select>
        </Field>

        <Field label="Max Concurrent Sessions">
          <input
            type="number"
            min={1}
            max={10}
            value={budgetSettings.max_concurrent_sessions}
            onChange={(e) => handleBudgetChange('max_concurrent_sessions', Number(e.target.value))}
            className="input w-24 mt-3"
          />
        </Field>
      </Section>

      {/* Diagnostics */}
      <Section title="Diagnostics">
        <p className="text-sm text-[var(--text-secondary)] mb-3">
          Export for bug reports. Includes config and logs.
        </p>
        <div className="flex gap-2">
          <button onClick={() => handleExportDiagnostics(true)} className="btn-secondary">
            Export (Masked)
          </button>
          <button onClick={() => handleExportDiagnostics(false)} className="btn-secondary">
            Export (Full)
          </button>
        </div>
        {exportStatus && <p className="mt-2 text-sm text-[var(--text-secondary)]">{exportStatus}</p>}
      </Section>

      {/* Danger Zone */}
      <section className="rounded-xl border border-danger/30 bg-danger/5 p-6">
        <h2 className="mb-2 text-lg font-medium text-danger">Danger Zone</h2>
        <p className="mb-4 text-sm text-[var(--text-secondary)]">
          Revoke consent to disable all mining functionality.
        </p>
        <button onClick={() => setConsent(false)} className="btn-danger">
          Revoke Mining Consent
        </button>
      </section>
    </div>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-xl border border-[var(--border)] bg-surface-elevated p-6">
      <h2 className="mb-4 text-lg font-medium">{title}</h2>
      {children}
    </section>
  )
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <label className="mb-1 block text-sm font-medium">{label}</label>
      {children}
    </div>
  )
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center gap-3 cursor-pointer">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-4 w-4 rounded border-[var(--border)] text-accent focus:ring-accent"
      />
      <span className="text-sm">{label}</span>
    </label>
  )
}
