# Licensing Information

OpenMineDash is MIT licensed. However, it orchestrates third-party miner software with different licenses.

## OpenMineDash License

```
MIT License

Copyright (c) 2026 WATAM (We Are The Art Makers)

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

## Third-Party Miner Licenses

### XMRig

- **License:** GPL-3.0 with linking exception
- **Source:** https://github.com/xmrig/xmrig
- **Usage:** Run as separate binary (sidecar)

XMRig's license includes a linking exception that allows it to be used with proprietary software when run as a separate process.

### cpuminer-opt

- **License:** GPL-2.0
- **Source:** https://github.com/JayDDee/cpuminer-opt
- **Usage:** Run as separate binary (sidecar)

**GPL-2.0 Compliance:**

OpenMineDash complies with GPL-2.0 by:

1. **No linking:** cpuminer-opt is never linked into OpenMineDash. It runs as a completely separate process.

2. **Source availability:** The cpuminer-opt source code is available at:
   https://github.com/JayDDee/cpuminer-opt

3. **License notice:** This document serves as the required license notice.

4. **No modification:** OpenMineDash does not modify cpuminer-opt. Users obtain the unmodified binary separately.

5. **Attribution:** cpuminer-opt is credited in the application and documentation.

## Binary Distribution

OpenMineDash does NOT bundle miner binaries. Users must:

1. Download binaries from official sources
2. Or compile from source
3. Place binaries in the expected location
4. Verify checksums match pinned values

This approach:
- Respects upstream licenses
- Ensures users get authentic binaries
- Allows users to verify source code
- Avoids redistribution complications

## Coin Definition Plugins

Coin definition JSON files in `assets/coins/` are MIT licensed as part of OpenMineDash.

## Questions?

For licensing questions, contact: [email]

For GPL compliance concerns regarding cpuminer-opt, refer to:
https://www.gnu.org/licenses/gpl-2.0.html
