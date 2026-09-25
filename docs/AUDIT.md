# Audit: OpenSeason and JustLegalAid

This audit grades what actually exists in source, not what marketing copy claims. Grades: **Works**, **Partial**, **Stub**, **Missing**, **Could not inspect**.

JustLegalAid could not be cloned or viewed. `git clone https://github.com/unlimitedinfinit/JustLegalAid.git` returned "Repository not found." `gh repo view unlimitedinfinit/JustLegalAid` could not resolve the repository. It is not in the owner's public repository list. No branch, push, or pull request was attempted on that repo. Findings about JustLegalAid below come only from this task's written description and from the public website [justlegal.me](https://justlegal.me).

## OpenSeason (this repository)

Frontend: **Svelte 5 + TypeScript + Tailwind CSS**, served by SvelteKit in SPA mode, wrapped by **Tauri v2**. Backend: **Rust**.

| Area | Grade | What we found |
|---|---|---|
| Legal Airlock | **Works** | Full-screen gate. User must type `I UNDERSTAND`. Now shown only for Confidential mode. |
| Vault password / Argon2id / session key | **Works** | Salt on disk, key in memory, `zeroize` on drop. Password is not stored. Unlock does not verify the password against ciphertext, so a wrong password still "unlocks" and later file decrypt will fail. |
| Hunt create / list / delete | **Partial** | Folders under the app data `vaults/` directory work. `create_new_hunt` used to create only an `info` table, so timeline, parties, evidence, and complaint queries could fail on a brand-new hunt. This run now initializes the full SQLite schema. |
| Evidence encrypt + EXIF strip | **Works** | JPEG/PNG metadata strip, SHA-256, XChaCha20Poly1305 `.enc` files. Drag-and-drop in the UI still opens a file picker instead of using the dropped file. Demo mode fakes evidence. |
| Timeline / parties / complaint sections | **Partial** | IPC and UI exist. They depend on SQLite tables that were not created for new hunts until this run. No tests. |
| HuntWizard + USAspending | **Partial** | Real HTTP POST to `api.usaspending.gov`. The dashboard "Report" button still passes a hardcoded dollar value in one path. The live test prints results and does not assert. Network is optional intel, not case upload. |
| Typst disclosure PDF | **Partial** | Compiler is real. Template is a qui tam disclosure statement. It hard-codes Arial, so Linux hosts without Arial can fail. Dashboard sometimes used mock totals. |
| `.osb` import/export | **Works** | Zip write/read with zip-slip protection. Import names the folder from the file stem, not the original hunt id. |
| SQLite metadata encryption | **Missing** | `metadata.db` is plaintext. Roadmap already notes this. |
| Tests | **Partial** | One network-facing USAspending test, no vault tests. This run adds case-mode, validation, sync-refusal, and Notice of Appeal tests. |
| Windows / macOS bundles | **Could not inspect** | `tauri.conf.json` has `targets: all`. This environment did not run `tauri build` or produce `.msi` / `.dmg`. |
| Telemetry | **Works (absent)** | No analytics SDK. USAspending is the only outbound HTTP client. |

### OpenSeason junk files

Removed from the tree and added to `.gitignore`:

- `api_debug.log`: captured USAspending JSON. Not source.
- `src-tauri/ok`: a zip archive (likely a debug `.osb` dump) sitting in the Rust crate.

Left in place because they are not clearly junk: `OriginalInstructions.txt`, `.vibe/`, `.vscode/`.

## JustLegalAid desktop app (could not inspect source)

The task described a second Tauri v2 app inside the owner's website repo, with docs at `docs/desktop-tauri.md`, `docs/features.md`, `docs/architecture.md`, `docs/roadmap.md`, plus `src-tauri/`, `client/`, `shared/`, `server/`, `AI Agent/`, and `build_desktop.bat`.

**Desktop UI framework: unknown.** The repo was not readable, so this audit cannot grade whether that desktop shell is React, Svelte, vanilla HTML, or something else. The public justlegal.me site is a marketing and intake website, not the desktop UI.

| Claim from the task | Grade | Notes |
|---|---|---|
| Guest Mode, local SQLite `guest.db` in `%APPDATA%/com.justlegal.app/` | **Could not inspect** | Plausible for a Tauri guest profile. Not verified. |
| Evidence folders at `~/Documents/JustLegal/Cases/{caseId}/` | **Could not inspect** | OpenSeason now uses this parent path so the two products do not invent conflicting homes. Subfolders (`evidence/`, `filings/`, and the rest) are the layout requested for this run. |
| `.jldb` backups | **Could not inspect** | Not ported. |
| OS keyring credentials | **Could not inspect** | OpenSeason still uses a session-only password, not a keyring. |
| Website builders for complaints, motions, appeals, Supreme Court | **Could not inspect in repo.** Public site shows help-finding, paper tools, and a signed-in case record. Dedicated builder URLs such as `/forms` render the same marketing page. Whether those builders exist behind login is unverified. |
| Online account sync | **Could not inspect** | See sync verdict. |

## Public justlegal.me site (inspected)

The live site presents three doors: get help with a problem, handle papers (forms, fee waiver, PDF tools, no account), and keep a signed-in case record (people, dates, documents). It is not a drop-in desktop document engine. No Tauri, Guest Mode, or `.jldb` code is visible from the public pages.

## Sync reuse verdict

**Do not reuse JustLegalAid sync as-is. Treat it as unverified, and likely in need of a new adapter even if the private repo later becomes available.**

Reasons:

1. Source was not readable, so "whether that sync actually works is unverified" remains the honest status.
2. OpenSeason's Confidential mode must refuse sync in Rust. Any future client that talks to justlegal.me has to call `sync::request_sync` (or the same guard) before any network write.
3. Guest Mode, keyring storage, and `.jldb` are a different storage model from OpenSeason vaults and from the new `case.json` folders. An adapter would map profile fields (parties, court, docket) into `case.json`, not mount their database files directly.
4. This run ships a stub: standard cases get `not_implemented`; confidential or ever-sealed cases get `confidential_forbidden`. Tests lock that contract down.

## What this foundation run added

- Standard vs Confidential case modes enforced in Rust.
- Portable case folder + fill-once `case.json`.
- Working Notice of Appeal export to `.docx` and PDF.
- `openseason` CLI and `AGENTS.md`.
- Home screen that treats Confidential (Open Season) as one front door, not the only app.

See [cases.md](cases.md) for the folder contract and [architecture.md](architecture.md) for how the pieces connect.
