# Architecture

## Systems

```
┌──────────────────┐           Tauri IPC            ┌────────────────────┐
│ Svelte 5 UI      │ ────────────────────────────> │ Tauri Rust Backend │
│ (Desktop Client) │ <──────────────────────────── │ ("The Armory")     │
└──────────────────┘                               └─────────┬──────────┘
                                                             │
                                           ┌─────────────────┼─────────────────┐
                                           ▼                 ▼                 ▼
                                    ┌─────────────┐   ┌─────────────┐   ┌──────────────┐
                                    │ SQLite DB   │   │ Local Files │   │ USAspending  │
                                    │ (per Hunt)  │   │ (Encrypted) │   │ (External)   │
                                    └─────────────┘   └─────────────┘   └──────────────┘
```

## How They Connect

| From | To | Protocol | Purpose |
|---|---|---|---|
| Svelte 5 UI | Tauri Rust Backend | Tauri IPC (Invokes/Events) | User action triggers, file loads, operations triggers |
| Tauri Rust Backend | SQLite DB | Rust SQLite driver | Save hunt metadata, events, parties, and complaint sections |
| Tauri Rust Backend | Local Files | OS File System API | Read/Write encrypted evidence files in target directories |
| Tauri Rust Backend | USAspending | HTTPS (reqwest) | Scouting target government contracts over $100k |

## Trust Boundaries

```
UNTRUSTED                         GATE                           TRUSTED
─────────────────────────────────────────────────────────────────────────────────
UI Input (Passphrase)       →   Argon2id Key Derivation   →   Session Key in Memory
External File Uploads       →   JPEG/PNG Chunk Stripper   →   Metadata-Scrubbed Bytes
Plaintext Evidence          →   XChaCha20Poly1305 Cipher  →   Encrypted Local Files
Outbound Search Query       →   reqwest TLS / API Scope   →   Target Verification Results
```
