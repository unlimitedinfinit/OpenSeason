# Developer & Scaling Guide

This guide explains how to build, test, maintain, and extend OpenSeason without breaking determinism or creating monolithic files.

---

## 1. Prerequisites & Setup

| Tool | Version | Notes |
|---|---|---|
| Node.js | 18+ | Run `npm install --legacy-peer-deps` |
| Rust / Cargo | 1.85+ stable | Install via [rustup.rs](https://rustup.rs) |
| Visual Studio Build Tools | 2022 C++ workload | Required on Windows for Tauri / native linking |

```bash
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason
npm install --legacy-peer-deps
```

---

## 2. Build, Test & Release Commands

```bash
# 1. Run all 163+ Rust unit and integration tests
cargo test --manifest-path src-tauri/Cargo.toml --lib

# 2. Run Svelte 5 + TypeScript typecheck and Vite static build
npm run check
npm run build

# 3. Compile standalone release binaries and copy to repo root
cargo build --release --manifest-path src-tauri/Cargo.toml
Copy-Item -Force "src-tauri\target\release\open-season.exe" ".\JustLegalOS.exe"
Copy-Item -Force "src-tauri\target\release\open-season.exe" ".\OpenSeason.exe"
Copy-Item -Force "src-tauri\target\release\justlegalos-cli.exe" ".\JustLegalOS-CLI.exe"
Copy-Item -Force "src-tauri\target\release\justlegalos-cli.exe" ".\justlegalos-cli.exe"
Copy-Item -Force "src-tauri\target\release\justlegalos-cli.exe" ".\openseason-cli.exe"
```

---

## 3. How to Extend OpenSeason (Scaling Playbook)

### A. Adding a New State Venue (e.g., NY, CA, TX, PA)
1. **Rule Pack Data (`templates/rule-packs/{state}.json`):** Create a JSON rule pack defining the state's court holidays and deadline rules (`days`, `count`, `weekend_holiday_roll`). Validate it with:
   ```bash
   .\JustLegalOS-CLI.exe --json packs validate templates/rule-packs/{state}.json
   ```
2. **Venue Competence Profile (`src-tauri/src/legal_os.rs`):**
   - Add the state's sworn statement statute and verbatim oath string in `resolve_venue(profile: &CaseProfile)`.
   - Specify `tolling_mode` (`"pause_remaining"` vs `"reset_full"`), `mail_service_adder_days`, and `requires_conferral_certificate`.
   - Add any state-specific motion or pleading defect checks in `check_venue_competence(profile: &CaseProfile)`.

### B. Adding a New Document / Instrument Kind
1. **Tier Classification (`src-tauri/src/case_profile.rs`):**
   - If the new kind is a Tier 1 transactional instrument, add its slug to `effective_tier()` under `"transactional"`.
   - If it is a Tier 2 pre-suit/agency notice, add its slug under `"administrative"`.
2. **Guided Prompts (`src-tauri/src/guided.rs`):**
   - Add a match arm in `sections_for_kind(kind: &str)` with plain-language `GuidedSection` prompts. The Guided Intake Wizard (`intake.rs`) automatically turns these sections into step-by-step interview questions!
3. **Header / Caption Rendering (`src-tauri/src/documents.rs`):**
   - Add the human-readable document title in `pleading_heading` / `motion_kind_label`.

### C. Adding a New CLI / MCP / Tauri Command
To maintain 100% parity between Human UI and AI Agent:
1. Implement the pure Rust function in the appropriate domain module (`legal_os.rs`, `intake.rs`, `portfolio.rs`, `pdf_tools.rs`, etc.).
2. Expose a `#[tauri::command]` wrapper in `portfolio.rs` or `case_commands.rs` and register it in `src-tauri/src/lib.rs`.
3. Add a CLI subcommand in `src-tauri/src/bin/openseason.rs`.
4. Add an MCP tool definition and handler in `src-tauri/src/mcp.rs`.
5. Add a browser-preview mock fallback in `src/lib/ipc.ts` and TypeScript interfaces in `src/lib/types/case.ts`.

---

## 4. Architectural Guardrails

- **Keep Every File Under 800 Lines:** Never allow a `.svelte` or `.rs` file to grow past 800 lines. Split UI features into focused components under `src/lib/components/` and Rust logic into focused modules under `src-tauri/src/`.
- **Zero Hallucination Invariant:** Never generate facts, party names, dates, or legal citations inside Rust or UI helpers.
- **Airgap Invariant:** Never bypass `seal::is_dir_sealed(case_dir)` in `sync.rs`.
