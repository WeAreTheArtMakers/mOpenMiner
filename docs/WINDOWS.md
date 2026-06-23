# OpenMiner on Windows

Complete guide for running OpenMiner on Windows 10/11.

## System Requirements

### Minimum
- Windows 10 (64-bit) version 1809 or later
- 4GB RAM
- 2 CPU cores
- 500MB free disk space
- WebView2 Runtime

### Recommended
- Windows 11 (64-bit)
- 8GB+ RAM
- 4+ CPU cores
- 2GB free disk space
- Dedicated GPU (for some mining algorithms)

## Installation

### Option 1: Pre-built Binary (Easiest)

1. Download the latest release:
   - Visit: https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest
   - Download: `openMiner-windows-x64.zip`

2. Extract the ZIP file to a folder (e.g., `C:\OpenMiner`)

3. Install WebView2 Runtime (if not already installed):
   - Usually pre-installed on Windows 11
   - Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
   - Run the installer

4. Run `openminedash.exe`

### Option 2: Build from Source

1. Install prerequisites:

**Visual Studio Build Tools:**
```powershell
# Download from: https://visualstudio.microsoft.com/downloads/
# Install "Desktop development with C++"
```

**Node.js:**
```powershell
# Download from: https://nodejs.org/
# Install LTS version (20.x)
```

**pnpm:**
```powershell
npm install -g pnpm
```

**Rust:**
```powershell
# Download from: https://rustup.rs/
# Run rustup-init.exe
```

2. Clone and build:
```powershell
git clone https://github.com/WeAreTheArtMakers/mOpenMiner.git
cd mOpenMiner
pnpm install
pnpm tauri:build
```

3. Run:
```powershell
.\apps\desktop\src-tauri\target\release\openminedash.exe
```

## Mining on Windows

### Supported Miners

| Miner | Coins | Windows Support |
|-------|-------|-----------------|
| XMRig | XMR, VRSC | ✅ Excellent |
| cpuminer-opt | Various | ✅ Good |
| T-Rex | ETH, RVN (GPU) | ✅ Excellent |
| lolMiner | ETH, BEAM (GPU) | ✅ Excellent |

### XMRig Setup

1. Download XMRig for Windows:
   - Visit: https://github.com/xmrig/xmrig/releases
   - Download: `xmrig-X.X.X-msvc-win64.zip`
   - Extract to a folder (e.g., `C:\XMRig`)

2. Configure in OpenMiner:
   - Go to Settings → Binary Path
   - Select `xmrig.exe`
   - Configure pool and wallet

### Windows Defender Exclusions

Mining software often triggers false positives. Add exclusions:

1. Open Windows Security
2. Go to Virus & threat protection → Manage settings
3. Scroll to Exclusions → Add or remove exclusions
4. Add folder: `C:\OpenMiner`
5. Add folder: `C:\XMRig` (or wherever XMRig is installed)

**Important**: Only add exclusions for software you trust and downloaded from official sources.

## Firewall Configuration

### Allow Mining Pools

1. Open Windows Defender Firewall
2. Click "Advanced settings"
3. Inbound Rules → New Rule
4. Program → Browse to `openminedash.exe`
5. Allow the connection
6. Repeat for `xmrig.exe`

### Common Mining Ports

- XMR pools: 3333, 5555, 7777
- Stratum: 3333
- HTTP API: 8080 (XMRig)

## Performance Optimization

### CPU Mining

**Power Plan:**
```powershell
# Set to High Performance
powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c
```

**Disable CPU Parking:**
1. Download ParkControl: https://bitsum.com/parkcontrol/
2. Disable CPU core parking
3. Set minimum processor state to 100%

**Large Pages (Huge Pages):**
1. Run as Administrator:
```powershell
# Enable large pages
bcdedit /set increaseuserva 3072
```
2. Restart computer
3. Configure XMRig to use large pages

### GPU Mining

