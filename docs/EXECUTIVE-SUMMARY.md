# OpenSeason: Executive Summary & System Architecture

**Document Version:** 1.0.0  
**Project:** OpenSeason Offline Case Manager & Court Document Assembly Engine  
**Target Environments:** Windows 10/11, macOS, Linux  
**Binaries:** `OpenSeason.exe` (Desktop GUI), `openseason-cli.exe` (Headless CLI & MCP Server)

---

## 1. High-Level Vision & Purpose

OpenSeason is a privacy-first, local case management platform and deterministic court document assembly engine. It turns a pro se litigant or attorney's own words, drafts, and evidence into court-shaped Word (`.docx`) and publication-grade PDF documents without practicing law, without hallucinating citations, and without ever filing or serving court papers.

The foundational design of OpenSeason treats the local Rust engine (`open_season_lib`) as a **shared deterministic legal compiler** accessed identically through:
1. **The Desktop Window (`OpenSeason.exe`):** A responsive Svelte 5 desktop interface built on Tauri 2.
2. **The Agent CLI & MCP Server (`openseason-cli.exe`):** A headless command-line interface and Model Context Protocol stdio server that enables autonomous AI assistants to manage cases and assemble filings with zero improvisation.

---

## 2. The 6 Major Implemented Upgrades

