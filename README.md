# OpenSeason

A free, privacy-first, offline-capable desktop builder for people who are representing themselves. It formats the user's own draft into a court-shaped Word or PDF file. It does not research the law and it does not invent facts.

The older Open Season whistleblower vault is still here. It is now **Confidential mode** of the same app.

## What it is

- **Standard cases**: a portable folder under `Documents/JustLegal/Cases/{caseId}/` with a fill-once `case.json` profile.
- **Confidential cases**: Legal Airlock plus a local vault. Unlock checks a password verifier stored next to the salt. After a desktop seal (password typed again), the vault copy encrypts the whole tree, including nested folders. `metadata.db` and `court-rules/` stay readable. Sync is refused. The Documents folder keeps a pointer `case.json` (id, mode, empty profile fields). This build has no in-app reader for sealed files. CLI `case seal` writes a `.sealed` marker so sync is refused, but does not encrypt files.
- **Notice of Appeal**: the first finished document. Caption and signature come from the profile. The body is the user's own text. Export is Word (`.docx`) and PDF.
- **Agent interface**: `AGENTS.md` plus the `openseason` command-line tool.

This software is a formatting engine, not a lawyer. Every export tells the user to review the file and to consider consulting an attorney. It is not legal advice.

## What it is not

- It will not write your arguments or find your citations.
- It will not send Confidential cases to justlegal.me or anywhere else.
- Account sync for standard cases is planned and stubbed, not built.
- Complaint and motion builders are planned for a later release.

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

### Tests and checks

```bash
cd src-tauri && cargo test
cd ..
npm run check
npm run build
```

Platform installers (`npm run tauri build`) need a full desktop toolchain. This repository does not ship those binaries.

### Command line (for a local AI agent)

Read [AGENTS.md](AGENTS.md) first. Then:

```bash
npm run cli -- case create --path ./my-case --title "Example v. Sample County Clerk"
# edit my-case/case.json with the user's real values and their own draft
npm run cli -- case validate ./my-case
npm run cli -- export notice-of-appeal ./my-case
```

A fake sample case lives at `fixtures/sample-appeal-case/` (Jordan Example v. Sample County Clerk). Do not put real people in fixtures.

## Project layout

```
src/                 Svelte 5 UI (home, case profile, Confidential vault)
src-tauri/           Rust backend, CLI binary `openseason`
templates/           Court formatting rules as data files
fixtures/            Fake sample case
docs/                Human docs, including the audit
AGENTS.md            Instructions any AI should read first
```

## Disclaimer

OpenSeason is a neutral formatting tool provided as-is. You are responsible for the accuracy of every filing and for following the rules of your court. Nothing in this app is legal advice.

Built for people who cannot or do not want to put a whole case in the cloud.
