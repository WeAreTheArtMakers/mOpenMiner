# Architecture

## Overview

OpenMineDash is a Tauri-based desktop application with a Rust backend and React frontend.

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Window                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              React UI (TypeScript)                │  │
│  │  ┌─────────┬─────────┬────────┬────────┬──────┐  │  │
│  │  │Dashboard│Profiles │ Pools  │  Logs  │About │  │  │
│  │  └─────────┴─────────┴────────┴────────┴──────┘  │  │
│  └───────────────────────────────────────────────────┘  │
│                         │ IPC                           │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Rust Backend (Tauri)                 │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │  │
│  │  │  Core    │ │ Adapters │ │  Pools           │  │  │
│  │  │ - Config │ │ - XMRig  │ │ - Health Check   │  │  │
│  │  │ - Process│ │ - ASIC   │ │ - Stratum Test   │  │  │
│  │  │ - Telemetry│         │ │                  │  │  │
│  │  └──────────┘ └──────────┘ └──────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
              ┌───────────┴───────────┐
              │                       │
        ┌─────▼─────┐          ┌──────▼──────┐
        │  XMRig    │          │ External    │
        │ (child    │          │ ASIC Miner  │
        │  process) │          │ (Stratum)   │
        └───────────┘          └─────────────┘
```

## Directory Structure

```
openminedash/
├── apps/
│   └── desktop/
│       ├── src/                 # React frontend
│       │   ├── components/      # UI components
│       │   ├── pages/           # Route pages
│       │   ├── hooks/           # Custom React hooks
│       │   ├── stores/          # State management
│       │   └── lib/             # Utilities
│       ├── src-tauri/           # Tauri Rust backend
│       │   ├── src/
│       │   │   ├── main.rs
│       │   │   └── commands/    # IPC command handlers
│       │   └── Cargo.toml
│       ├── index.html
│       └── package.json
│
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config.rs        # App configuration
│   │   │   ├── process.rs       # Process manager
│   │   │   └── telemetry.rs     # Stats collection
│   │   └── Cargo.toml
│   │
│   ├── miner_adapters/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── xmrig.rs         # XMRig adapter
│   │   │   └── external.rs      # External ASIC adapter
│   │   └── Cargo.toml
│   │
│   └── pools/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── health.rs        # Pool health check
│       │   └── stratum.rs       # Stratum protocol
│       └── Cargo.toml
│
├── assets/
│   └── coins/
│       ├── xmr.json             # Monero definition
│       ├── btc.json             # Bitcoin definition
│       └── ltc.json             # Litecoin definition
│
└── docs/
    ├── THREAT_MODEL.md
    ├── USAGE.md
    └── DEVELOPMENT.md
```

## Core Components

### Process Manager (crates/core/process.rs)

Manages miner child processes:
- Start with configured parameters
- Stream stdout/stderr to UI
- Graceful shutdown (SIGTERM → SIGKILL fallback)
- Kill switch for immediate termination

### Miner Adapters (crates/miner_adapters/)

Abstraction layer for different miners:
- `XMRigAdapter`: Manages XMRig binary, parses API stats
- `ExternalAdapter`: Monitors external ASIC miners via Stratum

### Telemetry (crates/core/telemetry.rs)

Collects and aggregates stats:
- XMRig: HTTP API on 127.0.0.1
- Fallback: Log parsing for other miners
- System: CPU usage, temperature (via sysinfo)

### Coin Plugin System (assets/coins/)

JSON-based coin definitions:
```json
{
  "id": "xmr",
  "name": "Monero",
  "symbol": "XMR",
  "algorithm": "randomx",
  "recommended_miner": "xmrig",
  "default_pools": [...],
  "notes": null
}
```

## IPC Commands

| Command | Description |
|---------|-------------|
| `start_mining` | Start miner with config |
| `stop_mining` | Stop running miner |
| `get_status` | Get current mining status |
| `get_logs` | Get log buffer |
| `list_coins` | List available coins |
| `save_profile` | Save mining profile |
| `load_profile` | Load mining profile |
| `check_pool_health` | Test pool connectivity |

## Security Boundaries

1. **User Consent Gate**: First-run consent dialog blocks all mining
2. **Process Isolation**: Miners run as child processes, not embedded
3. **Binary Verification**: SHA256 checksum before execution
4. **No Persistence**: No launchd/auto-start registration
5. **Public Data Only**: Only wallet addresses stored, never private keys
