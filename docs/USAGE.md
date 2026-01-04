# Usage Guide

## First Run

1. Launch OpenMineDash
2. Read and accept the consent dialog
3. The dashboard will appear with mining disabled

## Quick Start (Monero)

1. Go to **Dashboard**
2. Select **Monero (XMR)** from coin dropdown
3. Enter your **wallet address**
4. Choose a **pool** or enter custom stratum URL
5. Set **worker name** (optional, defaults to hostname)
6. Click **Start Mining**

## Dashboard

The main dashboard shows:
- **Status**: Running/Stopped indicator
- **Hashrate**: Current and average H/s
- **Shares**: Accepted / Rejected count
- **Uptime**: Time since mining started
- **System**: CPU usage, temperature

## Profiles

Save different configurations:
- **Home**: Full power, all threads
- **Office**: Low power, reduced threads
- **Custom**: Your own settings

To save a profile:
1. Configure your settings
2. Go to **Profiles**
3. Click **Save Current**
4. Enter profile name

## Pools

### Built-in Pools
Pre-configured pools for each coin with health status.

### Custom Pool
1. Go to **Pools**
2. Click **Add Custom**
3. Enter stratum URL (e.g., `stratum+ssl://pool.example.com:3333`)
4. Test connection
5. Save

### Health Check
- Green: Connected, low latency
- Yellow: Connected, high latency (>200ms)
- Red: Connection failed

## Logs

Real-time miner output with filtering:
- **INFO**: Normal operation messages
- **WARN**: Warnings (rejected shares, reconnects)
- **ERROR**: Errors requiring attention

## Settings

- **Theme**: Light / Dark / System
- **CPU Threads**: Number of threads for mining
- **CPU Priority**: Low / Normal
- **Start Minimized**: Launch to menu bar
- **Revoke Consent**: Disable all mining functionality

## External ASIC (BTC/LTC)

For Bitcoin and Litecoin:
1. Select BTC or LTC
2. App shows "External Miner Mode"
3. Configure your ASIC with the displayed pool settings
4. Use OpenMineDash to monitor pool-side stats (future feature)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+S | Start mining |
| Cmd+. | Stop mining |
| Cmd+, | Open settings |
| Cmd+L | Toggle logs |
| Cmd+Q | Quit (stops mining) |
