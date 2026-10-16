# OpenSeason

A free, privacy-first, offline-capable **Document Builder** (Offline Case Builder) for people who are representing themselves. It formats the user's own draft into a court-shaped Word or PDF file, then runs a check stage (record cites, authority status, deadline math, sign-off). It does not research the law, invent facts, file, or serve.

The older Open Season whistleblower vault is still here. It is **one Confidential case type** inside the same app, not the whole product.

Agents drive the same engine through `openseason --json` or `openseason mcp`. See [docs/AGENT-INTERFACE.md](docs/AGENT-INTERFACE.md).

## What it is

- **Standard cases**: a portable folder under `Documents/JustLegal/Cases/{caseId}/` with a fill-once `case.json` profile.
- **Confidential cases**: Legal Airlock plus a local vault. Unlock reads the salt from disk and checks a password verifier. After a desktop seal (password typed again), the vault copy encrypts the whole tree, including nested folders. The app's own `metadata.db` and `court-rules/` stay readable. Sync is refused. Documents is then emptied to `.sealed`, `SEALED.txt`, and a pointer `case.json`. This build has no in-app reader for sealed files. CLI `case seal` writes a `.sealed` marker so sync is refused, but does not encrypt files.
- **One case list**: Standard and Confidential cases share the home screen and the same dashboard. Existing vault hunts show up as Confidential cases.
- **Notice of Appeal**: finished Word and PDF export. Caption and signature come from the profile. The body is the user's own text.
- **Motions and appeals**: Notice of Appeal is ready. Reconsideration, enlargement, stay, amend findings, appellate brief, and appellate-court motion are working basic templates or scaffolds. New Jersey Appellate Division is the first rule pack (`templates/rule-packs/nj-app-div.json`).
- **Check stage**: fact-check (no invented cites), authority list (verified / unverified / not found), deadline calculator (weekend and holiday roll), sign-off checklist.
- **Mandamus**: Colorado C.A.R. 21 basic scaffold. The Mandamus-Colorado-09-2026 repo was not readable from this environment.
- **Complaint**: working basic template. Not a full interview.
- **Agent interface**: `AGENTS.md` plus the `openseason` command-line tool.
- **Forms packs**: catalog + optional install into `Documents/JustLegal/Packs/`. The 7 GB JustLegal tree is not in git. See [docs/FORMS-PACKS.md](docs/FORMS-PACKS.md).
- **What do I need to file?**: plain-language finder in the window and `openseason --json guide`. Teaching lives in packs. Legal information, not advice.
- **Get legal help**: Help menu links to JustLegal after a confirmation. CLI/MCP return the URLs only.

- **JustLegal Cloud (not implemented in this build)**: `openseason sync` for standard cases returns `not_implemented`. Confidential and sealed cases are refused. In-app login widgets exist; do not treat them as a working backup. Use local Word/PDF or `.osb` export.
- **Court Readiness Pre-Flight Audit**: Real-time validation checklist banner reviewing caption, docket number, filer signature, statutory deadline math, and fact-citation grounding before export.
- **Bates-Stamped Exhibit Packet Assembler**: One-click generation of unified Bates-stamped PDFs with custom prefixes (e.g. `ABRAMS-000001`) and automated Typst Table of Exhibits with SHA-256 integrity hashes.
- **Official Court Form (AcroForm) Filler**: Inspects PDF form fields and auto-maps `case.json` values into official court forms (e.g. Colorado JDF 205 fee waiver and JDF 1911 direct appeal), exporting filled PDFs and audit JSON.
- **Transcript Grounding & Indexer**: Indexes depositions and hearing transcripts line-by-line (`TRANSCRIPTS-AND-RECORD.md`) and cross-references assertions against the official record.
- **Multi-Motion & Multi-Filing Docket**: Manage multiple concurrent motions, stays, enlargements, and responses in a single case docket without overwriting drafts. 1-click Word & PDF export.
- **Certificate of Service & Proposed Order Generator**: Auto-fill opposing parties and generate court-mandated Certificates of Service (with multiple service methods) and judge-ready Proposed Forms of Order.
- **Offline "Zero-Leakage" PDF Power Station**: 100% private, pure-Rust PDF tools: Merge multiple court files, Split & extract page ranges, Rotate orientations, and Byte-level scrub SSNs, DOBs, phone numbers, and custom names.
- **Visual Legal Signature & Stamp Studio**: Electronic `/s/` formatting, formal script styles, and HTML5 stylus/mouse drawing pad for direct PDF stamping without cloud vendors.
- **3-Tier Legal Operating System (`Transactional`, `Administrative/Pre-Suit`, `Litigation`)**: Draft everyday instruments (Last Will & Testament with Self-Proving Affidavit, Residential Lease, Independent Contractor Agreement, Durable Power of Attorney), Pre-Suit Notices (Demand Letters, Cure Notices, Security Deposit Demands, OPRA/CORA/FOIA Requests), and Court Pleadings (including `pre_filing` complaints that render `Case No. __________ (To Be Assigned)`).
- **Multi-Venue Competence Matrix (`NJ`, `CO`, `FEDERAL`, `SCOTUS`)**: Enforces venue-specific sworn statement formulas (e.g. NJ R. 1:4-4(b) verbatim Certification in Lieu of Oath rejecting `"to the best of my knowledge"`, Colorado C.R.S. § 13-27-106 + mandatory C.R.C.P. 121 § 1-15(8) Certificate of Conferral, Federal 28 U.S.C. § 1746 declaration, SCOTUS Rule 14.1 Questions Presented, and Will 2-witness self-proving checks).
- **Predictive Procedural State Graph (`case next-steps`)**: Calculates downstream legal branches from timeline events and orders, modeling **NJ Pause/Toll (`pause_remaining` under R. 2:4-3(e))** vs. **Colorado/Federal Full Clock Reset (`reset_full` under FRAP 4(a)(4) / C.R.C.P. 59)** and **Mail Service Offset (+3 days)**.
- **1-Question-at-a-Time Guided Intake Wizard & Offline PDF Ingestion (`intake`)**: Extracts text offline from PDFs in `orders/` and `evidence/` (`lopdf`), surfaces candidate docket numbers/dates/courts with provenance, and walks users or AI agents through `next_question` prompts with filing-scoped draft storage (`drafts/sections-{filing_id}.json`).
- **Cross-Venue Case Escalation & Multi-State Portfolio Radar (`case escalate`, `portfolio status`)**: 1-click graduate a Tier 2 Pre-Suit Notice into a Tier 3 Lawsuit, or escalate a Trial case to Appellate / Mandamus / SCOTUS while automatically copying lower-court orders and filings into the new case's `evidence/` appendix, plus a Global Filer Profile (`Documents/JustLegal/filer.json`).
- **Ready-to-Run Standalone Binaries**: Pre-compiled `OpenSeason.exe` (desktop GUI) and `openseason-cli.exe` (CLI/MCP server) located directly in the root repository.
- **One-Click Launch Scripts**: `launch.ps1` (PowerShell) and `launch.bat` (batch file) for immediate menu or direct launching.

