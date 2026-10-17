# Architecture & Module Map

**JustLegal OS** is a deterministic, local-first **Legal Operating System** powered by the **OpenSeason** pure-Rust formatting and Qui Tam fraud triage engine. It provides two equal front doors:
1. **Desktop GUI (`JustLegal Desktop` / `OpenSeason.exe`):** Svelte 5 + Tauri v2 window.
2. **Headless Agent CLI & MCP Server (`JustLegalOS-CLI.exe` / `justlegalos-cli`):** `--json` command-line interface and JSON-RPC 2.0 stdio MCP server.

Both doors call the exact same Rust library (`open_season_lib` in `src-tauri/src/`). Neither ever invents facts, hallucinates citations, or files/serves papers.

---

## 1. Strict Modularity Rule (< 800 Lines Per File)

To prevent unmaintainable monoliths as OpenSeason scales across 50 states and hundreds of legal instruments, **every Rust module and Svelte component must remain well under 800 lines**.

### Frontend Component Architecture (`src/`)
| File | Lines | Responsibility |
|---|---|---|
| `src/routes/+page.svelte` | ~340 | Home screen, case list, quick actions, and `PortfolioRadarCard` host |
| `src/routes/case/[id]/+page.svelte` | ~815 | Case dashboard shell, top toolbar, `ProceduralGraphCard` host, tab router, and modals |
| `src/lib/components/PortfolioRadarCard.svelte` | ~355 | **3-Tier Launchpad** (Wills/Leases, Pre-Suit Notices, Court Pleadings), **Global Filer Profile** (`filer.json`), and **Multi-State Deadline Radar** |
| `src/lib/components/ProceduralGraphCard.svelte` | ~350 | **Predictive Procedural State Graph** ("If This Happens, What Are My Next Moves?"), NJ Pause vs CO/Fed Reset badges, Mail Service (+3d) toggle, and 1-Click Case Escalation |
| `src/lib/components/IntakeWizardModal.svelte` | ~430 | **1-Question-at-a-Time Guided Intake Wizard**, Offline PDF Auto-Discovery chips, and **Multi-Venue Competence Matrix** |
| `src/lib/components/MotionDocket.svelte` | ~490 | Multi-Motion & Multi-Filing Docket manager (`profile.filings[]`) |
| `src/lib/components/ServiceCertCard.svelte` | ~435 | Certificate of Service & Proposed Form of Order generator |
| `src/lib/components/PdfToolsModal.svelte` | ~390 | Offline Zero-Leakage PDF Power Station (Merge, Split, Rotate, Byte-Level PII Redact) |
| `src/lib/components/SignatureStudioModal.svelte` | ~385 | Electronic `/s/`, cursive script, and HTML5 canvas signature stamp studio |
| `src/lib/components/fraud/GhostFacilityWizardModal.svelte` | ~460 | **EO 14395 Ghost Facility & Fraud Investigation Wizard** (NPPES lookup, FOIA licensing pack, EXIF evidence ingestion, Rule 9(b) viability audit) |
| `src/lib/components/fraud/QuiTamModeBanner.svelte` | ~75 | Strict airgap alert banner with 31 U.S.C. § 3730(b)(2) seal protection & relator legal warnings |
| `src/lib/components/PostExportNextStepsModal.svelte` | ~135 | Post-export filing checklist, unbundled review guidance, and state bar referral navigator |
| `src/lib/components/LegalTermsModal.svelte` | ~90 | Mandatory first-run legal disclaimer & non-attorney terms acknowledgment with persistent preference |
| `src/lib/components/ai/McpDeveloperTab.svelte` | ~270 | Consumer-facing "Work with an AI Assistant" hub with live diagnostic doctor status and collapsible MCP server config |
| `src/lib/components/tabs/CaseOverviewTab.svelte` | ~95 | Court, caption, and filer profile editor |
| `src/lib/components/tabs/CaseDocumentsTab.svelte` | ~220 | Guided sections editor, Markdown import, and AcroForm inspector/filler |
| `src/lib/components/tabs/CaseEvidenceTab.svelte` | ~155 | Evidence upload and Bates-stamped Exhibit Packet assembler |
| `src/lib/components/tabs/CaseTimelineTab.svelte` | ~70 | Chronological timeline event logger |
| `src/lib/components/tabs/CaseCheckTab.svelte` | ~185 | Fact grounding, authority verification, deadline math, and human sign-off |

