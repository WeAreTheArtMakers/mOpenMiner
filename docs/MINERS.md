# Miner Software Guide

OpenMineDash orchestrates external miner binaries. This document explains which miners are used for different algorithms.

## Supported Miners

### XMRig (Primary)

XMRig is the primary miner for CPU-optimized algorithms.

**Supported Algorithm Families:**
- RandomX (Monero, Wownero, Arqma, etc.)
- CryptoNight (legacy coins)
- Argon2 (Chukwa variants)
- GhostRider

**License:** GPL-3.0 with linking exception  
**Source:** https://github.com/xmrig/xmrig

**Installation:**
1. Download from [XMRig Releases](https://github.com/xmrig/xmrig/releases)
2. For macOS: Allow in System Settings → Privacy & Security
3. Place in `~/Library/Application Support/openminedash/bin/xmrig`
4. Or set custom path in Settings

### cpuminer-opt (Secondary)

cpuminer-opt handles algorithms not covered by XMRig, primarily SHA-256d and Scrypt.

**Supported Algorithms:**
- SHA-256d (Bitcoin, Bitcoin Cash) - ASIC dominated
- Scrypt (Litecoin, Dogecoin) - ASIC dominated
- X11, X13, X16r, X17 series
- Lyra2 variants
- Yescrypt/Yespower
- Many others (see full list below)

**License:** GPL-2.0  
**Source:** https://github.com/JayDDee/cpuminer-opt

**Important:** cpuminer-opt is GPL-2.0 licensed. OpenMineDash runs it as a separate process (sidecar) to comply with licensing requirements.

**Installation (macOS):**

cpuminer-opt does not provide official macOS binaries. You must compile from source:

```bash
# Install dependencies
brew install automake autoconf openssl@3

# Clone repository
git clone https://github.com/JayDDee/cpuminer-opt.git
cd cpuminer-opt

# Configure for macOS
./autogen.sh
CFLAGS="-O3 -march=native" ./configure --with-crypto=/opt/homebrew/opt/openssl@3

# Build
make -j$(sysctl -n hw.ncpu)

# Copy binary
mkdir -p ~/Library/Application\ Support/openminedash/bin
cp cpuminer ~/Library/Application\ Support/openminedash/bin/cpuminer-opt
```

## Algorithm Routing

OpenMineDash automatically selects the appropriate miner:

| Algorithm | Miner | Practical? |
|-----------|-------|------------|
| RandomX | XMRig | ✅ Yes |
| CryptoNight | XMRig | ✅ Yes |
| Argon2 | XMRig | ✅ Yes |
| GhostRider | XMRig | ✅ Yes (RTM) |
| VerusHash | XMRig | ✅ Yes (VRSC) |
| SHA-256d | cpuminer-opt | ⚠️ ASIC dominated |
| Scrypt | cpuminer-opt | ⚠️ ASIC dominated |
| X11/X16r | cpuminer-opt | ⚠️ Low hashrate |
| Ethash | External GPU | ❌ GPU only |
| KawPoW | External GPU | ❌ GPU only |

## "Try Mining Anyway" Mode

For algorithms dominated by ASICs or GPUs, OpenMineDash offers a "Try Mining Anyway" mode:

- **Purpose:** Educational/experimental CPU mining
- **Warning:** Hashrate will be extremely low
- **No profit expected:** This is for learning, not earning

When enabled:
1. The app routes to cpuminer-opt if the algorithm is supported
2. Shows clear warnings about impracticality
3. Displays hashrate and shares (even if negligible)

## Telemetry & Stats (Best-Effort)

### XMRig
XMRig provides a built-in HTTP API for real-time stats. OpenMineDash queries this API for accurate hashrate, shares, and uptime data.

### cpuminer-opt
cpuminer-opt does not have a built-in API. OpenMineDash uses **best-effort log parsing** to extract metrics:

- **Hashrate:** Parsed from stdout patterns like `1.5 kH/s`, `Total: 2.0 MH/s`
- **Shares:** Parsed from patterns like `accepted: 5/6`, `yay! (10)`
- **Difficulty:** Parsed from `diff: 1.5` patterns

**Important Notes:**
- cpuminer-opt output format may vary between versions
- If hashrate cannot be detected, UI shows "hashrate unknown"
- Logs are always displayed regardless of parsing success
- Rolling average calculated over 60-second window

## Binary Verification

All miner binaries are verified using pinned SHA-256 checksums:

- `assets/checksums/xmrig.json`
- `assets/checksums/cpuminer-opt.json`

Checksums are embedded at build time. Remote hash fetching is disabled for security.

## Full cpuminer-opt Algorithm List

```
sha256d, scrypt, x11, x13, x14, x15, x16r, x16rv2, x16s, x17,
x21s, x22i, x25x, lyra2v2, lyra2v3, lyra2z, lyra2h, yescrypt,
yescryptr8, yescryptr16, yescryptr32, yespower, yespowerr16,
allium, blake, blake2b, blake2s, groestl, heavy, keccak, lbry,
neoscrypt, nist5, phi1612, phi2, quark, qubit, skein, skein2,
tribus, whirlpool
```

See [cpuminer-opt wiki](https://github.com/JayDDee/cpuminer-opt/wiki) for the complete list.