## Documentation
- [Executive Summary](docs/EXECUTIVE-SUMMARY.md): Comprehensive system overview and upgrade breakdown.
- [Cloud Sync & Airgap Architecture](docs/SYNC-AND-CLOUD.md): Full technical spec for Firebase integration and confidentiality protection.
- [Agent Interface](docs/AGENT-INTERFACE.md): CLI `--json` contract and MCP stdio tools.
- [Forms Packs](docs/FORMS-PACKS.md): State court rule packs and AcroForm catalogs.

This software is a formatting engine, not a lawyer. Every export tells the user to review the file and to consider consulting an attorney. It is not legal advice.

## What it is not

- It will not write your arguments or find your citations.
- It will not send Confidential or sealed cases to justlegal.me or anywhere else.
- It does not practice law, file, or serve documents.
- Complaint and motion builders are basic templates, not full drafters.

## Technology

- Desktop: Tauri v2 (`OpenSeason.exe`)
- CLI & MCP: Rust native binary (`openseason-cli.exe`)
- UI: Svelte 5, TypeScript, Tailwind CSS
- Backend: Rust, SQLite per confidential hunt, XChaCha20Poly1305 + Argon2id
- Documents: Typst for PDF, Office Open XML for Word, `pdf-forms` for AcroForms
- No telemetry; Confidential mode strictly airgapped

## Getting started

### Prerequisites

- Node.js 18 or later
- Rust (stable, via [rustup](https://rustup.rs/))
- Windows: Visual Studio Build Tools (C++ workload)
- macOS: Xcode Command Line Tools
- Linux (dev only): `build-essential`, WebKitGTK, and the usual Tauri system libraries

### Install and run

```bash
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason
npm install --legacy-peer-deps
npm run tauri dev
```

Windows owners: follow [docs/TESTING-ON-WINDOWS.md](docs/TESTING-ON-WINDOWS.md) for Visual Studio Build Tools, `npm run tauri build`, and a manual checklist.

### Tests and checks

```bash
cd src-tauri && cargo test
cd ..
npm run check
npm run build
```

Platform installers (`npm run tauri build`) need a full desktop toolchain. This repository does not ship those binaries.

### Command line (for a local AI agent)

The CLI is a first-class surface, equal to the window. Read [AGENTS.md](AGENTS.md) and [docs/AGENT-INTERFACE.md](docs/AGENT-INTERFACE.md). `--json` returns `{ ok, never_files, never_serves, issues, data }`. `openseason mcp` is the same engine over stdio.

```bash
bash examples/agent-e2e.sh
# or, one command at a time:
npm run cli -- --json case create --path ./my-case --title "Example v. Sample County Clerk" --kind reconsideration --pack nj-app-div
npm run cli -- --json draft import ./my-case --md fixtures/sample-reconsideration.md
npm run cli -- --json draft preview --md fixtures/sample-reconsideration.md
npm run cli -- --json check all ./my-case --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09
npm run cli -- --json export court-md ./my-case
```

A fake sample case lives at `fixtures/sample-appeal-case/` (Jordan Example v. Sample County Clerk). Do not put real people in fixtures.

## Project layout

```
src/                 Svelte 5 UI (home, wizard, unified case dashboard)
src-tauri/           Rust backend, CLI binary `openseason`
templates/           Court formatting rules and rule packs
fixtures/            Fake sample case and sample Markdown
examples/            Headless agent end-to-end script
docs/                Human docs, including the audit and AGENT-INTERFACE.md
AGENTS.md            Instructions any AI should read first
```

## Disclaimer

OpenSeason is a neutral formatting tool provided as-is. You are responsible for the accuracy of every filing and for following the rules of your court. Nothing in this app is legal advice.

Built for people who cannot or do not want to put a whole case in the cloud.
