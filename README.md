# JustLegal OS

> **A Free, Private, Local-First Legal Operating System & Document Compiler**  
> Developed by **Just Liberty Incorporated**, a Colorado 501(c)(3) Nonprofit Organization.  
> Powered by the **OpenSeason** pure-Rust formatting core and Executive Order 14395 False Claims Act triage engine.

[![Test Suite](https://img.shields.io/badge/Rust%20Tests-163%20Passed-brightgreen)](file:///src-tauri)
[![Svelte Check](https://img.shields.io/badge/Svelte%20Check-0%20Errors-brightgreen)](file:///src)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Zero Telemetry](https://img.shields.io/badge/Telemetry-Zero%20(Airgapped)-success)](docs/SYNC-AND-CLOUD.md)
[![EO 14395](https://img.shields.io/badge/Whistleblower%20Triage-EO%2014395%20Compliant-purple)](docs/QUI-TAM-EO14395.md)

---

## Executive Overview

**JustLegal OS** is a free, privacy-first legal self-help platform and deterministic procedural operating system designed to dramatically reduce the procedural barriers faced by self-represented litigants, legal aid clinics, and pro bono attorneys.

The central problem JustLegal addresses is the enormous procedural burden placed on people navigating civil courts, administrative agencies, or whistleblower disclosures without full legal representation. Courts strictly enforce caption styles, deadline math, evidentiary separation, certificate of service requirements, and sworn declaration oaths. Missing a single statutory requirement or hedging an affidavit can trigger summary dismissal.

JustLegal solves this by acting as a **deterministic procedural compiler**: it accepts a person's own drafts, verified court records, and physical evidence, checks them against jurisdiction-specific court rules and statutory deadlines, and outputs publication-grade, court-compliant Word (`.docx`) and 28-line numbered PDF packages.

### Legal Guardrails & Mission

* **Not a Law Firm:** JustLegal is developed by a 501(c)(3) nonprofit and does not practice law, give legal advice, invent facts, hallucinate citations, or file/serve documents.
* **Pro-Se Empowerment & Professional Referrals:** Every export reminds users to review every line and consult a licensed attorney. JustLegal directly connects users with state bar association referral services, legal aid organizations, and affordable limited-scope representation under ABA Model Rule 5.4.
* **100% Airgapped Confidentiality:** Sealed matters and whistleblower dossiers are permanently isolated from the internet with zero telemetry, client-side encryption (XChaCha20Poly1305 + Argon2id), and hard filesystem locks.

---

## The 3-Part JustLegal Ecosystem

```mermaid
graph TD
    A["Just Liberty Inc. (501(c)(3) Nonprofit)"] --> B["JustLegal OS (Legal Operating System Umbrella)"]
    
    B --> C["Level 1: JustLegal.me (Public Web Platform)"]
    B --> D["Level 2: JustLegal OS Desktop (Local Airgapped App)"]
    B --> E["Level 3: JustLegal Assistant (GrokBot & MCP Bridge)"]
    
    D --> F["OpenSeason Engine (Pure-Rust Compiler & JustLegalOS-CLI)"]
    E -. Authorized Bridge .-> F
    
    F --> G["Civil Court Assembly & Rules (NJ, CO, Federal, SCOTUS)"]
    F --> H["EO 14395 Qui Tam & False Claims Triage Standard"]
```

| Component | Platform / Tech | Purpose & Core Capabilities | Development Status |
|---|---|---|---|
| **[JustLegal.me](https://justlegal.me)** | React, TypeScript, Vite, Firebase | **Public Legal Resource Hub:** 22,936 court forms, 16,023 mirrored PDFs, in-browser PDF tools, legal aid directories, fee waiver guides, and optional authenticated case sync. | Existing operational website; UX and deep-linking upgrades active. |
| **JustLegal OS Desktop** | Tauri v2, Svelte 5, Rust (`open_season_lib`) | **Local-First Legal Operating System:** Offline case dossiers, multi-tier matter management, 1-click Bates stamping, court record reconstruction, ADA Title II petitions, and zero-telemetry vault. | Pre-compiled standalone `.exe`, `.msi`, and source builds ready. |
| **JustLegal Assistant / GrokBot** | x.ai GrokBot Blueprint, Local CLI Bridge, MCP | **Conversational Guide & Agent Operator:** Public conversational triage for civil procedure and fraud intake; can request authorized local execution of `JustLegalOS-CLI.exe` on a user's machine. | Architecture, system prompts, skills, and recipes ready in `MCP_Grok/`. |

---

## National Court Forms Registry & Public Tools

The public **[JustLegal.me](https://justlegal.me)** platform hosts one of the largest structured legal self-help catalogs in the United States:

* **22,936 Indexed Court Forms:** Covering all 50 states, Washington D.C., and all 94 federal judicial districts.
* **16,023 Mirrored Official PDFs:** Stored with retrieval metadata, official source links, and SHA-256 integrity verification hashes.
* **In-Browser Legal Utilities:**
  - **PDF Studio & Zero-Leakage PII Redactor:** Merge, split, rotate, and scrub SSNs/DOBs client-side.
  - **Bates Stamper:** Sequentially number exhibits with custom prefix formatting (e.g., `ABRAMS-000001`).
  - **Fee Waiver Generator:** Guided completion of in forma pauperis financial qualification forms.
  - **Courtroom Oral Argument Cards:** Structured rehearsal notes and hearing outline cards.
  - **State Bar & Legal Aid Router:** Direct referral to State Bar certified lawyer referral services and Legal Services Corporation (LSC) providers.

---

## 12 Production Subsystems (Pure-Rust Engine & CLI)

The deterministic Rust core (`src-tauri/src/`) and native command-line interface (**`JustLegalOS-CLI.exe`**) expose 12 modular subsystems that both human operators and autonomous AI agents can operate with mathematical reproducibility:

```
┌────────────────────────────────────────────────────────────────────────┐
│               JUSTLEGAL OS SUB-SYSTEM ARCHITECTURE MATRIX              │
├───────────────────────────────────┬────────────────────────────────────┤
│ 1. 360° Case Context Bundle       │ 7. Accountability & Ethics Packets │
│    • `case context` (1-call JSON) │    • `accountability routes|packet`│
│    • Profile, events, gaps, rules │    • Judicial & attorney ethics    │
├───────────────────────────────────┼────────────────────────────────────┤
│ 2. Guided Intake & PDF Auto-Scan  │ 8. ADA Title II Petition Engine    │
│    • `intake scan` & `answer`     │    • `ada petition`                │
│    • Offline PDF text extraction  │    • CART, remote hearing, breaks  │
├───────────────────────────────────┼────────────────────────────────────┤
│ 3. Predictive Next Steps & Venue  │ 9. Proof of Service & Orders       │
│    • `case next-steps`            │    • `service cert` & `order`      │
│    • NJ Pause vs. Fed Reset toll  │    • Judge-ready Proposed Orders   │
├───────────────────────────────────┼────────────────────────────────────┤
│ 4. 30-Day Extension Safeguard     │ 10. Bates Packets & Court Exports  │
│    • `motion new --kind enlarge`  │    • `export exhibit-packet`       │
│    • Procedural lifeline & email  │    • Typst 28-line numbered PDFs   │
├───────────────────────────────────┼────────────────────────────────────┤
│ 5. Court Record Reconstruction    │ 11. Zero-Leakage Offline PDF Studio│
│    • `record inventory|reconcile` │    • `pdf merge|split|rotate|redact`│
│    • Docket text parser & Form NJ │    • Byte-level PII scrubbing      │
├───────────────────────────────────┼────────────────────────────────────┤
│ 6. Judicial Findings Audit        │ 12. EO 14395 Qui Tam Fraud Triage  │
│    • `review record`              │    • `qui-tam verify|audit-9b|pack`│
│    • Cross-examines orders vs doc │    • Dual-track routing & card     │
└───────────────────────────────────┴────────────────────────────────────┘
```

### CLI Command Family Quick-Reference

All commands output structured, envelope-wrapped JSON (`{ ok, never_files, never_serves, data, issues }`):

```bash
# 1. Inspect complete case context in a single call
.\JustLegalOS-CLI.exe --json case context Documents/JustLegal/Cases/lucille-abrams

# 2. Extract offline text and candidate dates from court PDFs
.\JustLegalOS-CLI.exe --json intake scan Documents/JustLegal/Cases/lucille-abrams

# 3. Compute predictive next procedural steps and deadline tolling
.\JustLegalOS-CLI.exe --json case next-steps Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json check venue Documents/JustLegal/Cases/lucille-abrams

# 4. Court Record Reconstruction & Transcript Reconciliation
.\JustLegalOS-CLI.exe --json record inventory Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json record reconcile Documents/JustLegal/Cases/lucille-abrams --docket "02/15/2026 ORDER DISMISSING COMPLAINT..."
.\JustLegalOS-CLI.exe --json record missing Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json record request-form Documents/JustLegal/Cases/lucille-abrams --state NJ

# 5. Judicial Findings Audit & Ethical Accountability
.\JustLegalOS-CLI.exe --json review record Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json accountability routes Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json accountability packet Documents/JustLegal/Cases/lucille-abrams --issue "finding-3-unsupported"

# 6. ADA Title II Accommodation Petition Generator
.\JustLegalOS-CLI.exe --json ada petition Documents/JustLegal/Cases/lucille-abrams --name "Jordan Example" --accommodations "remote,reading_time"

# 7. False Claims Act / Qui Tam Fraud Triage (EO 14395)
.\JustLegalOS-CLI.exe --json qui-tam verify-entity --name "Jordan Health LLC" --state MN
.\JustLegalOS-CLI.exe --json qui-tam audit-9b Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json qui-tam ingest-observation Documents/JustLegal/Cases/lucille-abrams --file "photo.jpg" --notes "Doors locked at 2 PM on Tuesday"
.\JustLegalOS-CLI.exe --json qui-tam records-pack Documents/JustLegal/Cases/lucille-abrams --state MN
.\JustLegalOS-CLI.exe --json qui-tam export-disclosure Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json qui-tam format-inquiry Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json qui-tam format-portal Documents/JustLegal/Cases/lucille-abrams --agency HHS

# 8. Document Production, Bates Stamping & Redaction
.\JustLegalOS-CLI.exe --json export exhibit-packet Documents/JustLegal/Cases/lucille-abrams --prefix "ABRAMS-"
.\JustLegalOS-CLI.exe --json export court-md Documents/JustLegal/Cases/lucille-abrams
.\JustLegalOS-CLI.exe --json pdf redact --input "sensitive.pdf" --patterns "SSN,DOB"
```

---

## Executive Order 14395 & Qui Tam Triage Engine

Built specifically to operationalize Presidential **Executive Order 14395** (*Preserving the Integrity of Federal Programs and Protecting Public Funds*) and **False Claims Act** (*31 U.S.C. §§ 3729–3733*) triage:

1. **60-Day Statutory Seal Clock Compliance (31 U.S.C. § 3730(b)(2)):** Automates the first 40 hours of a federal investigation before the dossier hits DOJ Fraud Section or OIG desks, eliminating the need for repeated 6-month seal extensions.
2. **Targets the 3 Primary Vulnerabilities in EO 14395:**
   - *Provider Enrollments & Ghost Clinics:* Keyless CMS NPPES v2.1 API cross-checking against commercial mail drops, strip malls, and deactivated NPI records.
   - *Eligibility Self-Attestations:* Auto-generated state licensing FOIA packs (MGDPA, OPRA, CORA, CPRA).
   - *Third-Party Intermediaries:* USAspending award linkage tracing federal funds to shell management LLCs.
3. **Deterministic 6-Element Rule 9(b) Viability Engine:** Grades dossiers from 0 to 100 across Who, What, When, Where, How, and Scienter/Materiality.
4. **Cryptographic Observation Sealing:** Smartphone photo/video ingestion with EXIF GPS coordinates, timestamp extraction, dual SHA-256 and SHA-512 hashes, read-only disk lock, and an immutable `provenance.json` Merkle chain.
5. **Dual-Track Submission Engine (The Critical Fork):**
   - **Track A (Relator Reward Track — Qui Tam Counsel First):** Generates privileged inquiry packets under ABA Model Rule 1.18 for private False Claims Act attorneys working on 100% contingency ($0 upfront; 15%–30% statutory relator bounty under 31 U.S.C. § 3730(d)), strictly protecting mandatory *in camera* seal rules (§ 3730(b)(2)) and preventing accidental forfeiture via premature public disclosure.
   - **Track B (Citizen Watchdog / Direct Agency Track):** Formats concise, character-limited (<2,000 char) text blocks pre-mapped to HHS-OIG, DOJ Civil Fraud, and State AG MFCU web portals.
6. **Standardized Calling Card Watermark:** Injects the machine-readable header `[JUSTLEGAL OS | OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]` with the public verification URL (`https://justlegal.me/openseason-standard`).

---

## Technical Specifications & Verification Gates

* **Desktop Application:** Tauri v2, Svelte 5, TypeScript, Tailwind CSS, Vite (`JustLegalOS.exe`).
* **Command-Line & MCP Server:** Rust native binary (`JustLegalOS-CLI.exe`), clap 4.5, stdio JSON-RPC 2.0.
* **Document Compilation:** Typst 0.12 (native PDF typesetting), Office Open XML (Word DOCX), `lopdf` (offline stream parsing).
* **Cryptography & Vault:** XChaCha20Poly1305, Argon2id password verifier, SHA-256/SHA-512 Merkle provenance, strict filesystem permissions.
* **Verification Gates (Passing):**
  - **163 Rust tests passing** (`cargo test --manifest-path src-tauri/Cargo.toml`).
  - **0 Svelte / TypeScript errors** (`npm run check`).
  - **Deterministic zero-telemetry guarantee:** Tested in `src-tauri/src/sync.rs` and `seal.rs`.

---

## Getting Started

### Ready-to-Run Standalone Binaries
* **Desktop Application:** Run `.\JustLegalOS.exe` in the root repository.
* **Agent CLI & MCP Server:** Run `.\JustLegalOS-CLI.exe` in the root repository.
* **Production Installers:** Full Windows installation packages are located in `installers/`:
  - `OpenSeason_0.1.0_x64-setup.exe` (NSIS Installer)
  - `OpenSeason_0.1.0_x64_en-US.msi` (Windows Installer Package)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason

# Install frontend dependencies
npm install --legacy-peer-deps

# Run test suite
cargo test --manifest-path src-tauri/Cargo.toml
npm run check

# Launch development environment (Vite + Tauri)
npm run tauri dev

# Compile standalone production binaries
npm run tauri build -- --no-bundle
```

---

## Project Repository Layout

```
JustLegal/
├── JustLegalOS.exe           # Standalone Desktop GUI (Tauri v2 + Svelte 5)
├── JustLegalOS-CLI.exe       # Headless Agent CLI & Stdio MCP Server (Rust)
├── launch.bat / launch.ps1   # 1-Click Interactive Desktop Launchers
├── src/                      # Svelte 5 Frontend (Dashboard, Wizards, Cards)
│   ├── routes/               # Case views, dockets, and home screens
│   └── lib/components/       # 3-Tier launchpad, audit cards, modals
├── src-tauri/                # Core Rust Engine (46+ Modular Crate Files)
│   ├── src/qui_tam/          # EO 14395 triage, Rule 9(b), NPPES, dual-track
│   ├── src/accountability/   # Judicial review, ethics routes, discrepancy packets
│   ├── src/court_access/     # Title II ADA petition generator & accommodations
│   ├── src/record/           # Evidentiary tiers, docket reconciliation & FOIA
│   ├── src/documents/        # Typst PDF and Word DOCX court document compilers
│   ├── src/mcp/              # Native Stdio JSON-RPC 2.0 MCP server tools
│   └── src/bin/openseason/   # Modular JustLegalOS-CLI command handlers
├── MCP_Grok/                 # GrokBot Blueprints, System Prompts & Skill Files
├── templates/                # Jurisdiction Rule Packs (NJ, CO, Federal, SCOTUS)
├── fixtures/                 # Obviously fake sample cases (Jordan Example)
└── docs/                     # Full Architectural & Procedural Documentation
```

---

## Ethical Boundaries & Disclaimers

**JustLegal OS is a formatting and assembly aid, not a law firm.**  
It does not practice law, file papers, serve process, or give legal advice. Users are solely responsible for reviewing every page before filing and ensuring accuracy under court rules. Confidential cases are never uploaded to the cloud.

Developed with care by **Just Liberty Incorporated** (Colorado 501(c)(3)) to make the rule of law accessible to everyone.
