# Experiments

## Active Experiments

### Local Wasm LLM Redaction
- **Hypothesis**: We can run a small ONNX or Wasm-based language model (like LLaMA-3-8B-Instruct via WebGPU) inside the Svelte client to automatically detect and redact names, SSNs, and street addresses in draft sections without external API connections.
- **Method**: Embed `transformers.js` in Svelte and load a quantized model locally from a directory.
- **Success Criteria**: Detects at least 90% of sensitive PII locally in drafts while keeping execution time under 3 seconds.
- **Status**: Running.

---

## Completed Experiments

### Typst vs Headless Chrome
- **Hypothesis**: The Svelte frontend can compile PDF disclosure statements using a headless browser, but Tauri binary size increases dramatically.
- **Result**: Including a headless browser package inflated the Windows installer size to 110MB and slowed compilation to ~3 seconds. Switching to the Rust Typst crate reduced installer size to 15MB and completed compiles in under 200ms.
- **Decision**: Pinned all PDF reports to Rust Typst engine compilation in `pdf.rs`.

---

## Ideas to Test
- **Encrypted USB Auto-Backup**: Testing if Tauri can automatically detect when a specific USB drive is inserted and perform an incremental backup of the SQLite databases.
- **Audio Transcript Scrubbing**: Testing local transcription (Whisper.cpp) for witness recordings to extract timelines automatically.