### Rust Engine Architecture (`src-tauri/src/`)
| Module | Lines | Responsibility |
|---|---|---|
| `legal_os.rs` | ~720 | **Multi-Venue Competence Matrix** (`NJ`, `CO`, `FEDERAL`, `SCOTUS`, `GENERIC`) + **Predictive Procedural State Graph** (`predict_next_steps`, tolling math, mail offsets) |
| `intake.rs` | ~700 | **1-Question-at-a-Time Guided Intake State Machine** (`intake_status`, `intake_answer`), **Offline PDF Scanner** (`intake_scan`), **Consolidated Context** (`case_context`), and **Global Filer** (`filer.json`) |
| `portfolio.rs` | ~355 | **Cross-Venue Case Escalation** (`escalate_case` copying lower record to `evidence/`), **Multi-State Portfolio Radar** (`portfolio_status`), and Tauri IPC command wrappers |
| `case_profile.rs` | ~520 | `CaseProfile` schema, 3-Tier resolution (`effective_tier()`), `pre_filing` complaint handling, and tier-aware validation |
| `documents.rs` | ~710 | Typst PDF + Office Open XML (`.docx`) compiler with Tier 1 (Instrument), Tier 2 (Statutory Notice), and Tier 3 (Court Caption) headers |
| `guided.rs` | ~280 | Guided section templates across all 3 tiers + filing-scoped storage (`drafts/sections-{filing_id}.json`) |
| `packet.rs` | ~420 | 5-part court motion packet & Bates-stamped Exhibit Packet compiler (uses `resolve_venue(profile).mandatory_oath_text`) |
| `pdf_tools.rs` | ~490 | Pure-Rust `lopdf` offline PDF Merge, Split, Rotate, Byte-Level PII Redaction, and **Page-by-Page Text Extraction** (`extract_pdf_text`) |
| `service.rs` | ~260 | Certificate of Service & Proposed Order compiler |
| `signature.rs` | ~210 | Signature profile persistence and PDF signature stamping |
| `sync.rs` | ~590 | JustLegal Cloud (`justlegal.me`) Firebase Auth, additive merge, subfolder routing (`orders/`, `filings/`, `evidence/`), and `.sealed` airgap |
| `mcp.rs` | ~730 | Offline JSON-RPC 2.0 stdio MCP server exposing all engine tools to AI agents |
| `doctor.rs` | ~85 | Environment & engine diagnostic verification (`openseason doctor`) |
| `qui_tam/types.rs` | ~195 | Data models for NPPES entities, Rule 9(b) audits, observations, FOIA packs, and disclosure packets |
| `qui_tam/nppes.rs` | ~210 | CMS NPPES v2.1 API and USAspending recipient lookup client |
| `qui_tam/rule9b.rs` | ~220 | Deterministic 6-element FRCP Rule 9(b) 0–100 viability evaluation engine |
| `qui_tam/observation.rs` | ~240 | EXIF GPS/timestamp extraction, SHA-256/SHA-512 dual hashing, read-only disk lock, and `provenance.json` log |
| `qui_tam/records_pack.rs` | ~235 | Multi-state licensing and public records request generator (MGDPA, OPRA, CORA, CPRA, FOIA) |
| `qui_tam/disclosure.rs` | ~230 | DOJ 60-day triage & relator disclosure packet compiler (`openseason-disclosure-v1.json` + Typst PDF) |
| `case_commands/quitam_ops.rs`| ~80 | Tauri IPC bridge for qui tam / fraud operations |
| `bin/openseason/` | ~1200 | Modular CLI (`args.rs`, `handlers_core.rs`, `handlers_quitam.rs`, `handlers_pdf.rs`, etc.) |

---

## 2. The 3-Tier Matter Architecture

