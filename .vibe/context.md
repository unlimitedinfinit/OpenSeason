# Context

## Current Focus
- Establishing the `.vibe` semantic layer and operational documentation framework for Open Season.
- Syncing existing case vaults and exporting the completed files to the Vibes Hub.

## Blockers
- None.

## Next Up
- Refactor dashboard PDF generation invokes to pull actual evidence counts and valuations from the active SQLite database instead of hardcoded demo values.
- Add import validation checks to verify `.osb` file integrity and key validity before unpacking.

## Active Experiments
- Investigating local LLM integration (like WebGPU or Wasm-based Llama) for automated redacting of sensitive names or addresses in evidence documents.

## Recent Wins
- Implemented core zero-trust cryptographic engine (Argon2id + XChaCha20Poly1305) with automatic memory zeroization.
- Wrote image EXIF and chunk metadata strip utilities in Rust backend.
- Structured Svelte 5 routing layout and page wizard using state runes.
- Fully integrated the Typst dynamic compiler in the Rust backend to generate PDF disclosure statements.

## Open Questions
- Should we add a mechanism to easily redact PDF files in-app, or keep it strictly to text sections and image metadata stripping?
