# Roadmap

## Current Milestone: MVP Stabilization
**Goal**: Finalize core local-first features and fix the mock dashboard PDF parameters.

- [ ] Connect Svelte dashboard PDF generation to pull actual SQLite database totals instead of hardcoded numbers.
- [ ] Add file validation check during `.osb` imports to verify zip integrity.
- [ ] Stabilize dev build on Windows and Mac operating systems.

## Next Milestone: Security Hardening
**Goal**: Increase protection against forensic memory inspection and local snooping.

- [ ] Implement local SQLite database file-level encryption using SQLCipher (currently database files themselves are plaintext metadata, though files are encrypted).
- [ ] Add a secure self-uninstall option to purge all local vaults instantly.

## Completed Milestones

### Core Architecture & Cryptography
- **Completed**: 2026-06-05
- **Goal**: Establish the zero-trust local-first workspace structure.
- [x] Legal airlock block screen modal
- [x] Argon2id Key Derivation and XChaCha20Poly1305 file encryption
- [x] Multi-vault directories with SQLite indexing
- [x] Typst compiler integrated for disclosure report builder
- [x] EXIF/chunk metadata stripping from jpeg/png uploads
