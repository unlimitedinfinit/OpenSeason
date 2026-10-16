# Troubleshooting & Diagnostic Runbook

Use this runbook to quickly diagnose and fix any issue across the Desktop UI (`OpenSeason.exe`), the Agent CLI/MCP (`openseason-cli.exe`), PDF compilation, Venue Validation, or JustLegal Cloud Sync.

---

## 1. Quick Health Check Commands

Run these from PowerShell in `c:\Users\Deus\Documents\DocumentBuilder`:

```powershell
# 1. Inspect full Legal OS context for a case folder (1-call diagnostic)
.\openseason-cli.exe --json case context "C:\Users\Deus\Documents\JustLegal\Cases\<case-slug>"

# 2. Check Guided Intake Wizard status & see what question/field is missing
.\openseason-cli.exe --json intake status "C:\Users\Deus\Documents\JustLegal\Cases\<case-slug>"

# 3. Check Multi-Venue Competence Matrix (NJ R. 1:4-4(b), CO Rule 121 conferral, Will witnesses, etc.)
.\openseason-cli.exe --json check venue "C:\Users\Deus\Documents\JustLegal\Cases\<case-slug>"

# 4. Run Court Pre-Flight Readiness Audit
.\openseason-cli.exe --json check preflight "C:\Users\Deus\Documents\JustLegal\Cases\<case-slug>"

# 5. View Portfolio Radar across all cases
.\openseason-cli.exe --json portfolio status
```

---

## 2. Common Issues & Exact Fixes

### Issue A: Export says `"Court name is required"` or `"Docket number is required"` on a Will, Lease, Demand Letter, or New Unfiled Complaint
- **Cause:** The case's `matter_tier` or `procedural_stage` was left at the default active litigation setting instead of Tier 1 (`transactional`), Tier 2 (`administrative`), or Pre-Filing (`pre_filing`).
- **Fix (UI):** Open the case, click **🧙 Guided Wizard** at the top right, and set **Matter Tier** to:
  - `Tier 1: Transactional` for Wills, Leases, Contractor Agreements, or POAs.
  - `Tier 2: Pre-Suit / Notice` for Demand Letters, Cure Notices, or OPRA/FOIA requests.
  - `Tier 3: Court Litigation` + **Stage: `Pre-Filing`** for an initial Complaint that does not have a court docket number yet (renders `Case No. __________ (To Be Assigned)`).
- **Fix (CLI):** Set `"matter_tier": "transactional"` (or `"administrative"`, or `"procedural_stage": "pre_filing"`) in `case.json` or via `intake answer`.

---

### Issue B: New Jersey Venue Check Fails with `venue.nj_r_1_4_4b`
- **Cause:** A New Jersey certification or motion draft contains hedging language like `"to the best of my knowledge"` or `"information and belief"`. Under **N.J. Ct. R. 1:4-4(b)** and **R. 1:6-6**, facts must be stated on personal knowledge only, and the certification must use the verbatim punishment clause.
- **Fix:** Remove `"to the best of my knowledge"` from your draft text. When you run `export packet`, OpenSeason automatically injects the verbatim `R. 1:4-4(b)` certification oath:
  > *"I certify that the foregoing statements made by me are true. I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment."*

---

### Issue C: Colorado Venue Check Fails with `venue.co_rule_121_conferral`
- **Cause:** Trial-court civil motions in Colorado often use **C.R.C.P. 121 § 1-15(8)** conferral. That label is `needs_verification`. Papers in the Colorado Court of Appeals may be governed by **C.A.R.** instead, so the checker does not assert Rule 121 there.
- **Fix:** Confirm the rule that governs the paper. For a trial-court motion, open the **Wizard** and answer the Conferral question, or run:
  ```powershell
  .\openseason-cli.exe --json intake answer <case-dir> --id conferral_status --value "Conferred with opposing party on YYYY-MM-DD; motion is opposed."
  ```

---

### Issue D: Multiple Motions in the Same Case Overwriting Draft Sections
- **Cause:** Calling global `draft fill` writes to `drafts/sections.json`.
- **Fix:** Use the **📑 Motions & Filings** docket tab in the UI, or use filing-scoped IPC (`scoped_sections_cmd` / `scoped_fill_cmd`) which stores each motion's guided sections in `drafts/sections-{filing_id}.json`.

---

### Issue E: `OpenSeason.exe` Fails to Copy During Build (`os error 32: process cannot access the file`)
- **Cause:** `OpenSeason.exe` is currently running in the background, locking the executable on Windows.
- **Fix:** Close the `OpenSeason` window or stop the process before copying the new binary:
  ```powershell
  Stop-Process -Name "OpenSeason" -Force -ErrorAction SilentlyContinue
  Copy-Item -Force "src-tauri\target\release\open-season.exe" ".\OpenSeason.exe"
  ```

---

### Issue F: Offline PDF Scan (`intake scan`) Returns 0 Candidates on a Scanned Image PDF
- **Cause:** `pdf_tools::extract_pdf_text` extracts embedded PDF text streams (`Tj`/`TJ` operators) 100% offline using `lopdf`. If a court order was scanned as a flat bitmap image without an OCR text layer, it contains no text operators.
- **Fix:** Place a text-searchable PDF or a companion `.txt` / `.md` note in `orders/` or `evidence/`—`intake scan` reads both `.pdf` text streams and `.txt`/`.md` files in `orders/` and `evidence/`.

---

### Issue G: JustLegal Cloud Sync Refused (`ConfidentialForbidden`)
- **Cause:** The case folder has a `.sealed` marker file, `mode: "confidential"`, or `sealed_at` set.
- **By Design:** Confidential whistleblower vaults are strictly airgapped in `src-tauri/src/sync.rs`. Never delete `.sealed` to force cloud sync. Use local Word/PDF export or `.osb` encrypted bundle export instead.
