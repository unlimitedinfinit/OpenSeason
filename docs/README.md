# Executive Summary

## Overview
Open Season is a local-first, zero-trust desktop workbench for False Claims Act investigations. It enables users to securely verify targets, strip metadata from evidence, document chronological events, and compile professional disclosure reports locally. By enforcing strict local-only execution and cryptographically protecting storage keys in memory, the application ensures whistleblower safety and protects the viability of legal claims.

The project is currently in the MVP stabilization phase. The core cryptographic engines, SQLite multi-vault storage, metadata scrubbers, and Typst report compilers have been implemented in Rust and Svelte 5.

## Documentation Map

| Document | Description |
|---|---|
| [topology.md](topology.md) | File and folder structure with descriptions |
| [architecture.md](architecture.md) | Systems, components, and data flow |
| [api.md](api.md) | API endpoints, request/response formats |
| [issues.md](issues.md) | Open bugs, blockers, known problems |
| [resolved.md](resolved.md) | Closed issues and their resolutions |
| [roadmap.md](roadmap.md) | Milestones, priorities, what's next |
| [developer_guide.md](developer_guide.md) | Setup, build, test, deploy instructions |
| [troubleshooting.md](troubleshooting.md) | Common errors and how to fix them |
| [glossary.md](glossary.md) | Project-specific terms defined |
| [decisions/](decisions/) | Architecture decision records |

## Quick Links

- **Current Milestone:** [see roadmap](roadmap.md)
- **Active Blockers:** [see issues](issues.md)
- **Semantic Layer:** [.vibe/](../.vibe/) — purpose, architecture intent, decisions rationale