To bridge local desktop functionality with the cloud ecosystem while empowering both human users and AI assistants, 6 synchronized upgrades were engineered:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        OPENSEASON SYSTEM MATRIX                        │
├───────────────────────────────────┬────────────────────────────────────┤
│         HUMAN UI UPGRADES         │         AI HELPER UPGRADES         │
├───────────────────────────────────┼────────────────────────────────────┤
│ 1. Cloud Sync Hub & Login Modal   │ 4. CLI & MCP Cloud Commands        │
│    • In-app Firebase Auth         │    • `openseason cloud login|pull` │
│    • Non-destructive additive pull│    • Autonomous AI cloud sync      │
│    • Subfolder routing            │    • Strict airgap defense         │
├───────────────────────────────────┼────────────────────────────────────┤
│ 2. Pre-Flight Court Audit Card    │ 5. Official AcroForm Auto-Filler   │
│    • Real-time caption/docket gate│    • `openseason form inspect|fill`│
│    • Fact-cite audit indicators   │    • PDF field mapping from case   │
│    • Missing requirement warnings │    • Exports filled PDF + JSON     │
├───────────────────────────────────┼────────────────────────────────────┤
│ 3. One-Click Bates Exhibit Packet │ 6. Transcript Grounding & Indexer  │
│    • Visual prefix & numbering UI │    • `openseason check transcripts`│
│    • Automated Table of Exhibits  │    • Line-by-line record indexing  │
│    • SHA-256 integrity table      │    • Verifies fact citations       │
└───────────────────────────────────┴────────────────────────────────────┘
```

### Feature 1 & 4: JustLegal Cloud Sync Hub & Agent Cloud Tools
- **Human UI:** Accessible via the `☁ JustLegal Cloud` header button. Displays account connection state, pull/push buttons, and non-destructive merge status.
- **AI Helper:** Full CLI subcommands (`cloud login`, `cloud status`, `cloud pull`, `cloud push`, `cloud logout`) and MCP tools (`cloud_pull`, `cloud_push`).
- **Routing Engine:** Automatically directs cloud records by record type:
  - `Order` $\rightarrow$ `orders/`
  - `Motion` / `Pleading` / `Complaint` $\rightarrow$ `filings/`
  - `Evidence` / `Transcript` $\rightarrow$ `evidence/`
- **Airgap Guarantee:** If a case is sealed or confidential, cloud sync is permanently disabled and refused.

### Feature 2 & 5: Court Readiness Pre-Flight Audit & AcroForm Auto-Filler
- **Human UI:** Prominent top-of-case audit checklist (`PreFlightAuditCard.svelte`) showing live status for Caption, Docket, Filer Signature, Statutory Deadlines, and Fact Grounding.
- **AI Helper:** Official court form filler (`openseason form inspect <PDF>` and `openseason form fill --case <DIR> <PDF>`). Automatically maps `case.json` values into official court forms (e.g. Colorado JDF 205 fee waiver and JDF 1911 direct appeal) and outputs both filled PDFs and audit JSON.

### Feature 3 & 6: Bates-Stamped Exhibit Packet & Transcript Indexer
- **Human UI:** Visual Exhibit Packet Assembler in the Evidence tab. Generates a unified Bates-stamped PDF with custom prefixes (e.g. `ABRAMS-000001` or `ANDERSON-000001`) alongside a Typst-rendered Table of Exhibits.
- **AI Helper:** Transcript indexer (`openseason check transcripts <DIR>`) indexes depositions and hearings into line numbers and cross-references assertions against sworn testimony.

---

## 3. Status Across The 3 Active Matters

| Case Identifier | State / Venue | Active Matter | Setup & Feature Status |
|---|---|---|---|
| `lucille-abrams` | New Jersey Appellate Division (Docket A-003810-25) | Abrams v. Middlesex County Surrogate (Motion for Reconsideration & Record Audit) | ✅ **Ready:** Pre-flight audit passing; transcripts indexed (`TRANSCRIPTS-AND-RECORD.md`); Bates exhibit packet compiled (`exports/exhibit-packet.pdf`). |
| `anderson-coa` | Colorado Court of Appeals | Defamation Appeal (Response to "Filed Too Late" letter) | ✅ **Ready:** Deadline math verified; response draft imported; preflight audit correctly identifies docket number requirement. |
| `colorado-unemployment` | Colorado Court of Appeals / ICAO | Unemployment Appeal & Fee Waiver | ✅ **Ready:** Official forms JDF 205 and JDF 1911 installed; auto-filling verified (`exports/filled-JDF-205.pdf`, `exports/filled-JDF-1911.pdf`). |

---

## 4. Verification & Quality Gates

The implementation has passed all system verification gates:
- **Rust Test Suite:** 94 unit and integration tests passing (`cargo test --manifest-path src-tauri/Cargo.toml`), 0 failures.
- **TypeScript & Svelte Check:** `npm run check` completed with 0 errors.
- **Production Build:** `npm run build` completed cleanly in 9.96s.
- **Standalone Binaries:** 
  - `OpenSeason.exe` (Desktop application GUI, 45.1 MB)
  - `openseason-cli.exe` (Command line interface and MCP server, 35.7 MB)
- **Convenience Launchers:**
  - `run.ps1` (PowerShell launch script)
  - `run.bat` (Double-click batch launcher)

---

## 5. Maintenance, Troubleshooting & Scalability

### File Structure Reference
```
Documents/JustLegal/Cases/{caseId}/
├── case.json             # Source-of-truth profile and caption data
├── orders/               # Court orders received from court or JustLegal Cloud
├── filings/              # Motions, petitions, and notices generated or filed
├── evidence/             # Exhibits, transcripts, and supporting documentation
├── forms/                # Official blank court-provided PDF AcroForms
├── drafts/               # Markdown drafts and guided drafting sections
├── court-rules/          # Localized court formatting rules (JSON)
└── exports/              # Compiled Word documents, PDFs, and Bates packets
```

### Key Diagnostic Commands
- **Check case readiness:**  
  `.\openseason-cli.exe --json check preflight Documents/JustLegal/Cases/{caseId}`
- **Check transcripts:**  
  `.\openseason-cli.exe --json check transcripts Documents/JustLegal/Cases/{caseId}`
- **Assemble exhibit packet:**  
  `.\openseason-cli.exe --json export exhibit-packet Documents/JustLegal/Cases/{caseId} --prefix {PREFIX}`
- **Inspect court form:**  
  `.\openseason-cli.exe --json form inspect path/to/form.pdf`
- **Fill court form:**  
  `.\openseason-cli.exe --json form fill --case Documents/JustLegal/Cases/{caseId} path/to/form.pdf`
- **Verify cloud status:**  
  `.\openseason-cli.exe --json cloud status`

### Scalability Principles
1. **Preserve Determinism:** Never introduce LLM calls into the core Rust compilation path. All layout math, Typst typesetting, and AcroForm parsing must remain deterministic and reproducible.
2. **Strict Invariant Defense:** Never remove the `NEVER_FILE_NOTICE` or `REVIEW_NOTICE`. Never allow a sealed vault to sync to the network.
3. **Additive-Only Extensions:** When adding new form packs or state court rules, add them as JSON data packs under `templates/court-rules/` or packs registries rather than hardcoded logic.
