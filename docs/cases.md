# Cases, modes, and folders

OpenSeason now has one engine and two doors.

## Standard mode

Use this for ordinary self-represented filings. The case lives in a normal folder you can copy to a USB drive.

Default folder (Windows and macOS):

`Documents/JustLegal/Cases/{caseId}/`

That parent path matches the JustLegalAid desktop layout described in the product notes (`~/Documents/JustLegal/Cases/{caseId}/`). Inside each case this app uses explicit subfolders so documents, evidence, and exports do not share one pile.

```
case.json
orders/
filings/
evidence/
drafts/
court-rules/
exports/
```

`case.json` is the fill-once profile: court, docket, parties, filer contact, and signature block. Later documents should read this file instead of asking again.

Standard cases **may** use account sync, cloud backup, or publish later. Those paths are stubbed in this build. The UI can show a "back up your case / create an account" note. Nothing is uploaded.

## Confidential mode

This is the existing Open Season whistleblower vault.

- Legal Airlock is required to open the vault in the desktop app.
- Session-only master password derives the encryption key.
- A desktop seal copies the folder into app data `vaults/{id}/`, then walks the whole tree (nested folders such as `evidence/photos/` and root-level user files included) and encrypts those files (nonce is stored on the file). `metadata.db` and `court-rules/` are left readable. That is a real limit, not full-disk encryption.
- Unlock stores a password verifier next to the salt. A wrong password is refused at unlock. Convert to Confidential asks for the password again and checks that verifier before any plaintext is deleted.
- CLI `case seal` writes a `.sealed` marker and updates `case.json`. It does **not** encrypt files. Use the desktop vault path when you need encryption.
- After the vault copy decrypt-verifies, the Documents folder is emptied and rewritten with only three items: `.sealed`, `SEALED.txt`, and a pointer `case.json` (schema, id, mode, document kind, sealed time; title, court, docket, parties, and notice text cleared). Root-level user files and extra folders are removed. This build has no in-app reader for sealed files.
- Sync, cloud backup, and publish are refused in Rust if `.sealed` exists, even if someone edits `case.json` back to `standard`.

A standard case can be converted to Confidential. That writes `.sealed` (not only fields inside `case.json`). Confidential does not convert back. If you need a copy, use a deliberate local export (Word, PDF, or `.osb`).

## Court rules

Formatting lives in data files, not in hard-coded court names.

- Default file: `templates/court-rules/generic.json`
- Copied into each new case as `court-rules/generic.json`
- Fields include paper size, margins, font, and line spacing
- `source_url` and `source_retrieved_on` are empty on the generic file so a later rules-fetch step can record where a real local-rule file came from

The generic file is a fallback. It is not a promise that a specific court will accept the layout.

## Notice of Appeal

The first finished document. It pulls caption, court, docket, and signature block from `case.json`, then merges the user's own notice text. Export writes both `.docx` and `.pdf` into `exports/`. Export fails if required fields are empty or if placeholders such as `[PLAINTIFF]` or `TODO` remain.

## Agent / CLI

See [AGENTS.md](../AGENTS.md). The `openseason` binary can create a folder, validate it, export the Notice of Appeal, seal a case, and hit the sync stub.
