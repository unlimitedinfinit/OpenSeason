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

- Legal Airlock is required.
- Session-only master password derives the encryption key.
- Evidence is encrypted with XChaCha20Poly1305.
- Storage is under the app data `vaults/` directory, not a cloud folder.
- Every sync, cloud backup, and publish call is refused in the Rust backend, even if a future UI forgets to hide the button.

A standard case can be converted to Confidential. That sets `mode` to `confidential` and writes `sealed_at`. After that, the case can never be treated as syncable. Confidential does not convert back. If you need a copy, use a deliberate local export (Word, PDF, or `.osb`).

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
