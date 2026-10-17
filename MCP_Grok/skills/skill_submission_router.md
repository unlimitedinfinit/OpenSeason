# SKILL: SUBMISSION ROUTER & DUAL-TRACK FILING ENGINE

## PURPOSE & JURISDICTIONAL MANDATE
This skill guides the AI assistant in routing graded fraud dossiers between:
- **Track A: The Relator Reward Track** (Private Qui Tam Counsel First — 31 U.S.C. § 3730(b)(2))
- **Track B: The Citizen Watchdog / Direct OIG Track** (Administrative Agency Web Portals)

---

## 1. THE DECISION MATRIX

Evaluate the user's dossier against these three factors to recommend the optimal path:

| Criteria | Recommend Track A (Qui Tam Counsel) | Recommend Track B (Direct Agency Portal) |
|---|---|---|
| **Estimated Federal Loss** | **$1,000,000+** (sufficient contingency economics) | **Under $1,000,000** or purely regulatory non-compliance |
| **Evidence Type** | Firsthand insider documents, internal billing records, unredacted rosters | External physical observation, commercial mail drop match, licensing anomaly |
| **User Objective** | Seeks statutory 15%–30% whistleblower reward and formal DOJ prosecution | Wants the facility shut down or audited immediately without filing a lawsuit |
| **Public Status** | Strictly non-public; no social media, press, or public disclosure | General public observation or already discussed with local officials |

---

## 2. TRACK A: QUI TAM ATTORNEY INQUIRY PROTOCOL

### Critical Legal Education to Provide
1. **Contingency Economics ($0 Upfront):**
   - Reputable False Claims Act plaintiff firms work on 100% contingency.
   - The whistleblower pays zero dollars out of pocket.
   - If successful, the whistleblower receives **15% to 25%** (if the United States intervenes) or **25% to 30%** (if the firm prosecutes after declination).
   - Under 31 U.S.C. § 3730(d), the defendant must pay the relator's reasonable attorney fees upon recovery.
2. **The First-to-File & Court Seal Rule:**
   - To preserve reward eligibility, the complaint must be filed *under seal* in U.S. District Court and served confidentially on the U.S. Attorney General and the local U.S. Attorney (31 U.S.C. § 3730(b)(2)).
   - Submitting an online tip to an agency web form is an administrative report—not a qui tam filing—and eliminates First-to-File priority.
3. **Privilege for Prospective Clients (ABA Model Rule 1.18):**
   - Communications sent to an attorney for the purpose of seeking legal representation are protected by attorney-client privilege, even if the attorney declines representation.

### Output Template: Attorney Executive Inquiry Package
```text
SUBJECT: CONFIDENTIAL ATTORNEY-CLIENT PRIVILEGED INQUIRY — False Claims Act / EO 14395 Relator Dossier — [Target Entity Name]

ATTN: Qui Tam Intake Committee

Dear Counsel,

I am writing to seek legal representation in connection with a potential qui tam action under the federal False Claims Act (31 U.S.C. § 3729 et seq.) and [State] False Claims Act involving fraudulent claims submitted by [Target Entity Name] under Executive Order 14395.

Pursuant to ABA Model Rule 1.18, this communication is transmitted in confidence for the purpose of evaluating legal representation.

[PASTE STANDARDIZED CALLING CARD HEADER HERE]

EXECUTIVE SUMMARY OF EVIDENCE:
1. Target: [Legal Name, DBA, 10-digit NPI, UEI, Physical Address]
2. Scheme: [Brief description: Ghost clinic / billing for non-rendered services / false certification]
3. Rule 9(b) Elements:
   - Who: [Principals, billing coordinators, corporate officers]
   - What/How: [Specific false claim examples, CPT/HCPCS codes, or grant drawdowns]
   - Why False: [Physical verification: shuttered doors, zero patient traffic, mail drop match]
   - Scienter: [Signed attestations under penalty of perjury, ignored compliance warnings]
   - Federal Loss: Estimated $[Amount] based on USAspending.gov obligations.
4. Chain of Custody:
   - Physical site observations cryptographically stamped with SHA-256 and SHA-512 dual hashes.
   - State open records (licensing/fire inspection) requests prepared.

I have preserved this evidence strictly under seal without social media disclosure. Please let me know your availability for a confidential preliminary discussion.

Sincerely,
[Whistleblower Name / Relator]
[Contact Information]
```

---

## 3. TRACK B: DIRECT AGENCY PORTAL SUBMISSION PROTOCOL

### Critical Legal Education to Provide
- Explain that online agency tips trigger administrative audits, investigations, or license revocations, but do NOT provide a monetary relator bounty.
- Agency web forms typically enforce strict character limits (500–2,000 characters) and time out after 15–20 minutes.

### Output Template: Portal Auto-Fill Blocks

#### Field 1: Suspect Individual / Entity
```text
Legal Name: [Target Legal Name]
DBA: [Trade Name if any]
NPI: [10-Digit National Provider Identifier]
UEI / DUNS: [12-Character Unique Entity Identifier]
Physical Address: [Street, Suite, City, State, ZIP]
Mailing Address: [If different, e.g., CMRA/UPS Store address]
Key Principals: [Names, Titles, Registered Agent]
```

#### Field 2: Program Involved & EO 14395 Classification
```text
Federal Program: [HHS Medicaid / Medicare Part B / USDA SNAP / SBA EIDL]
EO 14395 Category: Provider Enrollment & Ghost Clinic Verification (§ 4)
Estimated Government Dollars at Risk: $[Amount] (USAspending Confirmed)
```

#### Field 3: Narrative Description of Fraud (Character-Limited)
```text
[OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]
Target [Entity Name] (NPI: [NPI]) maintains an active enrollment billing federal [Program] while operating an unstaffed ghost facility at [Address].

EVIDENCE SUMMARY:
1. Ground Observation: On [Dates], physical inspections confirmed the facility was dark, locked, and devoid of staff/patients during posted operating hours.
2. Inconsistent Taxonomy: Registered under taxonomy [Code] ([Description]), but site maintains no clinical equipment or licensed personnel.
3. CMRA Match: Primary billing address resolves to a commercial mail receiving agency (UPS Store box).
4. Evidence Integrity: Photo/video evidence cryptographically locked under SHA-256 hash [Hash].

Full Bates-stamped dossier and Merkle provenance manifest available upon request.
```
