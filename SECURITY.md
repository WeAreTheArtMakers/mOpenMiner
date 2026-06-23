# Security Policy

## Anti-Cryptojacking Commitment

OpenMineDash is designed with a **zero-tolerance policy for cryptojacking**.

## Core Principles

### 1. Explicit User Consent
- First-run consent dialog is mandatory
- No mining functionality accessible until consent is granted
- Consent can be revoked anytime in Settings

### 2. Default OFF, No Auto-Start
- Mining is never enabled by default
- **No launchd agents** are installed
- **No login items** are registered
- **No background daemons** after app quit
- User must explicitly click "Start Mining"

### 3. Instant Termination (Kill Switch)
- One-click STOP button always visible when mining
- Keyboard shortcut (Cmd+. or Escape) stops immediately
- Process termination: SIGTERM → 3s timeout → SIGKILL
- App quit always terminates child processes (Drop guard)

### 4. Safe Data Persistence
We store **non-sensitive configuration only**:
- ✅ Wallet addresses (public)
- ✅ Pool URLs
- ✅ Worker names, thread counts
- ✅ Performance presets
- ❌ Never: private keys, seeds, passwords

### 5. Supply Chain Security

#### Pinned Checksums
Miner binary checksums are **pinned in the application**, not fetched remotely:
```
assets/checksums/xmrig.json
├── 6.21.0-macos-arm64: "sha256:abc123..."
├── 6.21.0-macos-x64: "sha256:def456..."
└── ...
```

#### Download Verification
1. Download from official GitHub release URL
2. Compute SHA256 of downloaded file
3. Compare against pinned checksum
4. **Block execution if mismatch**
5. Log verification result

#### Manual Install Option
Enterprise users can:
- Provide their own verified XMRig binary
- Set custom binary path in Settings
- Skip automatic download entirely

## What We DON'T Do

| Anti-Pattern | Our Stance |
|--------------|------------|
| Hidden mining | ❌ Never |
| Auto-start on boot | ❌ Never |
| Background mining after quit | ❌ Never |
| Remote checksum fetching | ❌ Never |
| Obfuscated code | ❌ Never |
| AV evasion techniques | ❌ Never |
| Private key storage | ❌ Never |

## Process Management Guarantees

### State Machine
```
[Stopped] → [Starting] → [Running] → [Stopping] → [Stopped]
                              ↓
                          [Error]
```
- Buttons disabled during transitions
- No "Start" spam possible
- Error state shows diagnostic info

### Shutdown Sequence
1. User clicks STOP (or app quits)
2. Send SIGTERM to miner process
3. Wait up to 3 seconds
4. If still running: SIGKILL
5. Verify process terminated
6. Update UI state

### Drop Guard
Rust's `Drop` trait ensures child processes are killed even on panic:
```rust
impl Drop for MinerProcess {
    fn drop(&mut self) {
        // Force kill on any exit path
    }
}
```

## macOS Security Integration

### Gatekeeper Handling
- Detect `com.apple.quarantine` attribute
- Show user-friendly explanation
- Guide to System Settings → Privacy & Security
- Offer manual install alternative

### Future: Notarization
Release builds will be:
- Code signed with Apple Developer ID
- Notarized with Apple
- Stapled for offline verification

## Diagnostics Export

"Export Diagnostics" feature for bug reports:
- App version, macOS version, architecture
- Config (wallet addresses masked by default)
- Last 2000 log lines
- No automatic transmission - user controls sharing

## Reporting Security Issues

1. **Do NOT open a public issue**
2. Email: security@example.com
3. Include: description, reproduction steps, impact
4. Response within 48 hours
