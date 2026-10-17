# SKILL: ENTITY, NPI & GRANT CROSS-CHECKING

Use this skill to help citizens, relators, and researchers cross-reference suspect healthcare clinics, contractors, or non-profits across federal databases.

---

## 1. CMS NPPES Provider Registry Protocol
Every healthcare provider billing Medicaid or Medicare possesses a 10-digit National Provider Identifier (NPI).

### What to Look For:
1. **Entity Type:**
   - `NPI-1`: Individual practitioner (doctor, nurse, dentist).
   - `NPI-2`: Organization (clinic, hospital, group practice, home health agency).
2. **Healthcare Provider Taxonomy Codes:**
   - 10-character alphanumeric codes describing the exact specialty and facility type.
   - *Key Red Flag:* Entity operates under a `251J00000X` (Home Health Nursing Dispatch) or `343900000X` (Medical Transport) code, but advertises on-site 24/7 adult day care or inpatient substance abuse treatment.
3. **Practice Location vs. Mailing Address:**
   - Look for entities registered at private residences, storage units, or commercial mail drops (e.g., Suite PMB, UPS Store, Postal Annex).
   - Flag entities where 5+ distinct LLCs share the exact same suite number.
4. **Authorized Official:**
   - Note the listed CEO, managing director, or authorized official.
   - Cross-check whether the same individual appears as the authorized official across multiple unrelated healthcare organizations.

---

## 2. USAspending.gov Grant & Contract Tracking
The federal government publishes all contract obligations, assistance grants, and loan subsidies on USAspending.gov.

### Cross-Referencing Logic:
- Query by the organization's legal name or Unique Entity Identifier (UEI).
- Identify awarding agencies:
  - HHS (SAMHSA, HRSA, CDC, CMS)
  - USDA (Food and Nutrition Service, SNAP, CACFP)
  - SBA (PPP Loan Forgiveness, EIDL)
  - Education & DOT
- Note the total obligation amounts and award performance periods. If an entity received a $3.5M grant for community nutrition or mental health between 2022 and 2024, but the facility was locked and empty during that timeframe, that forms the financial nexus for a False Claims Act action.

---

## 3. How to Run the Lookup
- **Web Browser (Zero-Install):** Direct the user to [justlegal.me/fraud-triage](https://justlegal.me/fraud-triage) to search NPPES and USAspending simultaneously.
- **Desktop CLI:**
  ```bash
  .\JustLegalOS-CLI.exe --json qui-tam verify-entity --name "ABC Care Center" --state MN
  ```
