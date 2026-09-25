# AGENTS.md

Read this file first before creating, editing, or exporting any OpenSeason case.

OpenSeason is a **formatting and assembly engine**, not a research engine. It turns a person's own draft into a court-shaped Word or PDF file. It does not practice law.

## Hard rules

1. **Never invent arguments, facts, or citations.** If the user did not supply a fact, date, party name, docket number, statute, case name, or quotation, leave it blank and report it as missing. Do not guess.
2. **Only format and assemble the user's own material.** You may move their words into a caption, signature block, or template. You may not improve, expand, or "complete" their legal theory.
3. **Do not fill placeholders with made-up text.** If you see `[PLAINTIFF]`, `{{docket}}`, `TODO`, `TBD`, or similar markers, stop and ask the user for the real value.
4. **Every export is the user's document.** Remind them to review every line before filing. The software is not legal advice. Consider consulting a licensed attorney.
5. **Confidential cases never go to the network.** Do not call sync, cloud backup, publish, email, or any upload API for a case whose mode is `confidential` or whose `sealed_at` field is set. Local Word/PDF export and local `.osb` export are allowed when the user asks.
6. **No telemetry.** Do not send case contents, prompts, or file paths to a remote logger.

## What you may do

- Create a standard case folder.
- Fill `case.json` with values the user provided.
- Copy the user's draft into `notice_of_appeal.user_text`.
- Validate the folder and report missing fields.
- Export a Notice of Appeal to Word and PDF after validation passes.
- Point the user at `templates/court-rules/` for formatting data. Rules are data files, not hard-coded court advice.

## Command line

From the repository root (Rust toolchain required):

```bash
# Create a case folder
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- case create \
  --path ./my-case \
  --title "Example v. Sample County Clerk" \
  --mode standard \
  --kind notice_of_appeal

# Check missing fields and leftover placeholders
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- case validate ./my-case

# Export Notice of Appeal (Word + PDF)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- export notice-of-appeal ./my-case

# Seal a standard case (cannot be reversed)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- case seal ./my-case

# Sync stub: standard cases return "not implemented"; confidential cases are refused
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- sync ./my-case --action backup
```

There is also `npm run cli -- ...` if you prefer that wrapper.

## Case folder layout

Portable on Windows and macOS. Default location for standard cases:

`Documents/JustLegal/Cases/{caseId}/`

```
case.json          # fill-once profile
orders/
filings/
evidence/
drafts/
court-rules/       # formatting JSON, including generic.json
exports/           # generated .docx and .pdf
```

Confidential cases also live in the app vault (`vaults/{id}/`) after sealing. Do not copy a sealed vault to a cloud drive.

## Sample data

Use only obviously fake names in tests and examples (Jordan Example, Sample County Clerk, docket 24-01000). Never paste real litigants or real dockets into fixtures.

## Product boundaries

- Home screen builders for complaints and motions are planned, not shipped.
- justlegal.me account sync is planned for standard cases only, and is stubbed.
- USAspending lookup is a public-records search used by Confidential mode. It does not upload the case folder.
