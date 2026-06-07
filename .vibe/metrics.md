# Metrics

## North Star Metric
- **Local Successful Builds**: The number of local users successfully installing and building the native desktop app for their target operating system.

## Key Metrics

| Metric | Current | Target | Why It Matters |
|---|---|---|---|
| Binary Footprint | ~15MB | < 20MB | Keeps the app fast to distribute and run. |
| Key Derivation Time | ~600ms | < 800ms | Argon2id cost parameters balancing security vs user patience on unlock. |
| Report Generation Time | ~200ms | < 500ms | Speed of compiling complex SQLite database structures into a PDF via Typst. |
| Memory usage | ~45MB | < 60MB | Low memory footprint to keep the app lightweight and secure. |

## What We Track
- **Zero Remote Tracking**: Open Season does not include Google Analytics, Mixpanel, or any telemetry. We track nothing over the network to guarantee absolute whistleblower safety.
- **Local Console Logs**: Diagnostic outputs in debug mode (`api_debug.log`) for troubleshooting.

## What We Don't Track (But Should)
- **Anonymous local error counts**: We do not know when the app crashes or runs out of memory on a user's machine unless reported manually via GitHub issues.
