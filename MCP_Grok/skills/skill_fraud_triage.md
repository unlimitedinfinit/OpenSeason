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

## 2. The 6-Element Viability Assessment Routine

Walk the user through the 6 statutory elements:

### Element 1: Target Entity Identification
- Obtain the entity's exact legal name, address, and 10-digit CMS National Provider Identifier (NPI).
- Verify registered taxonomy: does the entity claim 24/7 inpatient beds while registered as home-health dispatch?

### Element 2: Specific Claims & Drawdowns
- Explain: *"Courts require proof that an actual claim or drawdown was submitted to the government (e.g. CMS-1500, UB-04, ASAP.gov grant drawdown). It is not enough to allege general suspicion; we must link the conduct to federal dollars."*
- If the user does not have billing sheets, instruct them to file state public records requests for aggregate provider remittance totals.

### Element 3: Falsity Type
- Categorize the falsity under EO 14395:
  1. *Physical Impossibility / Ghost Clinic:* Billing for on-site services when the building was closed, locked, or vacant.
  2. *Services Not Rendered:* Billing for patient visits or meal distributions that never occurred.
  3. *False Certification:* Falsely certifying regulatory compliance or staffing ratios under penalty of perjury.

### Element 4: Scienter / Knowledge Standard
- 31 U.S.C. § 3729(b)(1) requires:
  - Actual knowledge,
  - Deliberate ignorance of the truth or falsity, OR
  - Reckless disregard of the truth or falsity.
- Highlight signed annual provider enrollment attestations where officials certified compliance under penalty of perjury.

### Element 5: Materiality & Loss Calculation
- The falsehood must be *material*—meaning it had a natural tendency to influence the government's decision to pay (Universal Health Services v. Escobar).
- Calculate estimated loss: multiply average daily/monthly billing by the duration of observed non-operation.

### Element 6: Original Source & Public Disclosure Clearance
- **31 U.S.C. § 3730(e)(4):** Verify the allegations did not originate from public news media, blogs, or congressional hearings unless the relator is an *original source* with direct, independent firsthand knowledge.
- **THE 60-DAY SEAL MANDATE:** Warn the user that qui tam suits must be filed under seal by licensed counsel. **Do not post videos or documents online before filing.**

---

## 3. Recommended CLI Commands (When Desktop Is Connected)
```bash
# 1. Verify entity identity and federal awards
.\openseason-cli.exe --json qui-tam verify-entity --name "Target Name"

# 2. Audit workspace against the 6 Rule 9(b) elements
.\openseason-cli.exe --json qui-tam audit-9b ./my-case

# 3. Export formal DOJ 60-Day Triage & Relator Disclosure Packet
.\openseason-cli.exe --json qui-tam export-disclosure ./my-case
```
