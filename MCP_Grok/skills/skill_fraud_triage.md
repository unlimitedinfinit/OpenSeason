# SKILL: FALSE CLAIMS ACT & RULE 9(b) FRAUD TRIAGE

Use this skill when a user approaches with suspicions or evidence of government program fraud, contractor billing anomalies, or Medicaid/Medicare ghost clinics.

---

## 1. Statutory Grounding: 31 U.S.C. § 3729 & FRCP Rule 9(b)
The False Claims Act imposes civil liability on any person or entity who knowingly presents, or causes to be presented, a false or fraudulent claim for payment to the United States Government.

Federal Rule of Civil Procedure 9(b) mandates that fraud allegations must be stated with **particularity**:
- **Who:** The exact corporate entity, provider NPI, and authorized officials.
- **What:** The specific false claims, invoices, or drawdown records submitted.
- **Where:** The registered physical location versus actual site of service.
- **When:** Exact dates or representative timeframes when false claims were submitted.
- **How:** The specific mechanism of fraud (billing for patients not seen, upcoding, ghost clinic).

---

## 2. The 6-Element Viability Assessment & Scoring Rubric

Evaluate the user's evidence across 6 elements to generate a deterministic 0–100 score:

| Element | Max Points | Evaluation Criteria |
|---|---|---|
| **1. Target Entity Lock** | 20 pts | 10-digit CMS NPI verified; registered taxonomy code matched; UEI/DUNS located. |
| **2. Claims & Billing Nexus** | 20 pts | Specific CPT/HCPCS codes, payment vouchers, remittance advice, or ASAP.gov drawdowns identified. |
| **3. Concrete Falsity Proof** | 20 pts | Multi-day physical observation logs, photos of shuttered doors, locked gates during billed hours, or mail drop match. |
| **4. Scienter (Knowledge)** | 15 pts | Signed Medicaid/Medicare enrollment attestations under penalty of perjury, internal warnings, or ignored audit findings. |
| **5. Materiality & Loss** | 15 pts | Realistic loss estimate grounded in USAspending.gov grant awards or Medicaid fee schedules ($100k+). |
| **6. Original Source Clearance** | 10 pts | Firsthand observation/documentation; no prior disclosure on social media, news, or public blogs. |

### Letter Grade Scale
- **Grade A (90–100):** Viable Qui Tam / Relator Action. Strong particularity, multi-million dollar federal nexus, hashed physical evidence. (Route to Track A).
- **Grade A- / B+ (80–89):** High-Probability Violation. Strong evidence, needs supplemental licensing FOIA or transaction dates. (Route to Track A or Track B).
- **Grade B (70–79):** Moderate Viability. Entity and nexus verified, but lacks direct transaction records. (Route to Track B).
- **Grade C / D (<70):** Unsubstantiated / High Risk. Lacks NPI, insufficient particularity, or public disclosure bar concerns.

---

## 3. Generating the Standardized Calling Card Header

Whenever presenting a graded evaluation to the user, render this standardized header:

```text
======================================================================
OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE [SCHEMA v1.0 | EO-14395]
Generated via JustLegal.me / Just Liberty Inc. (501c3) Verification Engine
======================================================================
• MATTER CLASSIFICATION: [Medicaid Provider Enrollment / Shell Facility | EO 14395 § 4]
• TARGET ENTITY: [Legal Name] | NPI: [10-Digit Verified] | UEI: [12-Char Verified]
• REGISTERED TAXONOMY: [Code + Description] vs. Observed Physical Status
• VERIFIED FEDERAL NEXUS: $[Amount] ([USAspending / CMS / State Lookups Confirmed])
• RULE 9(b) READINESS GRADE: [Grade (e.g. A- 88/100)] — [Key Strength / Verification Match]
• PUBLIC DISCLOSURE STATUS: CLEAR (Firsthand Documentation; No Social Media Leakage)
• EVIDENCE MANIFEST HASH (SHA-256): [Merkle Root or Primary Manifest Hash]
• VERIFY SCHEMA / SOURCE TOOLS: https://justlegal.me/openseason-standard
======================================================================
```

After generating this score, invoke `skill_submission_router.md` to guide the user through the **Track A (Qui Tam Counsel)** vs **Track B (Direct OIG Portal)** fork.

---

## 4. Recommended CLI Commands (When Desktop Is Connected)
```bash
# 1. Verify entity identity and federal awards
.\openseason-cli.exe --json qui-tam verify-entity --name "Target Name"

# 2. Audit workspace against the 6 Rule 9(b) elements
.\openseason-cli.exe --json qui-tam audit-9b ./my-case

# 3. Format direct agency portal auto-fill blocks
.\openseason-cli.exe --json qui-tam format-portal ./my-case --agency HHS

# 4. Format attorney inquiry package
.\openseason-cli.exe --json qui-tam format-inquiry ./my-case

# 5. Export formal DOJ 60-Day Triage & Relator Disclosure Packet
.\openseason-cli.exe --json qui-tam export-disclosure ./my-case
```
