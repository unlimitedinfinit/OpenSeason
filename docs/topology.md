# Codebase Topology

## Directory Structure

```
open-season/
├── src/                    — Svelte 5 Frontend
│   ├── lib/
│   │   └── components/     — UI modals, Airlock, and Vault Launcher
│   └── routes/             — Application pages (Dashboard, Case detail)
├── src-tauri/              — Tauri Rust Backend
│   ├── capabilities/       — Tauri security scopes configuration
│   └── src/                — Core Rust logic modules ("The Armory")
├── static/                 — Static assets and assets
└── .vibe/                  — Semantic repository layer files
```

## File Inventory

| File | Lines | Language | Description |
|---|---|---|---|
| `src-tauri/src/commands.rs` | ~706 | Rust | Tauri command invoke handlers connecting UI to Rust modules |
| `src-tauri/src/crypto.rs` | ~195 | Rust | Key derivation (Argon2id), encryption (XChaCha20Poly1305), EXIF scrubbing |
| `src-tauri/src/db.rs` | ~76 | Rust | Isolated SQLite database schema definitions per Hunt vault |
| `src-tauri/src/pdf.rs` | ~170 | Rust | Reports generator compiling Case data into PDF via Typst library |
| `src-tauri/src/usaspending.rs` | ~155 | Rust | Scouting client fetching award contracts from api.usaspending.gov |
| `src-tauri/src/bundle.rs` | ~90 | Rust | Bug Out Bag protocol exporting/importing vaults to/from `.osb` zip files |
| `src/routes/+page.svelte` | ~310 | Svelte | Dashboard, Airlock disclaimer check, and Vault Unlock UI |
| `src/routes/hunt/[id]/+page.svelte` | ~1100 | Svelte | Main Case Vault workbench: scouting list, evidence checklist, timeline, and drafts |

## Dependency Map
- **Frontend** calls **Tauri Commands** (`src-tauri/src/commands.rs`) via Tauri IPC (`invoke`).
- **Tauri Commands** delegate to:
  - `crypto.rs` for file encryption, key unlocking, and EXIF scrubbing.
  - `db.rs` for writing events, evidence records, and complaint drafts to the SQLite db.
  - `bundle.rs` for zipping/unzipping `.osb` files.
  - `usaspending.rs` for querying contract awards.
  - `pdf.rs` for compiling Typst PDFs.
