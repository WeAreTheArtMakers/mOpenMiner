# Contributing to OpenMineDash

Thank you for your interest in contributing!

## Development Setup

### Prerequisites
- macOS 12+ (Apple Silicon recommended)
- Node.js 18+
- pnpm 8+
- Rust 1.70+ (via rustup)
- Xcode Command Line Tools

### Local Development

```bash
# Clone
git clone https://github.com/user/openminedash.git
cd openminedash

# Install dependencies
pnpm install

# Run in development
pnpm tauri dev

# Run tests
cargo test --workspace
pnpm test
```

## Code Style

### TypeScript
- Strict mode enabled
- ESLint + Prettier formatting
- Functional components with hooks

### Rust
- `cargo fmt` for formatting
- `cargo clippy` for lints
- Error handling with `thiserror`

## How to Add a Coin Plugin

1. Create `assets/coins/{symbol}.json`:

```json
{
  "schema_version": 1,
  "id": "coin-id",
  "name": "Coin Name",
  "symbol": "SYMBOL",
  "algorithm": "randomx|sha256|scrypt",
  "recommended_miner": "xmrig|external-asic|custom",
  "cpu_mineable": true,
  "default_pools": [
    {
      "name": "Pool Name",
      "stratum_url": "stratum+ssl://pool.example.com:3333",
      "tls": true,
      "region": "global"
    }
  ],
  "notes": "Optional notes",
  "trusted": false
}
```

2. Validate against `assets/coins/schema.json`
3. Test with `pnpm tauri dev`
4. Submit PR

## Pull Request Guidelines

- One feature/fix per PR
- Include tests for new functionality
- Update documentation as needed
- Ensure CI passes
- Follow commit message format: `type: description`

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Security Requirements

All contributions must:
- Not introduce persistence mechanisms (launchd, login items)
- Not bypass consent checks
- Not obfuscate functionality
- Include security impact assessment for sensitive changes
- Validate all external inputs

## Testing

### Unit Tests
```bash
cargo test --workspace
```

### Integration Tests (with FakeMiner)
```bash
cargo test --workspace --features fake-miner
```

### Manual Testing Checklist
- [ ] Consent dialog appears on first run
- [ ] Mining starts only after explicit click
- [ ] STOP button terminates immediately
- [ ] App quit kills miner process
- [ ] Logs stream correctly
- [ ] Stats update from API

## Code of Conduct

Be respectful and constructive. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