**NVIDIA:**
- Install latest drivers: https://www.nvidia.com/drivers
- Use MSI Afterburner for overclocking
- Typical settings: Core -200MHz, Memory +800MHz, Power 70%

**AMD:**
- Install latest drivers: https://www.amd.com/drivers
- Use AMD Radeon Software for tuning
- Enable Compute Mode in driver settings

### Thermal Management

**Monitor Temperatures:**
- CPU: Use HWMonitor or Core Temp
- GPU: Use MSI Afterburner or GPU-Z
- Keep CPU under 80°C, GPU under 70°C

**Improve Cooling:**
- Clean dust from fans and heatsinks
- Improve case airflow
- Consider additional case fans
- Repaste thermal compound if needed

## Auto-Start on Boot

### Method 1: Task Scheduler (Recommended)

1. Open Task Scheduler
2. Create Basic Task
3. Name: "OpenMiner"
4. Trigger: "When I log on"
5. Action: "Start a program"
6. Program: `C:\OpenMiner\openminedash.exe`
7. Finish

### Method 2: Startup Folder

1. Press `Win + R`
2. Type: `shell:startup`
3. Create shortcut to `openminedash.exe`
4. Place in Startup folder

### Method 3: Windows Service (Advanced)

Use NSSM (Non-Sucking Service Manager):

```powershell
# Download NSSM: https://nssm.cc/download
nssm install OpenMiner "C:\OpenMiner\openminedash.exe"
nssm start OpenMiner
```

## Remote Management

### Remote Desktop

Built-in Windows feature:

1. Enable Remote Desktop:
   - Settings → System → Remote Desktop → Enable
2. Connect from another PC:
   - Run `mstsc.exe`
   - Enter computer name or IP

### TeamViewer / AnyDesk

For remote access over internet:
- TeamViewer: https://www.teamviewer.com/
- AnyDesk: https://anydesk.com/

## Troubleshooting

### App Won't Start

**Error: "WebView2 Runtime not found"**
- Download and install: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

**Error: "VCRUNTIME140.dll not found"**
- Install Visual C++ Redistributable: https://aka.ms/vs/17/release/vc_redist.x64.exe

**Error: "Application failed to start (0xc000007b)"**
- Install both x86 and x64 Visual C++ Redistributables
- Ensure you're running 64-bit Windows

### Mining Won't Start

**XMRig not found:**
- Check binary path in Settings
- Ensure XMRig is not blocked by antivirus
- Run OpenMiner as Administrator

**Pool connection failed:**
- Check firewall settings
- Verify pool URL and port
- Test connectivity: `telnet pool.supportxmr.com 3333`

### Low Hashrate

**CPU Mining:**
- Close background applications
- Disable Windows Search indexing
- Set power plan to High Performance
- Enable large pages in XMRig

**GPU Mining:**
- Update GPU drivers
- Reduce overclock if unstable
- Check GPU temperature (thermal throttling)
- Ensure PCIe power cables are connected

### High CPU/GPU Temperature

**Immediate actions:**
- Reduce thread count or GPU power limit
- Improve case ventilation
- Clean dust from components

**Long-term solutions:**
- Upgrade CPU cooler
- Add case fans
- Repaste thermal compound
- Consider undervolting

## Security Best Practices

### Antivirus Configuration

1. Use Windows Defender (built-in)
2. Add exclusions only for trusted mining software
3. Download miners only from official sources
4. Verify checksums/signatures

### Network Security

1. Enable Windows Firewall
2. Use strong passwords
3. Keep Windows updated
4. Don't expose mining ports to internet

### Wallet Security

1. Use hardware wallets for large amounts
2. Never share private keys
3. Use strong, unique passwords
4. Enable 2FA on exchange accounts
5. Keep wallet software updated

## Monitoring & Management

### Built-in Tools

**Task Manager:**
- Press `Ctrl + Shift + Esc`
- Monitor CPU, GPU, RAM usage

