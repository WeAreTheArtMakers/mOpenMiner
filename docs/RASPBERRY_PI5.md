# OpenMiner on Raspberry Pi 5

Complete guide for running OpenMiner on Raspberry Pi 5 (8GB).

## Why Raspberry Pi 5?

The Raspberry Pi 5 is an excellent platform for mining management:
- **8GB RAM**: Sufficient for mining operations and GUI
- **ARM Cortex-A76 CPU**: 2-3x faster than Pi 4
- **64-bit OS**: Full support for modern mining software
- **Low Power**: ~15W power consumption
- **Always-On**: Perfect for 24/7 mining operations
- **Cost-Effective**: ~$80 for the board

## Hardware Requirements

### Minimum
- Raspberry Pi 5 (4GB RAM)
- 32GB microSD card (Class 10 or better)
- 5V/5A USB-C power supply (official Pi 5 PSU recommended)
- Active cooling (heatsink + fan)

### Recommended
- Raspberry Pi 5 (8GB RAM) ✅
- 64GB+ microSD card or NVMe SSD via PCIe
- Official Raspberry Pi 5 27W USB-C PSU
- Active cooling case (e.g., Argon ONE V3)
- Ethernet connection (more stable than WiFi for mining)

## Software Requirements

### Operating System
- **Raspberry Pi OS (64-bit)** - Recommended
  - Download: https://www.raspberrypi.com/software/
  - Choose "Raspberry Pi OS with desktop (64-bit)"
  - Use Raspberry Pi Imager for easy setup

### Alternative OS Options
- Ubuntu Desktop 23.10+ (ARM64)
- Manjaro ARM (XFCE or KDE)

## Installation

### Option 1: Pre-built Binary (Easiest)

1. Download the latest release:
```bash
wget https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest/download/openMiner-pi5-arm64.tar.gz
```

2. Extract:
```bash
tar -xzf openMiner-pi5-arm64.tar.gz
```

3. Install dependencies:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1
```

4. Run:
```bash
chmod +x openminedash
./openminedash
```

### Option 2: Build from Source

1. Install prerequisites:
```bash
# Update system
sudo apt-get update
sudo apt-get upgrade -y

# Install Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install pnpm
sudo npm install -g pnpm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Tauri dependencies
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

2. Clone and build:
```bash
git clone https://github.com/WeAreTheArtMakers/mOpenMiner.git
cd mOpenMiner
pnpm install
pnpm tauri:build
```

3. Run:
```bash
./apps/desktop/src-tauri/target/release/openminedash
```

## Mining on Raspberry Pi 5

### Supported Coins

| Coin | Algorithm | Hashrate (Pi 5) | Profitability |
|------|-----------|-----------------|---------------|
| Monero (XMR) | RandomX | ~1-2 KH/s | Low but viable |
| Verus (VRSC) | VerusHash | ~5-8 MH/s | Better |
| Raptoreum (RTM) | GhostRider | ~100-200 H/s | Low |

### XMRig Setup for Pi 5

1. Download XMRig for ARM64:
```bash
wget https://github.com/xmrig/xmrig/releases/download/v6.21.0/xmrig-6.21.0-linux-arm64.tar.gz
tar -xzf xmrig-6.21.0-linux-arm64.tar.gz
```

2. Configure in OpenMiner:
   - Go to Settings → Binary Path
   - Select the xmrig binary
   - Set thread count to 3-4 (leave 1 core for system)

### Performance Optimization

#### CPU Governor
```bash
# Set to performance mode
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

#### Cooling
- Monitor temperature: `vcgencmd measure_temp`
- Keep under 70°C for optimal performance
- Use active cooling (fan + heatsink)

#### Memory
```bash
# Check available memory
free -h

# Increase swap if needed (for 4GB models)
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Set CONF_SWAPSIZE=2048
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

#### Power Settings
```bash
# Disable screen blanking
sudo nano /etc/lightdm/lightdm.conf
# Add: xserver-command=X -s 0 -dpms
```

## Auto-Start on Boot

