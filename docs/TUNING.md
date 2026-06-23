# Performance Tuning Guide

## Monero (XMR) on Apple Silicon

### Understanding RandomX

RandomX is Monero's mining algorithm, designed to be CPU-friendly. On Apple Silicon (M1/M2/M3), performance varies based on:

- Core count (performance vs efficiency cores)
- Memory bandwidth
- Thermal throttling

### Performance Presets

| Preset | CPU Usage | Fan Noise | Best For |
|--------|-----------|-----------|----------|
| Eco | ~25% | Silent | Background mining, laptop on battery |
| Balanced | ~50% | Low | Daily use while mining |
| Max | ~75% | Moderate | Dedicated mining sessions |

### Manual Tuning (Advanced)

In Settings → Binary Path, you can use a custom XMRig with additional flags:

```bash
# Example: Custom thread count
./xmrig -t 4 ...

# Example: CPU affinity (specific cores)
./xmrig --cpu-affinity 0xF ...

# Example: Lower priority
./xmrig --cpu-priority 1 ...
```

### M1 Pro/Max Specific Tips

1. **Use Balanced preset** - M1 chips throttle aggressively at high temps
2. **Monitor Activity Monitor** - Watch for thermal pressure
3. **Avoid Max preset on battery** - Drains quickly and throttles

### Huge Pages (Limited on macOS)

Unlike Linux, macOS doesn't support huge pages for RandomX. This means:
- ~10-15% lower hashrate vs Linux on same hardware
- No tuning available for this

### Expected Hashrates (Approximate)

| Chip | Eco | Balanced | Max |
|------|-----|----------|-----|
| M1 | ~400 H/s | ~800 H/s | ~1200 H/s |
| M1 Pro | ~600 H/s | ~1200 H/s | ~1800 H/s |
| M1 Max | ~800 H/s | ~1600 H/s | ~2400 H/s |
| M2 | ~450 H/s | ~900 H/s | ~1350 H/s |
| M3 | ~500 H/s | ~1000 H/s | ~1500 H/s |

*Actual results vary based on cooling, background apps, and silicon lottery.*

### Benchmark Feature (Coming Soon)

Future versions will include a 60-second benchmark to measure your specific hardware's performance.

### Thermal Management

If your Mac gets too hot:
1. Switch to Eco preset
2. Ensure good ventilation
3. Consider a laptop stand with airflow
4. Close other CPU-intensive apps

### Power Consumption

Rough estimates for M1 Pro:
- Eco: ~15W additional
- Balanced: ~25W additional  
- Max: ~40W additional

Calculate your electricity cost before mining long-term.
