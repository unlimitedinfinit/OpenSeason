# Developer Guide

## Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Node.js | 18+ | `npm install --legacy-peer-deps` (Svelte 5 peers disagree with a few UI packages) |
| Rust / Cargo | 1.75+ stable | [rustup.rs](https://rustup.rs) |
| VS Build Tools | 2022 C++ | Windows only |
| Xcode CLT | current | macOS only |

## Setup

```bash
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason
npm install --legacy-peer-deps
```

## Run

```bash
npm run tauri dev
```

## Test

```bash
cd src-tauri
cargo test
cd ..
npm run check
npm run build
```

CLI smoke test using the fake fixture:

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- case validate fixtures/sample-appeal-case
```

## Rules

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`).
- Zeroize passwords and session keys.
- Confidential data stays out of `sync.rs`.
- Court names and margins belong in JSON under `templates/court-rules/`, not in scattered string literals.
- Do not add analytics or new cloud SDKs.
- Docs and UI copy: no em dashes.

## Bundles

`npm run tauri build` is the intended path for Windows (`.msi` / `.exe`) and macOS (`.dmg`). That step needs the native WebView and code-signing toolchain. It was not run in the environment that produced this foundation.