OpenSeason classifies every matter via `profile.effective_tier()`:
1. **Tier 1: `transactional`** (`will`, `lease`, `contractor`, `power_of_attorney`)
   - **Validation:** Does *not* require `court.name` or `docket_number`. Requires at least one party/testator and signature name.
   - **Formatting:** Renders a centered instrument header (`LAST WILL AND TESTAMENT`, `RESIDENTIAL LEASE AGREEMENT`, etc.) instead of an adversarial `"v."` caption.
   - **Venue Checks:** Audits state witness/notary self-proving affidavit language for Wills (`N.J.S.A. 3B:3-4` / `C.R.S. § 15-11-504`), deposit/rent clauses for Leases, and IP/milestone clauses for Contractor Agreements.
2. **Tier 2: `administrative`** (`demand_letter`, `cure_notice`, `security_deposit_demand`, `records_request`, `agency_appeal`)
   - **Validation:** Does *not* require `court.name` or `docket_number` (uses agency/recipient if provided).
   - **Formatting:** Renders a formal statutory notice block (`TO: / FROM: / DATE: / VIA CERTIFIED MAIL`).
   - **Graduation:** Can be 1-click graduated via `escalate_case(dir, "trial_lawsuit")` into a Tier 3 `pre_filing` Civil Complaint with the demand letter and certified mail receipts automatically copied into `evidence/` as Exhibit A.
3. **Tier 3: `litigation`** (`reconsideration`, `motion`, `enlargement`, `stay`, `complaint`, `notice_of_appeal`, `appellate_brief`, `mandamus`)
   - **Pre-Filing Support:** When `procedural_stage == "pre_filing"` (or `document_kind == "complaint"` before a docket is assigned), validation succeeds without `docket_number` and renders `Case No. __________ (To Be Assigned)` in Word and PDF captions.

---

## 3. Multi-Venue Competence Matrix & Tolling Math

`legal_os::resolve_venue` inspects `governing_state`, `rule_pack_id`, and `court.name`:

| Venue | Sworn Statement Formula | Conferral Requirement | Reconsideration Tolling Effect on Appeal Clock |
|---|---|---|---|
| **New Jersey (`NJ`)** | **Certification in Lieu of Oath (`R. 1:4-4(b)`)**: Verbatim `"I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment."` Rejects `"to the best of my knowledge"`. | `R. 1:6-2(c)` (Discovery motions only) | **`pause_remaining` (`R. 2:4-3(e)`)**: Pauses the 45-day appeal clock on the date the R. 4:49-2 motion is served/filed; only the remaining $45 - \text{elapsed}$ days run after disposition. |
| **Colorado (`CO`)** | **Unsworn Declaration (`C.R.S. § 13-27-106`)**: Under penalty of perjury under the law of Colorado. | **Mandatory (`C.R.C.P. 121 § 1-15(8)`)** on every motion | **`reset_full` (`C.A.R. 4(a)(3)`)**: Timely C.R.C.P. 59 post-trial motion resets the full 49-day appeal clock upon disposition. |
| **Federal (`FEDERAL`)** | **Unsworn Declaration (`28 U.S.C. § 1746`)** | Local Rule 7.1 conferral | **`reset_full` (`FRAP 4(a)(4)`)**: Timely Rule 59(e)/60 motion resets the full 30/60-day appeal clock. |
| **U.S. Supreme Court (`SCOTUS`)** | **28 U.S.C. § 1746 / S. Ct. Rule 39 IFP** | N/A | Timely rehearing/reconsideration in lower court of last resort tolls the 90-day cert window (`S. Ct. Rule 13.3`). |

---

## 4. Storage & Confidentiality Boundary

| Resource | Disk Path |
|---|---|
| Standard Cases | `Documents/JustLegal/Cases/{caseId}/` |
| Global Filer Profile | `Documents/JustLegal/filer.json` |
| Installed Form Packs | `Documents/JustLegal/Packs/{packId}/` |
| Confidential Vault Hunts | AppData `vaults/{id}/` (XChaCha20Poly1305 + Argon2id) |

`sync.rs` enforces the airgap in Rust: any folder containing `.sealed`, `mode == "confidential"`, or `sealed_at` is permanently blocked from `cloud_push` or `cloud_pull`.
