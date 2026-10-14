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

This software is a formatting engine, not a lawyer. Every export tells the user to review the file and to consider consulting an attorney. It is not legal advice.

## What it is not

- It will not write your arguments or find your citations.
- It will not send Confidential cases to justlegal.me or anywhere else.
- Account sync for standard cases is planned and stubbed, not built.
- Complaint and motion builders are basic templates, not full drafters.

## Technology

- Desktop: Tauri v2 (Windows `.msi` / `.exe` and macOS `.dmg` are the intended bundles)
- UI: Svelte 5, TypeScript, Tailwind CSS
- Backend: Rust, SQLite per confidential hunt, XChaCha20Poly1305 + Argon2id
- Documents: Typst for PDF, Office Open XML for Word
- No telemetry and no new cloud services

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
