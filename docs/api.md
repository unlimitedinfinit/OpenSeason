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

Saving a sealed case as standard is rejected.

### `validate_case_cmd`

Returns `{ "ok": true|false, "issues": [{ "field", "message" }] }`.

### `export_notice_of_appeal_cmd`

Validates, then writes `.docx` and `.pdf` under `exports/`. Fails if the case is incomplete.

### `convert_case_to_confidential`

Requires an unlocked vault. Copies the folder into `vaults/{id}/`, encrypts leftover evidence files, writes a `SEALED.txt` stub in the Documents copy, and sets `sealed_at`. Cannot be reversed.

### `sync_case_cmd`

Arguments: `{ "caseId": "...", "action": "backup" | "publish" | "pull_account" }`

Never uploads. Returns a structured refusal:

- `confidential_forbidden` if the case is confidential or was ever sealed
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