**Resource Monitor:**
- Run `resmon.exe`
- Detailed resource usage

**Performance Monitor:**
- Run `perfmon.exe`
- Create custom monitoring dashboards

### Third-Party Tools

**Hardware Monitoring:**
- HWMonitor: https://www.cpuid.com/softwares/hwmonitor.html
- GPU-Z: https://www.techpowerup.com/gpuz/
- MSI Afterburner: https://www.msi.com/Landing/afterburner

**Mining Monitoring:**
- OpenMiner built-in dashboard
- XMRig HTTP API: http://localhost:8080
- Pool dashboard (varies by pool)

## Power Consumption

### Measuring Power Usage

**Hardware:**
- Kill A Watt meter: ~$20
- Smart plugs with power monitoring

**Software:**
- HWMonitor (estimates)
- GPU-Z (GPU power)
- Manufacturer utilities

### Typical Power Draw

| Component | Idle | Mining |
|-----------|------|--------|
| CPU (8-core) | 20W | 65-95W |
| GPU (RTX 3070) | 15W | 120-220W |
| System Total | 80W | 200-350W |

### Cost Calculation

```
Daily Cost = (Power in kW) × (Hours) × (Rate per kWh)
Example: 0.25 kW × 24h × $0.12 = $0.72/day
```

## Profitability

### Factors

1. **Hashrate**: Your mining speed
2. **Power Cost**: Electricity rate ($/kWh)
3. **Coin Price**: Current market value
4. **Network Difficulty**: Competition
5. **Pool Fees**: Usually 1-2%

### Calculators

- WhatToMine: https://whattomine.com/
- CryptoCompare: https://www.cryptocompare.com/mining/calculator/
- NiceHash: https://www.nicehash.com/profitability-calculator

### Realistic Expectations

**CPU Mining (8-core Ryzen):**
- XMR: ~5-8 KH/s
- Daily earnings: ~$0.50-1.00
- Monthly profit: $5-15 (after electricity)

**GPU Mining (RTX 3070):**
- ETH: ~60 MH/s (if still mineable)
- Daily earnings: Varies greatly
- ROI: 6-18 months (market dependent)

## Upgrading

### Check for Updates

1. Visit: https://github.com/WeAreTheArtMakers/mOpenMiner/releases
2. Download latest version
3. Extract and replace old files
4. Settings are preserved in `%APPDATA%\openminedash`

### Backup Settings

```powershell
# Backup config
copy "%APPDATA%\openminedash\config.json" "C:\Backup\"

# Restore config
copy "C:\Backup\config.json" "%APPDATA%\openminedash\"
```

## Uninstallation

### Complete Removal

1. Close OpenMiner
2. Delete installation folder (e.g., `C:\OpenMiner`)
3. Delete config folder:
```powershell
rmdir /s "%APPDATA%\openminedash"
```
4. Remove Task Scheduler entries (if created)
5. Remove firewall rules (if created)

## FAQ

**Q: Is mining legal in my country?**
A: Check local regulations. Mining is legal in most countries, but some have restrictions.

**Q: Will mining damage my hardware?**
A: Not if properly cooled and maintained. Keep temperatures in safe ranges.

**Q: Can I mine while gaming?**
A: Not recommended. Mining uses 100% of resources. Stop mining before gaming.

**Q: Why is my antivirus blocking the miner?**
A: False positive. Miners are often flagged. Add exclusions for trusted software only.

**Q: Can I mine on a laptop?**
A: Not recommended. Laptops have limited cooling and may overheat.

**Q: How much can I earn?**
A: Depends on hardware, electricity cost, and coin prices. Use profitability calculators.

## Community & Support

- GitHub Issues: https://github.com/WeAreTheArtMakers/mOpenMiner/issues
- Discord: [Coming Soon]
- Reddit: r/MoneroMining, r/gpumining

## License

MIT License - See [LICENSE](../LICENSE)
