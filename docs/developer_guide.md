# Developer Guide

## Prerequisites

Before setting up the project, make sure you have the following installed:

| Tool | Version | Install |
|---|---|---|
| Node.js | v18+ (LTS preferred) | [nodejs.org](https://nodejs.org) |
| Rust / Cargo | v1.75+ (stable) | [rustup.rs](https://rustup.rs) |
| VS Build Tools | 2022 (C++ workload) | [visualstudio.com](https://visualstudio.microsoft.com/downloads/) (Windows only) |

## Setup

```bash
# Clone the repository
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason

# Install frontend dependencies
npm install
```

## Build & Development

```bash
# Run in development mode (launches Svelte frontend + compiles Rust backend dynamically)
npm run tauri dev

# Build the production executable (.msi / .exe on Windows)
npm run tauri build
```

## Test

```bash
# Run Svelte frontend TypeScript checks
npm run check

# Run Cargo test suite in backend
cd src-tauri
cargo test
```

## Project Rules
- **Runes for Reactivity**: Always use Svelte 5 state management runes (`$state`, `$derived`, `$effect`).
- **Memory Security**: All raw password strings and derived key arrays must implement `zeroize` upon completion.
- **Tauri Scopes**: File system reading and writing must occur strictly inside the sandbox directory `C:\Users\<user>\AppData\Local\com.openseason.app\vaults\`.

## Deployment
Open Season is distributed as a standalone desktop binary:
- **Windows**: Outputs to `src-tauri/target/release/bundle/msi/Open Season_0.1.0_x64_en-US.msi`
- **Mac/Linux**: Outputs to corresponding `.dmg` or `.deb` packages inside `src-tauri/target/release/bundle/`.
