---
name: New Coin Plugin
about: Submit a new coin/algorithm for inclusion
title: '[COIN] '
labels: coin-plugin
assignees: ''
---

## Coin Information

- **Name**: 
- **Symbol**: 
- **Algorithm**: 
- **Website**: 

## Mining Method

- [ ] CPU mineable (XMRig compatible)
- [ ] External ASIC only
- [ ] Custom miner required

## Default Pools

Please provide at least 2 pools with the following information:

### Pool 1
- **Name**: 
- **Stratum URL**: `stratum+ssl://...` or `stratum+tcp://...`
- **TLS**: Yes/No
- **Region**: 

### Pool 2
- **Name**: 
- **Stratum URL**: 
- **TLS**: Yes/No
- **Region**: 

## Pool Verification Checklist

- [ ] Pool has been operational for at least 6 months
- [ ] Pool uses HTTPS for web interface
- [ ] Pool has public uptime/status page
- [ ] Pool is listed on miningpoolstats.stream or similar

## JSON Definition

```json
{
  "schema_version": 1,
  "id": "coin-id",
  "name": "Coin Name",
  "symbol": "SYMBOL",
  "algorithm": "algorithm",
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
  "notes": null,
  "trusted": false
}
```

## Additional Notes

Any special considerations, warnings, or notes for users.
