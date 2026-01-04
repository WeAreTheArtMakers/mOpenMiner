# Development Guide

## Prerequisites

- macOS 12+ (Apple Silicon recommended)
- Node.js 18+
- pnpm 8+
- Rust 1.70+ (via rustup)
- Xcode Command Line Tools

## Setup

```bash
# Clone
git clone https://github.com/user/openminedash.git
cd openminedash

# Install Node dependencies
pnpm install

# Install Rust dependencies (handled by Cargo)
cd apps/desktop/src-tauri
cargo build
cd ../../..
```

## Development

```bash
# Run in development mode
pnpm tauri dev

# Run frontend only (no Tauri)
pnpm --filter @openminedash/desktop dev

# Run Rust tests
cargo test --workspace

# Run TypeScript checks
pnpm typecheck

# Lint
pnpm lint
cargo clippy --workspace
```

## Project Structure

```
openminedash/
├── apps/desktop/          # Tauri application
│   ├── src/               # React frontend
│   └── src-tauri/         # Rust backend
├── crates/                # Shared Rust crates
│   ├── core/              # Core functionality
│   ├── miner_adapters/    # Miner integrations
│   └── pools/             # Pool management
├── assets/coins/          # Coin definitions
└── docs/                  # Documentation
```

## Adding a New Coin

1. Create `assets/coins/{symbol}.json`:
```json
{
  "id": "coin_id",
  "name": "Coin Name",
  "symbol": "SYMBOL",
  "algorithm": "algo_name",
  "recommended_miner": "xmrig|external-asic|custom",
  "cpu_mineable": true|false,
  "default_pools": [...],
  "notes": "Optional notes"
}
```

2. If custom miner needed, add adapter in `crates/miner_adapters/`

## Adding a New Miner Adapter

1. Create `crates/miner_adapters/src/{miner}.rs`
2. Implement `MinerAdapter` trait:
```rust
pub trait MinerAdapter {
    fn start(&self, config: &MiningConfig) -> Result<Child>;
    fn stop(&self, process: &mut Child) -> Result<()>;
    fn get_stats(&self) -> Result<MinerStats>;
    fn verify_binary(&self) -> Result<bool>;
}
```

3. Register in `crates/miner_adapters/src/lib.rs`

## Building for Release

```bash
# Build optimized release
pnpm tauri build

# Output: apps/desktop/src-tauri/target/release/bundle/
```

## Code Style

### TypeScript
- Strict mode enabled
- ESLint + Prettier
- Functional components with hooks

### Rust
- `cargo fmt` for formatting
- `cargo clippy` for lints
- Error handling with `thiserror`

## Testing

```bash
# All tests
pnpm test
cargo test --workspace

# Frontend unit tests
pnpm --filter @openminedash/desktop test

# Rust unit tests
cargo test -p openminedash-core
```

## Commit Messages

Format: `type: short description`

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
- `feat: add pool health check`
- `fix: handle XMRig crash gracefully`
- `docs: update README installation`
