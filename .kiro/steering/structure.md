# Project Structure

```
openminedash/
├── .github/workflows/     # CI/CD
├── .kiro/steering/        # AI assistant guidance
├── apps/
│   └── desktop/           # Tauri + React application
│       ├── src/           # React frontend
│       │   ├── components/  # UI components
│       │   ├── pages/       # Route pages
│       │   ├── stores/      # Zustand state
│       │   ├── lib/         # Utilities
│       │   └── styles/      # CSS
│       └── src-tauri/     # Rust backend
│           └── src/
│               ├── main.rs
│               └── commands.rs
├── crates/
│   ├── core/              # Config, process manager, telemetry
│   ├── miner_adapters/    # XMRig and other miner adapters
│   └── pools/             # Pool config, health check
├── assets/
│   └── coins/             # Coin definition JSONs
└── docs/                  # Documentation
```

## Directory Conventions
- `apps/`: Deployable applications
- `crates/`: Shared Rust libraries
- `assets/`: Static resources (coin definitions)
- `docs/`: User and developer documentation
- Components: PascalCase (e.g., `Sidebar.tsx`)
- Rust modules: snake_case (e.g., `miner_adapters`)
