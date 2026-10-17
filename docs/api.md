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

Read or write `case.json` by case id. Hunt and case ids must be UUID text. Absolute paths and `../` ids are refused.

`save_case` checks the on-disk `.sealed` marker, not only the incoming payload. A write that tries `{ "mode": "standard", "sealed_at": null }` on a sealed folder is rejected. Editing `case.json` by hand does not remove `.sealed`.

### `validate_case_cmd`

Returns `{ "ok": true|false, "issues": [{ "field", "message" }] }`.

### `export_notice_of_appeal_cmd`

Validates, then writes `.docx` and `.pdf` under `exports/`. Fails if the case is incomplete.

### `convert_case_to_confidential`

Arguments: `{ "caseId": "<uuid>", "password": "<vault password>" }`

Requires an unlocked vault and a stored password verifier. The password is typed again and checked before any copy, encrypt, or delete. Then writes `.sealed`, copies the folder into `vaults/{id}/`, and encrypts the whole tree except the app's own `{vault}/metadata.db` and `{vault}/court-rules/` (nonce prefixed on each file, decrypt-verified before plaintext is deleted). User files named `metadata.db`, `.enc`, or `.gitkeep` in other folders are encrypted. After that, Documents is emptied to `.sealed`, `SEALED.txt`, and a pointer `case.json`. This build has no in-app sealed-file reader. CLI `case seal` only writes the marker. Cannot be reversed by editing `case.json`.

Hunt export refuses a target inside app data or the cases root, a file named `.sealed` (any case), and overwriting an existing file. Sealed vault delete and purge require the vault password.

### `sync_case_cmd`

Arguments: `{ "caseId": "...", "action": "backup" | "publish" | "pull_account" }`

Never uploads. Returns a structured refusal:

- `confidential_forbidden` if `.sealed` exists or the loaded profile is confidential / has `sealed_at`
- `not_implemented` if the case is still standard

## Confidential vault commands (existing)

Still available: `get_salt`, `unlock_vault`, `lock_vault`, `is_locked`, hunt CRUD, evidence, timeline, parties, complaint sections, `verify_target_cmd`, `save_disclosure_cmd`, `.osb` import/export, `purge_vault_cache`.

`create_new_hunt` now initializes the full SQLite schema (info, events, parties, evidence, complaint_sections).

## Document builder commands (shared with CLI)

The window and the CLI call the same functions (`check::run_all`, `court_md::import_markdown`, `documents::export_court_markdown`, `documents::export_pleading`).

### `list_rule_packs`

Returns bundled packs: `nj-app-div`, `co-mandamus`, `generic-us-appellate`.

### Forms packs and finder

`list_form_packs_cmd`, `install_form_pack_cmd`, `remove_form_pack_cmd`, `verify_form_pack_cmd`, `guide_questions_cmd`, `guide_find_cmd`, `help_links_cmd`. CLI: `packs list|install|remove|verify`, `guide questions|find`, `docs links`. There is no Tauri command that opens a browser; the UI confirms first. See [FORMS-PACKS.md](FORMS-PACKS.md).

### `import_markdown_cmd`

Arguments: `{ "caseId": "<uuid>", "markdown": "---\n..." }`

Writes `drafts/draft.md` and fills empty caption fields from YAML front matter.

### `check_case_cmd`

Arguments: `{ "caseId", "pack"?, "rule"?, "triggerDate"? }`

Runs fact-check, authority list, optional deadline, and sign-off. Returns a `CheckAllReport` with `never_files`, `never_serves`, `ok`, `issues`.

### `deadline_cmd`

Arguments: `{ "caseId", "pack", "rule", "triggerDate" }`

### `export_pleading_cmd`

`kind` may be `notice_of_appeal`, `complaint`, `motion`, `reconsideration`, `appellate_brief`, `appellate_motion`, `mandamus`, or `court_md`.

## Qui Tam & Fraud Investigation Commands (EO 14395)

### `quitam_verify_entity_cmd`
Arguments: `{ "name": "...", "npi"?: "...", "state"?: "..." }`  
Directly cross-checks entity against CMS NPPES v2.1 public registry and USAspending.gov prime award records. Detects deactivated NPIs, commercial mail drop practice locations, and multi-million dollar federal grant totals.

### `quitam_audit_9b_cmd`
Arguments: `{ "caseId": "..." }`  
Performs deterministic FRCP Rule 9(b) 6-element viability scoring (Who, What, When, Where, How, Scienter). Returns an objective 0–100 score and pinpoint checklist of missing statutory elements.

### `quitam_ingest_observation_cmd`
Arguments: `{ "caseId": "...", "filePath": "...", "notes": "...", "observer"?: "..." }`  
Ingests field photo/video observations. Extracts EXIF GPS coordinates and timestamp, computes dual SHA-256 and SHA-512 hashes, marks file read-only on disk, and appends to an immutable `provenance.json` Merkle-style log.

### `quitam_records_pack_cmd`
Arguments: `{ "caseId": "...", "state": "MN"|"NJ"|"CO"|"CA"|"FED", "agency"?: "..." }`  
Generates jurisdiction-specific licensing and public records request letters (MGDPA, OPRA, CORA, CPRA, FOIA).

### `quitam_export_disclosure_cmd`
Arguments: `{ "caseId": "..." }`  
Compiles the complete DOJ 60-Day Relator Written Disclosure Packet required under 31 U.S.C. § 3730(b)(2), generating `openseason-disclosure-v1.json` and `DOJ_Relator_Disclosure_Packet.pdf` in `exports/`.

## Diagnostic & Health Commands

### `doctor_diagnostic_cmd`
Runs an end-to-end environment and integrity check, verifying CLI versions, cases root, rule packs root, offline crypto readiness, airgap status, and EO 14395 qui tam engine availability.

## CLI and MCP (agent interface)

Binary name: `openseason`. `--json` prints the stable envelope documented in [AGENT-INTERFACE.md](AGENT-INTERFACE.md). `openseason mcp` is a local stdio MCP server with the same tools. Both are offline.

```
openseason --json case create --path DIR --title TEXT [--mode standard|confidential] [--kind reconsideration] [--pack nj-app-div]
openseason --json draft import DIR --md FILE
openseason --json evidence add DIR --file FILE [--description TEXT]
openseason --json check fact|authority|deadline|signoff|all DIR
openseason --json export court-md|pleading|notice-of-appeal DIR
openseason --json packs list
openseason mcp
openseason case validate DIR
openseason case seal DIR
openseason sync DIR [--action backup|publish|pull]
```

Example headless run: `bash examples/agent-e2e.sh`.
