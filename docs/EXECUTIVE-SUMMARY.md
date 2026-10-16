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
  - `launch.ps1` (PowerShell launch script with interactive menu)
  - `launch.bat` (Double-click batch launcher with options)

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

---

## 6. The Four Privacy-First Legal Workbench Pillars

OpenSeason includes 4 core workbench capabilities accessible through both the Svelte GUI and the headless Rust CLI/MCP:

### Pillar 1: Multi-Motion & Multi-Filing Docket
- **Problem Solved:** Real litigation requires multiple concurrent motions (e.g. Motion for Reconsideration, Motion for Stay Pending Appeal, Motion to Enlarge Time, Opposition Briefs) without clobbering existing drafts.
- **Implementation:** `filings: Vec<CaseFilingItem>` in `case.json`. Complete with category tags (`🔵 motion`, `🟣 reconsideration`, `🟡 stay`, `🟢 enlarge`, `🟠 response`, `🔴 brief`), rule references, relief descriptions, drafting workbench, and 1-click Word/PDF export (`openseason motion export`).

### Pillar 2: Certificate of Service & Proposed Order Generator
- **Problem Solved:** Courts mandate a formal Certificate of Service (proof of who was served, on what date, and by what method) and a Proposed Form of Order with judicial decretal language.
- **Implementation:** Auto-detects opposing counsel and parties from `caption`, formats service method (electronic court portal, certified mail, direct email, hand delivery), and compiles formal court-formatted Certificates of Service and judge-ready Proposed Orders (`openseason service cert` and `openseason service order`).

### Pillar 3: Offline "Zero-Leakage" PDF Power Station
- **Problem Solved:** Litigants are forced to use third-party web tools (e.g. iLovePDF, Smallpdf) that upload sensitive legal pleadings to remote servers.
- **Implementation:** Pure-Rust, 100% offline document processor using `lopdf`:
  - **Merge:** Stitches multiple pleadings, orders, and exhibits into a single court submission PDF.
  - **Split & Extract:** Extracts specific page ranges (e.g. `1-3, 5-8`) into a clean PDF.
  - **Rotate:** Reorients sideways scanned filings by 90°, 180°, or 270° clockwise.
  - **Byte Redaction:** Pure stream scrubbing for SSNs (`XXX-XX-XXXX`), phone numbers, dates of birth, and custom sensitive names.

### Pillar 4: Visual Legal Signature & Stamp Studio
- **Problem Solved:** Applying court-compliant electronic `/s/` signatures or visual cursive/stylus stamps without cloud electronic signature vendors.
- **Implementation:** Local signature profile store with interactive drawing pad, formal cursive styles, and 1-click AcroForm/PDF placement (`openseason sign stamp`).

---

## 7. The Legal Operating System Architecture (6 Structural Upgrades)

To bridge the gap between a document compiler and a predictive, multi-state **Legal Operating System** for both everyday people and AI agents, six deterministic Rust subsystems (`legal_os.rs`, `intake.rs`, `portfolio.rs`) and three modular Svelte 5 UI components (`PortfolioRadarCard.svelte`, `ProceduralGraphCard.svelte`, `IntakeWizardModal.svelte`) were engineered:

1. **3-Tier Matter Architecture & Pre-Filing Complaints (`case_profile.rs`, `documents.rs`, `guided.rs`)**
   - **Tier 1 (`transactional`):** Everyday legal instruments (`will`, `lease`, `contractor`, `power_of_attorney`). Bypasses adversarial `"v."` captions and docket number requirements; renders clean centered instrument titles and statutory witness/notary self-proving blocks.
   - **Tier 2 (`administrative`):** Pre-suit & agency notices (`demand_letter`, `cure_notice`, `security_deposit_demand`, `records_request`, `agency_appeal`). Renders `TO: / FROM: / VIA CERTIFIED MAIL` statutory notice headers and tracks cure windows.
   - **Tier 3 (`litigation`):** Trial, Appellate, Mandamus, and SCOTUS filings. Supports `procedural_stage: "pre_filing"` so initial Complaints validate and render `Case No. __________ (To Be Assigned)` before the clerk assigns a docket number.

2. **Multi-Venue Competence Matrix (`legal_os.rs`)**
   - Deterministically resolves `NJ`, `CO`, `FEDERAL`, `SCOTUS`, and `GENERIC` venue rules:
     - **New Jersey:** Injects verbatim `N.J. Ct. R. 1:4-4(b)` Certification in Lieu of Oath into assembled packets and flags defective hedging phrases (`"to the best of my knowledge"` or `"information and belief"`).
     - **Colorado:** Suggests a `C.R.S. § 13-27-106` unsworn declaration and a `C.R.C.P. 121 § 1-15(8)` conferral paragraph as `needs_verification` user-editable text. Appellate papers may be governed by C.A.R.; a UUDA declaration often needs a signing date and place.
     - **Federal & SCOTUS:** Enforces `28 U.S.C. § 1746` declarations and `S. Ct. Rule 14.1(a)` Questions Presented.

3. **Predictive Procedural State Graph & Tolling Math (`predict_next_steps`)**
   - Computes downstream legal branches whenever a timeline event or order is logged.
   - Accurately models **NJ Pause/Toll (`pause_remaining` under R. 2:4-3(e))** (pausing the 45-day appeal clock with $45 - \text{elapsed}$ days remaining) vs. **Colorado/Federal Full Clock Reset (`reset_full` under FRAP 4(a)(4) / C.R.C.P. 59)**, plus **Mail Service Offsets (+3 days)**.

4. **1-Question-at-a-Time Guided Intake Wizard & Offline PDF Text Extraction (`intake.rs`, `pdf_tools.rs`)**
   - Extracts text page-by-page offline from PDFs in `orders/` and `evidence/`, auto-detects candidate docket numbers, dates, and court names with file provenance, and drives a step-by-step interview (`intake status`, `intake answer`, `case context`).
   - Supports filing-scoped draft storage (`drafts/sections-{filing_id}.json`) so multiple motions never clobber each other.

5. **Cross-Venue Case Escalation & Multi-State Portfolio Radar (`portfolio.rs`)**
   - `case escalate` graduates a Tier 2 Pre-Suit Notice into a Tier 3 Lawsuit or escalates a Trial case to Appellate / Mandamus / SCOTUS, automatically copying `orders/`, `filings/`, and `evidence/` into the new matter's `evidence/` appendix.
   - `portfolio status` aggregates deadlines across all matters and maintains a fill-once Global Filer Profile (`Documents/JustLegal/filer.json`).

