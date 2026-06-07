# Architecture

## Components

### Svelte Frontend ("Hunter's UI")
- **Responsibility**: Manages application layout state, captures user passwords, triggers wizards, processes file drag-and-drop events, and renders the Case details editor.
- **Key Files**: `src/routes/+page.svelte`, `src/routes/hunt/[id]/+page.svelte`, `src/lib/components/`
- **Depends On**: `@tauri-apps/api/core` for invoking IPC commands.
- **Depended On By**: None.

### Rust Backend ("The Armory")
- **Responsibility**: Conducts encryption/decryption, metadata stripping, SQLite initialization, PDF report compilation, and ZIP compression.
- **Key Files**: `src-tauri/src/commands.rs`, `crypto.rs`, `db.rs`, `bundle.rs`, `usaspending.rs`, `pdf.rs`
- **Depends On**: `chacha20poly1305`, `argon2`, `zeroize`, `typst`, `rusqlite`
- **Depended On By**: Svelte Frontend (via IPC handlers)

## Data Flow

### 1. Cryptographic Key Derivation
1. User enters master password in the UI launcher.
2. UI invokes Tauri `unlock_vault`.
3. Rust backend reads/generates the local vault salt.
4. Password and salt are passed into the Argon2id key derivation function.
5. The resulting 32-byte key is stored in the Tauri `AppState` state container (`Arc<Mutex<Option<SessionKey>>>`) in memory.
6. Crucially, raw password string and intermediate bytes are scrubbed via `Zeroize`.

### 2. Evidence Processing and Storage
1. User drops a file (e.g. image) into the evidence uploader.
2. UI invokes Tauri `add_hunt_evidence`.
3. Rust backend reads file bytes, detects JPG/PNG, and strips EXIF and text metadata chunks.
4. Rust backend retrieves the `SessionKey` from memory, generates a random 192-bit nonce, and encrypts the stripped bytes using XChaCha20Poly1305.
5. The encrypted file is saved inside the case's folder on disk under `evidence/`.
6. An entry including the file description, path, nonce, and computed SHA-256 hash is inserted into the hunt's isolated SQLite database.

## Local Storage Layout
Open Season stores all data in the system's local application data directory under `vaults/`:
- Windows: `C:\Users\<user>\AppData\Local\com.openseason.app\vaults\`
- Each hunt gets its own sub-folder containing:
  - `metadata.db` (Isolated SQLite database)
  - `evidence/` (Directory with encrypted files)
  - `disclosure_statement.pdf` (Compiled report)
