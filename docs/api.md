# API Reference

OpenSeason does not run a web server. The window talks to Rust through Tauri commands. The CLI calls the same library.

## Case commands (new)

### `get_cases_root`

Returns the standard-case directory (`Documents/JustLegal/Cases`).

### `list_cases`

Lists **standard** profiles found in that directory. Sealed stubs are omitted.

### `create_case`

Arguments: `{ "title": "Example v. Sample County Clerk", "documentKind": "notice_of_appeal" }`

Creates the folder layout and a `case.json`. Mode is standard.

### `get_case` / `save_case`

Read or write `case.json` by case id (or a full folder path).

`save_case` checks the on-disk `.sealed` marker, not only the incoming payload. A write that tries `{ "mode": "standard", "sealed_at": null }` on a sealed folder is rejected. Editing `case.json` by hand does not remove `.sealed`.

### `validate_case_cmd`

Returns `{ "ok": true|false, "issues": [{ "field", "message" }] }`.

### `export_notice_of_appeal_cmd`

Validates, then writes `.docx` and `.pdf` under `exports/`. Fails if the case is incomplete.

### `convert_case_to_confidential`

Requires an unlocked vault. Writes `.sealed` first, copies the folder into `vaults/{id}/`, encrypts evidence plus `case.json`, orders, filings, drafts, and exports (nonce prefixed on each file, decrypt-verified before plaintext is deleted). `metadata.db` and `court-rules/` stay readable. The Documents folder is replaced with a pointer that has the case id but not the title, court, or docket. CLI `case seal` only writes the marker. Cannot be reversed by editing `case.json`.

### `sync_case_cmd`

Arguments: `{ "caseId": "...", "action": "backup" | "publish" | "pull_account" }`

Never uploads. Returns a structured refusal:

- `confidential_forbidden` if `.sealed` exists or the loaded profile is confidential / has `sealed_at`
- `not_implemented` if the case is still standard

## Confidential vault commands (existing)

Still available: `get_salt`, `unlock_vault`, `lock_vault`, `is_locked`, hunt CRUD, evidence, timeline, parties, complaint sections, `verify_target_cmd`, `save_disclosure_cmd`, `.osb` import/export, `purge_vault_cache`.

`create_new_hunt` now initializes the full SQLite schema (info, events, parties, evidence, complaint_sections).

## CLI

Binary name: `openseason`

```
openseason case create --path DIR --title TEXT [--mode standard|confidential] [--kind notice_of_appeal]
openseason case validate DIR
openseason case seal DIR
openseason export notice-of-appeal DIR [--out DIR]
openseason sync DIR [--action backup|publish|pull]
```
