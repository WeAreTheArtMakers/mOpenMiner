# macOS Sandbox Strategy

## Current Status: No Sandbox

OpenMineDash currently runs **without** macOS App Sandbox enabled.

### Why No Sandbox?

The primary reason is **miner binary execution**:

1. XMRig runs as a child process spawned by the app
2. Sandboxed apps cannot execute arbitrary binaries
3. The miner binary is downloaded/user-provided, not bundled

### Security Mitigations Without Sandbox

Since we can't use sandbox, we implement these security measures:

| Risk | Mitigation |
|------|------------|
| Malicious binary | SHA256 checksum verification |
| Path traversal | Canonical path validation |
| Unauthorized network | User-configured pools only |
| Data exfiltration | No telemetry, local-only config |
| Persistence | No launchd/login items |

### Entitlements Used

```xml
<!-- Current entitlements -->
<key>com.apple.security.network.client</key>
<true/>
```

We request only:
- Network client (for pool connections and XMRig API)

We do NOT request:
- `com.apple.security.app-sandbox` (disabled)
- `com.apple.security.files.user-selected.read-write`
- `com.apple.security.automation.apple-events`

### Future Considerations

If Apple requires sandbox for distribution:

**Option A: Bundled Miner**
- Bundle XMRig inside the app
- Requires maintaining builds for each XMRig version
- Loses flexibility of user-provided binaries

**Option B: XPC Service**
- Separate helper tool for miner execution
- Complex architecture
- May still face sandbox restrictions

**Option C: Stay Outside App Store**
- Distribute via GitHub releases
- Notarization still works without sandbox
- Current approach

### Notarization Without Sandbox

Apple allows notarization of non-sandboxed apps:
- Must be signed with Developer ID
- Must pass automated security checks
- Users see "Apple checked it for malicious software"

### Transparency

We document this clearly because:
1. Users should understand the security model
2. Enterprise users may have policies about sandboxing
3. It affects what the app can/cannot do

### File Access

Without sandbox, the app can access:
- `~/Library/Application Support/openminedash/` (config)
- `~/Library/Caches/openminedash/` (downloads)
- User-selected binary paths
- Network (pools, XMRig API)

The app does NOT access:
- Documents, Desktop, Downloads (unless user selects)
- Contacts, Calendar, Photos
- Other apps' data
- System files (no elevated privileges)

### Recommendation for Users

If you're concerned about running non-sandboxed apps:
1. Review the source code (it's open source)
2. Build from source yourself
3. Use a dedicated user account for mining
4. Monitor with Activity Monitor / Little Snitch
