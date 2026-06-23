# Changelog

All notable changes to OpenMiner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-06-23

### Added
- 🏊 **Pool Management UI**: Add/Remove pools directly from the Pools page
- 🗺️ **Pool Discovery**: Auto-discover pools via miningpoolstats.co.uk API with smart fallback
- 📋 **Audit Trail**: Every pool add/remove is logged with timestamp for security
- 💻 **CPU "Try Anyway" Mode**: GPU/ASIC algorithms can now be force-run on CPU for testing
- 🪙 **New Coin Plugins**: Dash (DASH), Zilliqa (ZIL) coin definitions added
- 🔄 **Manual Coin Reload**: "Reload" button to refresh pool list from disk without restart
- 🛡️ **Enhanced Pool Validation**: URL scheme, host:port, duplicate name/URL checks on backend
- 🌐 **Pool Discovery Fallback**: Hardcoded pool lists for 15+ coins when API is unavailable
- 🎯 **Import All / Single**: Discovered pools can be imported individually or all at once

### Changed
- ⚡ **Pool Management Refactored**: Pool operations now write directly to JSON files + update store
- 🔧 **Algo Routing Extended**: `External` miner type now routes to cpuminer-opt with `try_anyway`
- 🧩 **Schema Updated**: JSON schema now includes all algorithm types and external-gpu miner
- 📦 **Pools Crate**: New `discover_pools` and `discover_pools_with_fallback` functions
- 🎨 **Pools UI Redesigned**: Per-coin action buttons (Discover, Add Pool, Check All, Remove)

### Fixed
- 🐛 **"command not found" Bug**: `discover_pools`, `add_pool_to_coin`, `remove_pool_from_coin` Tauri commands not registered in handler
- 🐛 **Pool URL No Validation**: Added strict stratum+tcp/ssl/tls URL validation on backend
- 🐛 **Duplicate Pool Names**: Pool name case-insensitive duplicate check before adding
- 🐛 **React useState Anti-pattern**: `useState(fn)` replaced with `useEffect` in PoolDiscoveryDialog
- 🐛 **`&&str: IntoUrl` Compile Error**: Double reference in reqwest::Client::get call fixed

## [1.0.0] - 2026-05-28

### Added
- 🎛️ Single-panel dashboard for mining management
- 💰 Multi-coin support via plugin system (JSON definitions)
- 📊 Real-time stats via XMRig HTTP API
- 📝 Live log viewer with filtering
- 🔒 Pinned checksum verification for miner binaries
- 🌐 Pool health checking (TCP + TLS handshake)
- ⚡ Performance presets: Eco / Balanced / Max
- 🔧 Manual binary path option for enterprise environments
- 🍎 Native macOS support (Apple Silicon and Intel)
- 🎨 Modern, responsive UI with Tauri + React
- 🔐 Transparent security model with user consent
- 📦 Clean uninstall process
- 🌙 Dark mode support
- 🔔 System notifications for mining events
- 📈 Hashrate monitoring and statistics
- 🎯 System tray integration
- 🔄 Auto-update support (planned)

### Security
- ✅ No hidden mining - requires explicit user consent
- ✅ Default OFF - mining never starts automatically
- ✅ One-click stop - instant termination always available
- ✅ No auto-start - no launchd agents or background daemons
- ✅ Clean uninstall - removing the app removes everything
- ✅ Checksum verification for all downloaded binaries
- ✅ Open-source and auditable codebase

### Supported Coins
- Monero (XMR) - RandomX algorithm via XMRig
- Bitcoin (BTC) - Educational/experimental mode
- Litecoin (LTC) - Educational/experimental mode
- Dogecoin (DOGE) - Educational/experimental mode

### Platform Support
- macOS 12+ (Apple Silicon)
- macOS 12+ (Intel)

## [0.1.0] - 2026-05-01

### Added
- Initial project setup
- Basic Tauri + React architecture
- Core mining management functionality
- XMRig adapter implementation
- Pool configuration system
- Basic UI components

---

## Release Notes

### Multi-Platform Release (Upcoming)

This release brings OpenMiner to multiple platforms:

**Raspberry Pi 5 Support:**
- Native ARM64 Linux build
- Optimized for Pi 5's Cortex-A76 CPU
- Low power consumption (~15W)
- Perfect for 24/7 mining operations
- Comprehensive setup guide included

**Windows Support:**
- Native x86_64 Windows build
- WebView2 integration
- Windows Defender exclusion guide
- Performance optimization tips
- Auto-start configuration options

**Enhanced macOS Support:**
- Continued excellent performance on Apple Silicon
- Intel Mac support maintained
- Improved stability and performance

**Build System:**
- Automated cross-compilation from macOS
- GitHub Actions CI/CD pipeline
- Pre-built binaries for all platforms
- Easy distribution via GitHub Releases

**Documentation:**
- Platform-specific setup guides
- Performance tuning recommendations
- Troubleshooting sections
- Security best practices

### Migration Guide

**From v1.0.0 to v1.1.0:**
- No breaking changes
- Settings are automatically migrated
- Simply download and install the new version
- Your existing configuration will be preserved

### Known Issues

- [ ] Cross-compilation on Linux may require additional setup
- [ ] Windows Defender may flag mining binaries (false positive)
- [ ] Raspberry Pi 4 support is experimental (Pi 5 recommended)
- [ ] GPU mining support is planned for future releases

### Roadmap

**v1.2.0 (Planned):**
- [ ] GPU mining support (NVIDIA/AMD)
- [ ] More coin plugins (Verus, Raptoreum, etc.)
- [ ] Mining pool auto-switching
- [ ] Profit calculator
- [ ] Advanced statistics and charts

**v1.3.0 (Planned):**
- [ ] Mobile app (iOS/Android) for monitoring
- [ ] Cloud sync for settings
- [ ] Multi-rig management
- [ ] Telegram/Discord notifications

**v2.0.0 (Future):**
- [ ] Built-in wallet support
- [ ] Decentralized pool support (P2Pool)
- [ ] Mining marketplace integration
- [ ] Advanced automation and scheduling

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute to OpenMiner.

## License

MIT License - See [LICENSE](LICENSE)

## Credits

**Created by WATAM (We Are The Art Makers)**
- Website: https://wearetheartmakers.com
- GitHub: https://github.com/WeAreTheArtMakers

**Built with:**
- Tauri - Cross-platform desktop framework
- React - UI framework
- Rust - Backend language
- XMRig - Mining software

**Special Thanks:**
- XMRig developers for excellent mining software
- Tauri team for amazing cross-platform framework
- Raspberry Pi Foundation for affordable hardware
- Open-source community for continuous support
