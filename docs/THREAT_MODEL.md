# Threat Model

## Assets

1. **User's CPU/GPU resources** - Primary asset to protect from unauthorized use
2. **Wallet addresses** - Public, but privacy-sensitive
3. **Mining earnings** - Directed by wallet address configuration
4. **System stability** - Mining should not crash or degrade the system

## Threat Actors

### Malicious Fork/Distribution
- **Threat**: Someone forks and adds cryptojacking
- **Mitigation**: Code signing, official distribution channels, reproducible builds

### Supply Chain Attack
- **Threat**: Compromised miner binary
- **Mitigation**: SHA256 verification, pinned versions, official source URLs only

### Local Privilege Escalation
- **Threat**: Malware uses our app to hide mining
- **Mitigation**: No elevated privileges, standard user permissions only

## Attack Vectors & Mitigations

| Vector | Risk | Mitigation |
|--------|------|------------|
| Auto-start persistence | High | No launchd/login items ever |
| Hidden background mining | High | Consent gate, visible UI, process tied to app |
| Malicious pool redirect | Medium | User confirms pool, health check shows endpoint |
| Binary tampering | Medium | Checksum verification before every launch |
| Config injection | Low | Sanitized inputs, no shell execution of user data |
| Memory scraping | Low | No private keys in memory |

## Trust Boundaries

```
┌─────────────────────────────────────────┐
│           Trusted (Our Code)            │
│  ┌─────────────────────────────────┐    │
│  │  Tauri App + Rust Backend       │    │
│  │  - Config management            │    │
│  │  - Process orchestration        │    │
│  │  - UI rendering                 │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
         Trust Boundary (Verified)
                    │
┌─────────────────────────────────────────┐
│      Semi-Trusted (Verified Binary)     │
│  ┌─────────────────────────────────┐    │
│  │  XMRig (checksum verified)      │    │
│  │  - Actual mining execution      │    │
│  │  - Direct pool communication    │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
                    │
         Trust Boundary (Network)
                    │
┌─────────────────────────────────────────┐
│           Untrusted (External)          │
│  - Mining pools                         │
│  - Network responses                    │
│  - User-provided custom pools           │
└─────────────────────────────────────────┘
```

## Security Controls

### Consent System
- First-run modal blocks all functionality
- Stored in local config, not system keychain
- Revocable at any time

### Process Isolation
- Miner runs as child process
- Stdout/stderr captured, not executed
- No shell interpretation of miner output

### Resource Limits
- Configurable CPU thread limit
- Priority setting (low/normal)
- User can set max temperature threshold (future)
