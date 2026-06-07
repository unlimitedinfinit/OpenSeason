# AI Agent Guide

## Never Touch
- **Cryptographic Key Derivation & Cipher Methods**: Do not change the Argon2id parameters or the XChaCha20Poly1305 nonce/encryption logic in `crypto.rs`.
- **Memory Zeroization**: Memory scrubbing via the `Zeroize` crate on session keys and password bytes is critical; do not bypass or remove these annotations.
- **Legal Airlock**: The full-screen blocking modal in the frontend is a non-negotiable legal compliance layer.

## Safe to Refactor
- **Svelte UI Styling**: The layouts, CSS, and Tailwind classes in the frontend wizard and dashboard.
- **USAspending API Mapping**: The parsing of JSON values from the API in `usaspending.rs` to add support for more contract award types.

## Requires Human Approval
- **Tauri capabilities config**: Modifying `src-tauri/capabilities/main.json` or changing target directories outside `$HOME/.open-season/**`.
- **Typst Report Templates**: Modifying the formatting template in `pdf.rs` that generates the final disclosure PDF statement.

## Project Rules
- **No External Syncing**: Never add packages or code that connects to any cloud database, telemetry service, or synchronization API. The app must remain local-first.
- **EXIF & Chunk Stripping**: All image files uploaded as evidence must be run through the Rust chunk-scrubbing helper to remove EXIF metadata.
- **Svelte 5 Runes**: Always use Svelte 5 reactive runes (`$state`, `$derived`, `$effect`) rather than legacy Svelte 4 reactivity patterns.

## Common Pitfalls
- **PowerShell Compatibility**: Avoid using the `&&` statement separator in Windows terminal commands (use `;` or split commands instead).
- **Session State Bypassing**: Do not store the Master Password or derived session keys to files or local storage. They must live in Tauri AppState memory only.

## Preferred Patterns
- **Tauri Commands**: Keep command signatures in `commands.rs` clean and return `Result<T, String>` to handle errors gracefully on the frontend.
- **Scoped SQLite connections**: Open and close SQLite connections per-command using the multi-vault directory structure rather than maintaining long-lived locks.
