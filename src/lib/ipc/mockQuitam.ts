import type { Store } from "./mockStore";

export function handleQuitamOps<T>(cmd: string, args: Record<string, unknown>, _store: Store): T | null {
  switch (cmd) {
    case "quitam_verify_entity_cmd": {
      const query = String(args.query || "North Star Healthcare LLC");
      const state = String(args.state || "MN");
      return {
        target_query: query,
        npi_matched: "1234567890",
        nppes_entity: {
          npi: "1234567890",
          entity_type: "NPI-2",
          organization_name: query,
          first_name: null,
          last_name: null,
          enumeration_date: "2020-04-15",
          last_updated: "2024-01-10",
          status: "A",
          practice_location: {
            purpose: "LOCATION",
            address_1: "123 Healthcare Blvd, Ste 400",
            address_2: null,
            city: "Minneapolis",
            state,
            postal_code: "55401",
            phone: "612-555-0199",
          },
          mailing_address: {
            purpose: "MAILING",
            address_1: "PO Box 9982",
            address_2: null,
            city: "Minneapolis",
            state,
            postal_code: "55401",
            phone: "612-555-0199",
          },
          taxonomies: [
            {
              code: "251J00000X",
              desc: "Nursing Care / Home Health",
              primary: true,
              state,
              license: "RN-882910",
            },
          ],
          other_names: [],
        },
        total_federal_obligations: 2450000.0,
        usaspending_count: 3,
        awards_summary: [
          {
            internal_id: 101,
            award_id: "CONT_AWD_HHS001",
            recipient_name: query,
            total_obligation: 1250000.0,
            description: "Medicaid Demonstration Grant / Community Care",
            awarding_agency: "Department of Health and Human Services",
            start_date: "2021-10-01",
            end_date: "2024-09-30",
          },
          {
            internal_id: 102,
            award_id: "CONT_AWD_HHS002",
            recipient_name: query,
            total_obligation: 1200000.0,
            description: "Provider Relief / Critical Access",
            awarding_agency: "Department of Health and Human Services",
            start_date: "2022-03-01",
            end_date: "2025-02-28",
          },
        ],
        red_flags: [
          "Address Warning: Mailing address is registered as a Post Office Box.",
          "USAspending: 2 active prime awards totaling $2,450,000.00 subject to EO 14395 audit.",
        ],
        verified_at: new Date().toISOString(),
      } as T;
    }

    case "quitam_records_pack_cmd": {
      const state = String(args.state || "MN").toUpperCase();
      return {
        state,
        statute_name: "Minnesota Government Data Practices Act (Minn. Stat. § 13.01 et seq.)",
        target_agency: "Minnesota Department of Health / Custodian of Records",
        statutory_deadline: "reasonable time (typically 10 business days)",
        records_items: [
          "1. Initial and renewal facility licensing applications, approved floor plans, and operating hours.",
          "2. Annual fire marshal and safety occupancy inspection reports (2020–present).",
          "3. State licensing surveys and statements of deficiencies (CMS Form 2567).",
          "4. Public Medicaid program payment rosters and monthly participant counts.",
        ],
        rendered_letter: `PUBLIC RECORDS REQUEST UNDER MGDPA\nTarget: North Star Healthcare LLC\nRequested Items: Licensing, Fire Inspections, Deficiencies`,
      } as T;
    }

    case "quitam_ingest_observation_cmd": {
      const filePath = String(args.filePath || "clinic_exterior_closed.jpg");
      const filename = filePath.split(/[/\\]/).pop() || "clinic_exterior_closed.jpg";
      return {
        id: `obs-${Date.now()}`,
        filename,
        file_path: filePath,
        sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        sha512: "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        notes: String(args.notes || "Building dark during posted clinical hours."),
        observer: String(args.observer || "Relator / Citizen Investigator"),
        recorded_at: new Date().toISOString(),
        exif_timestamp: "2026-10-14 14:15:22",
        exif_gps_coords: "44.9778° N, 93.2650° W",
        read_only_locked: true,
      } as T;
    }

    case "quitam_audit_9b_cmd": {
      return {
        overall_score: 88,
        readiness_level: "High",
        readiness_grade: "Grade A",
        elements: [
          { id: "rule9b-1-entity-id", title: "Target Entity Identification (Who)", statutory_reference: "FRCP Rule 9(b)", satisfied: true, details: "Target clinic identified with active NPI 1234567890.", missing_evidence: [] },
          { id: "rule9b-2-claim-mechanism", title: "Specific Claim / Drawdown Mechanism (What & How)", statutory_reference: "31 U.S.C. § 3729(a)(1)(A)", satisfied: true, details: "Documented billing for services during shuttered clinic hours.", missing_evidence: [] },
          { id: "rule9b-3-falsity-evidence", title: "Falsity Type & Ground Evidence (Why False)", statutory_reference: "31 U.S.C. § 3729(b)", satisfied: true, details: "Field observations and physical inspection photos confirm non-operation.", missing_evidence: [] },
          { id: "rule9b-4-scienter", title: "Scienter & Knowledge Standard", statutory_reference: "31 U.S.C. § 3729(b)(1)", satisfied: true, details: "Deliberate ignorance or reckless disregard established.", missing_evidence: [] },
          { id: "rule9b-5-materiality-nexus", title: "Materiality & Federal Nexus ($)", statutory_reference: "31 U.S.C. § 3729(a)(1)(B)", satisfied: true, details: "USAspending verifies $2,450,000 in federal drawdowns.", missing_evidence: [] },
          { id: "rule9b-6-public-disclosure", title: "Original Source & Public Disclosure Clearance", statutory_reference: "31 U.S.C. § 3730(e)(4)", satisfied: true, details: "Relator has independent firsthand knowledge prior to public disclosure.", missing_evidence: [] },
        ],
        missing_requirements: [],
        recommended_next_steps: [
          "Export official DOJ Relator Disclosure Packet under seal.",
          "Engage qualified qui tam counsel under ABA Model Rule 1.18 for Track A submission.",
        ],
        audited_at: new Date().toISOString(),
      } as T;
    }

    case "quitam_export_disclosure_cmd": {
      const caseId = String(args.caseId || "mock");
      const category = String(args.category || "Provider Enrollments & Ghost Clinics");
      return {
        case_id: caseId,
        category,
        calling_card: {
          standard_name: "OpenSeason Standard v1.0",
          eo_reference: "EO-14395",
          target_entity: "North Star Healthcare LLC",
          target_npi: "1234567890",
          taxonomy_match: "MATCH",
          federal_program: "HHS Medicaid/Medicare",
          verified_nexus_dollars: "$2,450,000.00",
          rule_9b_readiness_grade: "Grade A (88/100)",
          evidence_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          standard_url: "https://justlegal.me/openseason-standard",
          formatted_header: "======================================================================\n[JUSTLEGAL OS | OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]\nTarget: North Star Healthcare LLC | NPI: 1234567890 | Taxonomy: MATCH\nFederal Program: HHS Medicaid/Medicare | Verified Draw: $2,450,000.00\nRule 9(b) Readiness: Grade A (88/100) | Evidence Hash: SHA-256:e3b0c44298fc1c14...\nTriage Standard: https://justlegal.me/openseason-standard\n======================================================================",
        },
        json_manifest_path: "/mock/exports/openseason-disclosure-v1.json",
        pdf_export_path: "/mock/exports/DOJ_Relator_Disclosure_Packet.pdf",
        evidence_count: 2,
        merkle_root_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        summary: "Relator disclosure statement and evidentiary packet compiled under 31 U.S.C. § 3730(b)(2).",
      } as T;
    }

    case "quitam_format_portal_cmd": {
      const agency = String(args.agency || "HHS");
      return {
        agency,
        portal_url: "https://oig.hhs.gov/fraud/report-fraud/",
        field_suspect_entity: "Entity Name: North Star Healthcare LLC\nNPI: 1234567890\nEO-14395 Category: Provider Enrollments & Ghost Clinics",
        field_program_classification: "Program: Provider Enrollments & Ghost Clinics (EO 14395 Priority)\nEstimated Government Funds: $2450000.00",
        field_narrative_summary: "======================================================================\n[JUSTLEGAL OS | OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]\nTarget: North Star Healthcare LLC | NPI: 1234567890 | Taxonomy: MATCH\nFederal Program: HHS Medicaid/Medicare | Verified Draw: $2,450,000.00\nRule 9(b) Readiness: Grade A (88/100) | Evidence Hash: SHA-256:e3b0c44298fc1c14...\nTriage Standard: https://justlegal.me/openseason-standard\n======================================================================\n\nFACTUAL SUMMARY OF SUSPECTED VIOLATIONS:\nTarget provider (North Star Healthcare LLC) maintains an active billing enrollment while physical ground inspections confirm that the facility is completely shuttered during billed operating hours.",
        character_count: 1546,
        calling_card_included: true,
      } as T;
    }

    case "quitam_format_inquiry_cmd": {
      return {
        subject: "CONFIDENTIAL ATTORNEY-CLIENT PRIVILEGED INQUIRY — False Claims Act / EO 14395 Relator Dossier — North Star Healthcare LLC",
        privilege_notice: "CONFIDENTIAL: ATTORNEY-CLIENT PRIVILEGED EVALUATION (ABA MODEL RULE 1.18)\nThis communication is transmitted strictly in confidence for the purpose of seeking legal representation under the False Claims Act (31 U.S.C. § 3729 et seq.) and Executive Order 14395.",
        calling_card: "======================================================================\n[JUSTLEGAL OS | OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]\nTarget: North Star Healthcare LLC | NPI: 1234567890 | Taxonomy: MATCH\nFederal Program: HHS Medicaid/Medicare | Verified Draw: $2,450,000.00\nRule 9(b) Readiness: Grade A (88/100) | Evidence Hash: SHA-256:e3b0c44298fc1c14...\nTriage Standard: https://justlegal.me/openseason-standard\n======================================================================",
        executive_summary: "Relator has compiled an airgapped, cryptographically verified fraud disclosure dossier concerning North Star Healthcare LLC.\n• Target NPI: 1234567890\n• Suspected Scheme: Provider Enrollments & Ghost Clinics\n• Estimated Federal Recovery Potential: $2,450,000.00\n• Rule 9(b) Viability Grade: Grade A (88/100)\n• Public Disclosure Status: CLEAR",
        rule9b_breakdown: "- Target Entity Identification (Who): SATISFIED\n- Specific False Claims (What/How): SATISFIED\n- Falsity Evidence (Why False): SATISFIED\n- Scienter (Mental State): SATISFIED\n- Materiality ($2.45M): SATISFIED\n- Original Source Clearance: SATISFIED",
        transmission_checklist: [
          "1. Do NOT file online OIG or state web portal tips (doing so waives the private relator reward).",
          "2. Do NOT discuss allegations on social media, YouTube, Reddit, or press (preserves 31 U.S.C. § 3730(b)(2) seal).",
          "3. Expect initial conflict check (24-48 hours) from prospective counsel.",
          "4. False Claims Act representation is conducted on 100% contingency ($0 out of pocket to whistleblower).",
          "5. Prospective counsel will draft the sealed Complaint and DOJ Written Disclosure Statement upon engagement.",
        ],
        complete_email_body: "CONFIDENTIAL: ATTORNEY-CLIENT PRIVILEGED EVALUATION (ABA MODEL RULE 1.18)\nThis communication is transmitted strictly in confidence for the purpose of seeking legal representation under the False Claims Act (31 U.S.C. § 3729 et seq.) and Executive Order 14395.\n\n======================================================================\n[JUSTLEGAL OS | OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE v1.0 | EO-14395]\nTarget: North Star Healthcare LLC | NPI: 1234567890 | Taxonomy: MATCH\nFederal Program: HHS Medicaid/Medicare | Verified Draw: $2,450,000.00\nRule 9(b) Readiness: Grade A (88/100) | Evidence Hash: SHA-256:e3b0c44298fc1c14...\nTriage Standard: https://justlegal.me/openseason-standard\n======================================================================\n\nDear Counsel,\n\nI am contacting your office to seek legal representation in connection with a potential False Claims Act qui tam action concerning federal program fraud by North Star Healthcare LLC.\n\nRespectfully submitted,\nConfidential Relator\nExecuted pursuant to 28 U.S.C. § 1746",
      } as T;
    }

    default:
      return null;
  }
}
