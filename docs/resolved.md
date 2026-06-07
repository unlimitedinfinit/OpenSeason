# Resolved Issues

## Resolved

### JPEG & PNG EXIF Metadata Leakage
- **Resolved**: 2026-06-05
- **Fix**: Created specialized Rust chunk-scrubbers in `crypto.rs` that filter out `eXIf`, `tEXt`, `zTXt`, and `iTXt` blocks on png uploads, and strip the E1/FE segment markers on jpeg uploads before files get encrypted and written to disk.

### Single-Database Metadata Exposure
- **Resolved**: 2026-06-04
- **Fix**: Re-architected storage to a Multi-Vault model where each case receives its own isolated directory and separate SQLite database file (`metadata.db`), completely eliminating cross-case leakage risk.

### Tauri v2 Permission Scoping
- **Resolved**: 2026-06-03
- **Fix**: Configured permissions strictly under `src-tauri/capabilities/main.json` to lock file system reads, writes, and creations strictly to `$HOME/.open-season/**`.
