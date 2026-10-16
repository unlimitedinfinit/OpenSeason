# AGENTS.md

Read this file first before creating, editing, or exporting any OpenSeason case.

OpenSeason is a **formatting and assembly engine**, not a research engine. It turns a person's own draft into a court-shaped Word or PDF file. It does not practice law.

## Hard rules

1. **Never invent arguments, facts, or citations.** If the user did not supply a fact, date, party name, docket number, statute, case name, or quotation, leave it blank and report it as missing. Do not guess.
2. **Only format and assemble the user's own material.** You may move their words into a caption, signature block, or template. You may not improve, expand, or "complete" their legal theory.
3. **Do not fill placeholders with made-up text.** If you see `[PLAINTIFF]`, `{{docket}}`, `TODO`, `TBD`, or similar markers, stop and ask the user for the real value.
4. **Every export is the user's document.** Remind them to review every line before filing. The software is not legal advice. Consider consulting a licensed attorney.
5. **Confidential cases never go to the network.** Do not call sync, cloud backup, publish, email, or any upload API if the folder has a `.sealed` marker, or if `mode` is `confidential`, or if `sealed_at` is set. Do not delete `.sealed` to "help" the user sync. Local Word/PDF export and local `.osb` export are allowed when the user asks. The optional USAspending lookup sends only the target name you type to `api.usaspending.gov`. It does not upload the case folder.
6. **No telemetry.** Do not send case contents, prompts, or file paths to a remote logger.

## What you may do

- Create a standard case folder.
- Fill `case.json` with values the user provided.
- Copy the user's draft into `notice_of_appeal.user_text`.
- Validate the folder and report missing fields.
- Export a Notice of Appeal to Word and PDF after validation passes.
- Point the user at `templates/court-rules/` for formatting data. Rules are data files, not hard-coded court advice.

## Command line (agent interface)

The CLI and MCP are a first-class surface, equal to the window. `--json` returns a structured envelope (`ok`, `issues`, `never_files`). Read [docs/AGENT-INTERFACE.md](docs/AGENT-INTERFACE.md). Call `openseason --json docs playbook` or MCP `playbook` before a motion. Skill file: [docs/skills/openseason-motion/SKILL.md](docs/skills/openseason-motion/SKILL.md). Example: `bash examples/agent-e2e.sh`. Fake sample: `openseason --json case seed --path ./tmp-lucille --template lucille`.

To figure out **what to file**, call `openseason --json guide questions` then `guide find --state NJ --situation reconsideration`. Recommend only forms the pack lists. If `citation_status` is not `pack_source`, say it needs verification. Do not invent a deadline. `docs links` / MCP `help_links` return JustLegal URLs as data — never open a browser. The window confirms first. Forms packs: [docs/FORMS-PACKS.md](docs/FORMS-PACKS.md).

From the repository root (Rust toolchain required):

```bash
# Create a case folder (NJ reconsideration)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- --json case create \
  --path ./my-case \
  --title "Example v. Sample County Clerk" \
  --mode standard \
  --kind reconsideration \
  --pack nj-app-div

# Import a Markdown draft (md2court subset)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- --json draft import ./my-case --md fixtures/sample-reconsideration.md

# Checks (never invent cites; never file)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- --json check all ./my-case \
  --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09

# Export court-formatted Word + PDF
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- --json export court-md ./my-case

# Legal OS 1-Call Context Bundle, Guided Intake Wizard, Predictive Next Steps & Venue Check
.\openseason-cli.exe --json case context ./my-case
.\openseason-cli.exe --json intake scan ./my-case
.\openseason-cli.exe --json intake status ./my-case
.\openseason-cli.exe --json intake answer ./my-case --id <question_id> --value "<user answer>"
.\openseason-cli.exe --json case next-steps ./my-case
.\openseason-cli.exe --json check venue ./my-case
.\openseason-cli.exe --json case escalate ./my-case --target appellate_appeal
.\openseason-cli.exe --json portfolio status

# Optional local MCP stdio server (same engine)
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- mcp

# Seal a standard case (writes .sealed; cannot be undone by editing case.json)
# CLI seal does not encrypt files. Use the desktop vault path for encryption.
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- case seal ./my-case

# Sync stub: standard cases return "not implemented"; confidential cases are refused
cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- sync ./my-case --action backup
```

There is also `npm run cli -- ...` or the pre-built `.\openseason-cli.exe` binary in the repo root.

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

- Motions and appeals are priority. Complaint, brief, and mandamus are basic templates or scaffolds.
- Confidential (False Claims vault) is one case type inside the builder.
- justlegal.me account sync is planned for standard cases only, and is stubbed.
- USAspending lookup is a public-records search used by Confidential mode. It does not upload the case folder.
- The tool never files or serves.
