# Tech Stack

## Languages & Frameworks
- TypeScript (React frontend)
- Rust (Tauri backend + crates)
- Tailwind CSS (styling)

## Build System
- pnpm (Node package manager)
- Cargo (Rust build)
- Vite (frontend bundler)
- Tauri (desktop packaging)

## Common Commands
```bash
# Install dependencies
pnpm install

# Development
pnpm tauri dev

# Build for production
pnpm tauri build

# Lint
pnpm lint
cargo clippy --workspace

# Format
cargo fmt --all

# Test
cargo test --workspace
pnpm test

# Type check
pnpm typecheck
```

## Dependencies

### Frontend
- react, react-dom: UI framework
- @tauri-apps/api: Tauri IPC
- zustand: State management
- clsx: Class name utility
- tailwindcss: Styling

### Backend (Rust)
- tauri: Desktop framework
- tokio: Async runtime
- serde/serde_json: Serialization
- reqwest: HTTP client
- sha2/hex: Checksum verification
- thiserror: Error handling
- tracing: Logging
