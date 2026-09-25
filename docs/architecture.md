# Architecture

OpenSeason is one local engine with two front doors: a human wizard in the desktop window, and a command-line tool that a local AI can call. Both doors read the same case folder and the same Rust rules.

## Components

### Svelte frontend

- **Responsibility**: Home screen, case profile form, Notice of Appeal editor, Confidential vault UI (Legal Airlock, hunts, evidence).
- **Key files**: `src/routes/+page.svelte`, `src/routes/case/`, `src/routes/confidential/`, `src/routes/hunt/`
- **Depends on**: Tauri IPC (`invoke`)

### Rust backend

- **Responsibility**: Case folders, mode enforcement, document assembly, vault crypto, optional USAspending lookup.
- **Key files**: `src-tauri/src/case_profile.rs`, `sync.rs`, `documents.rs`, `court_rules.rs`, `case_commands.rs`, `crypto.rs`, `commands.rs`, `pdf.rs`
- **Depends on**: `rusqlite`, `chacha20poly1305`, `argon2`, `typst`, `zip`

### CLI (`openseason`)

- **Responsibility**: Create, validate, export, seal, and hit the sync stub without opening the window.
- **Key file**: `src-tauri/src/bin/openseason.rs`

### Court rules data

- **Responsibility**: Paper size, margins, font, line spacing. Not legal holdings.
- **Key file**: `templates/court-rules/generic.json`

## Modes

```
Standard case.json  -->  may call sync stub (returns not implemented)
                    -->  can convert to Confidential (writes .sealed plus sealed_at)
Confidential        -->  sync/backup/publish refused if .sealed exists
                    -->  cannot become standard by editing case.json
                    -->  local Word/PDF/.osb export still allowed
```

The UI is not the security boundary. `sync::request_sync_for_dir` checks the `.sealed` file first, then `case.json`. `save_case` refuses an unseal payload when that marker is present.

Desktop seal checks the vault password against a verifier, then encrypts the whole vault tree (nested folders and root-level user files included). Only the app's own `{vault}/metadata.db` and `{vault}/court-rules/` stay readable. Documents is then emptied to the pointer files. CLI `case seal` only writes the marker. Hunt and case command ids must be UUIDs. Export will not write inside app data or the cases root, will not write a file named `.sealed`, and will not use a relative, UNC, or symlink target. Repeat downloads get a numeric suffix and are created with O_EXCL. Sealed vault delete, purge, evidence delete, and other hunt edits require the vault password.

## Data flow: Notice of Appeal

1. User (or CLI) creates a folder and writes `case.json`.
2. `validate_case` checks required caption fields and leftover placeholders.
3. `documents::export_notice_of_appeal` merges profile + user text.
4. Word is written as Office Open XML (zip). PDF is compiled with Typst using the court-rules font list.
5. Files land in `exports/`. Both copies include the review notice.

## Data flow: Confidential vault (unchanged core)

1. User passes the Legal Airlock and unlocks with a master password.
2. Argon2id derives a 32-byte session key. An encrypted verifier next to `master_salt.bin` rejects a wrong password. The password itself is not stored.
3. Evidence is scrubbed, hashed, encrypted, and indexed in `metadata.db`.
4. Disclosure PDFs and `.osb` bundles stay on disk.

## Storage

| Kind | Location |
|---|---|
| Standard case | `Documents/JustLegal/Cases/{id}/` |
| Confidential hunt | App local data `vaults/{id}/` |
| Vault salt | App local data `master_salt.bin` |
| Password verifier | App local data `master_verifier.bin` |

Windows, macOS, and Linux resolve those roots through the `dirs` crate and Tauri path APIs. Paths are not hard-coded to one drive letter.

## Sync (stub)

`sync.rs` is the only place a future justlegal.me adapter should hook in. It currently returns `NotImplemented` for standard cases and `ConfidentialForbidden` for sealed ones. There is no HTTP client in this module.