### Method 1: Desktop Autostart

1. Create autostart entry:
```bash
mkdir -p ~/.config/autostart
nano ~/.config/autostart/openminer.desktop
```

2. Add content:
```ini
[Desktop Entry]
Type=Application
Name=OpenMiner
Exec=/home/pi/openminedash
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
```

### Method 2: Systemd Service

1. Create service file:
```bash
sudo nano /etc/systemd/system/openminer.service
```

2. Add content:
```ini
[Unit]
Description=OpenMiner Dashboard
After=graphical.target

[Service]
Type=simple
User=pi
Environment=DISPLAY=:0
ExecStart=/home/pi/openminedash
Restart=on-failure

[Install]
WantedBy=graphical.target
```

3. Enable:
```bash
sudo systemctl enable openminer.service
sudo systemctl start openminer.service
```

## Monitoring

### System Resources
```bash
# CPU usage
htop

# Temperature
watch -n 1 vcgencmd measure_temp

# Network
iftop
```

### Mining Stats
- Use OpenMiner's built-in dashboard
- Check XMRig HTTP API: http://localhost:8080

## Troubleshooting

### App Won't Start

**Error: `error while loading shared libraries`**
```bash
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1
```

**Error: `cannot execute binary file`**
- Ensure you're using 64-bit Raspberry Pi OS
- Check: `uname -m` should show `aarch64`

### Low Hashrate

1. Check CPU governor:
```bash
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
```

2. Reduce thread count if thermal throttling occurs

3. Ensure active cooling is working

### High Temperature

```bash
# Check current temp
vcgencmd measure_temp

# Check throttling
vcgencmd get_throttled
```

If throttled (0x50000 or higher):
- Improve cooling
- Reduce thread count
- Lower CPU frequency

### Network Issues

```bash
# Test pool connectivity
ping pool.supportxmr.com

# Check firewall
sudo ufw status
```

## Performance Benchmarks

### Monero (XMR) Mining
- **Threads**: 4
- **Hashrate**: 1.2-1.8 KH/s
- **Power**: ~12W
- **Temperature**: 60-70°C (with active cooling)
- **Daily Earnings**: ~$0.05-0.10 (varies with XMR price)

### Power Efficiency
- Pi 5 + Mining: ~15W
- Comparable x86 system: ~65W+
- **4x more power efficient** than desktop mining

## Remote Access

### VNC (Built-in)
```bash
# Enable VNC
sudo raspi-config
# Interface Options → VNC → Enable
```

Access: `vnc://raspberrypi.local:5900`

### SSH
```bash
# Enable SSH
sudo raspi-config
# Interface Options → SSH → Enable
```

Access: `ssh pi@raspberrypi.local`

## Security Best Practices

1. **Change default password**:
```bash
passwd
```

2. **Update regularly**:
```bash
sudo apt-get update && sudo apt-get upgrade -y
```

3. **Enable firewall**:
```bash
sudo apt-get install ufw
sudo ufw allow ssh
sudo ufw enable
```

4. **Use strong wallet passwords**
5. **Keep private keys offline**

## Community & Support

- GitHub Issues: https://github.com/WeAreTheArtMakers/mOpenMiner/issues
- Raspberry Pi Forums: https://forums.raspberrypi.com/
- XMRig Support: https://github.com/xmrig/xmrig/issues

## FAQ

**Q: Is Pi 5 profitable for mining?**
A: Not significantly. It's better for learning, supporting the network, or as a low-power always-on node.

**Q: Can I mine Bitcoin on Pi 5?**
A: Technically yes, but completely impractical. Use ASIC miners for Bitcoin.

**Q: Will mining damage my Pi?**
A: No, if properly cooled. Keep temps under 70°C.

**Q: Can I use Pi 4?**
A: Yes, but expect 30-40% lower hashrate. Pi 5 is recommended.

**Q: Headless operation?**
A: OpenMiner requires a display server. Use VNC for remote access.

## License

MIT License - See [LICENSE](../LICENSE)
