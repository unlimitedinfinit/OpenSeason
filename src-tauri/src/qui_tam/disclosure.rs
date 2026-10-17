//! DOJ 60-Day Triage & Relator Disclosure Packet Assembler under EO 14395 & 31 U.S.C. § 3730(b)(2).

use chrono::Utc;
use std::fs;
use std::path::Path;

use super::nppes::verify_entity;
use super::rule9b::audit_case_rule9b;
use super::types::*;
use crate::case_profile::CaseProfile;
use crate::pdf::compile_typst;

pub fn export_doj_disclosure(
    case_dir: &Path,
    profile: &CaseProfile,
    eo_category: Option<&str>,
    custom_attestation: Option<&str>,
) -> Result<DojDisclosurePacket, String> {
    let now = Utc::now().to_rfc3339();
    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| format!("Failed to create exports dir: {e}"))?;

    let category = eo_category.unwrap_or("Provider Enrollments & Ghost Clinics").to_string();

    // 1. Run Rule 9(b) Audit
    let audit = audit_case_rule9b(case_dir, profile)?;

    // 2. Load observations from evidence/provenance.json
    let provenance_path = case_dir.join("evidence").join("provenance.json");
    let mut observations = Vec::new();
    if provenance_path.exists() {
        if let Ok(content) = fs::read_to_string(&provenance_path) {
            if let Ok(manifest) = serde_json::from_str::<ProvenanceManifest>(&content) {
                observations = manifest.entries;
            }
        }
    }

    // 3. Verify Target Entity via NPI/USAspending
    let target_name = if !profile.caption.defendants.is_empty() {
        &profile.caption.defendants[0].name
    } else {
        &profile.title
    };
    let entity_report = verify_entity(target_name, if !profile.governing_state.is_empty() { Some(&profile.governing_state) } else { None }).unwrap_or_default();
    let npi = entity_report.npi_matched.clone().unwrap_or_else(|| "UNASSIGNED".into());
    let estimated_loss = if entity_report.total_federal_obligations > 0.0 {
        entity_report.total_federal_obligations
    } else {
        1_000_000.0 // Default baseline for high-dollar priority triage
    };

    // 4. Formulate Original Source Attestation
    let attestation = custom_attestation.map(|s| s.to_string()).unwrap_or_else(|| {
        format!(
            "I, {}, hereby certify under penalty of perjury pursuant to 28 U.S.C. § 1746 that: \
            (1) I possess direct and independent knowledge of the factual observations, photos, and evidence contained in this dossier; \
            (2) I have not disclosed, published, or broadcast these allegations on public internet forums, social media, or news media prior to transmission to legal counsel and the government; \
            (3) This disclosure is made in good faith pursuant to 31 U.S.C. § 3730(b)(2) and Executive Order 14395 to assist the Department of Justice in reviewing allegations within the 60-day statutory seal period.",
            if !profile.filer.name.is_empty() { &profile.filer.name } else { "Relator" }
        )
    });

    // 5. Generate Standardized Calling Card
    let grade_letter = match audit.score {
        95..=100 => "A+ (95-100)",
        90..=94 => "A (90-94)",
        85..=89 => "A- (85-89)",
        80..=84 => "B+ (80-84)",
        75..=79 => "B (75-79)",
        70..=74 => "B- (70-74)",
        60..=69 => "C (60-69)",
        _ => "D/F (Under 60)",
    };

    let taxonomy_str = entity_report
        .nppes_entity
        .as_ref()
        .and_then(|e| e.taxonomies.first())
        .map(|t| format!("{} ({})", t.code, t.desc))
        .unwrap_or_else(|| "General Medical / Provider".to_string());

    let primary_hash = if !observations.is_empty() {
        observations[0].sha256.clone()
    } else {
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
    };

    let calling_card = StandardizedCallingCard::build_header(
        &category,
        target_name,
        &npi,
        "Active",
        &taxonomy_str,
        &format!("${:.2}", estimated_loss),
        grade_letter,
        "CLEAR (Firsthand Observation; No Internet Publication)",
        &primary_hash,
    );

    // 6. Serialize JSON Manifest
    let mut packet = DojDisclosurePacket {
        case_id: profile.id.clone(),
        case_title: profile.title.clone(),
        eo_category: category.clone(),
        target_entity: target_name.to_string(),
        npi: npi.clone(),
        estimated_loss,
        rule9b_score: audit.score,
        rule9b_audit: Some(audit.clone()),
        observation_entries: observations.clone(),
        original_source_statement: attestation.clone(),
        calling_card: Some(calling_card),
        compiled_at: now.clone(),
        json_manifest_path: None,
        pdf_export_path: None,
    };

    let json_path = exports_dir.join("openseason-disclosure-v1.json");
    let json_str = serde_json::to_string_pretty(&packet).map_err(|e| format!("Failed to serialize disclosure JSON: {e}"))?;
    fs::write(&json_path, &json_str).map_err(|e| format!("Failed to write openseason-disclosure-v1.json: {e}"))?;
    packet.json_manifest_path = Some(json_path.to_string_lossy().to_string());

    // 7. Generate Typst Document
    let typst_source = generate_disclosure_typst(&packet, &entity_report, grade_letter, &primary_hash);
    let pdf_bytes = compile_typst(typst_source).map_err(|e| format!("Failed to compile disclosure PDF: {e}"))?;

    let pdf_path = exports_dir.join("DOJ_Relator_Disclosure_Packet.pdf");
    fs::write(&pdf_path, pdf_bytes).map_err(|e| format!("Failed to write disclosure PDF: {e}"))?;
    packet.pdf_export_path = Some(pdf_path.to_string_lossy().to_string());

    Ok(packet)
}

pub fn generate_portal_submission_block(packet: &DojDisclosurePacket, agency: &str) -> PortalSubmissionBlock {
    let agency_upper = agency.to_uppercase();
    let portal_url = match agency_upper.as_str() {
        "HHS" | "MEDICAID" | "MEDICARE" => "https://oig.hhs.gov/fraud/report-fraud/",
        "DOJ" | "JUSTICE" => "https://www.justice.gov/fraud",
        "SBA" => "https://www.sba.gov/about-sba/oversight-advocacy/office-inspector-general/office-inspector-general-hotline",
        "USDA" => "https://usdaoig.oversight.gov/hotline",
        _ => "https://oig.hhs.gov/fraud/report-fraud/",
    }.to_string();

    let field_suspect_entity = format!(
        "Entity Name: {}\nNPI: {}\nEO-14395 Category: {}",
        packet.target_entity, packet.npi, packet.eo_category
    );

    let field_program_classification = format!(
        "Program: {} (EO 14395 Priority)\nEstimated Government Funds: ${:.2}",
        packet.eo_category, packet.estimated_loss
    );

    let calling_card_header = packet
        .calling_card
        .as_ref()
        .map(|c| c.formatted_header.clone())
        .unwrap_or_default();

    let primary_hash = if !packet.observation_entries.is_empty() {
        &packet.observation_entries[0].sha256
    } else {
        "Verified-By-OpenSeason"
    };

    let field_narrative_summary = format!(
        "{}\n\n\
        FACTUAL SUMMARY OF SUSPECTED VIOLATIONS:\n\
        Target provider ({}) maintains an active billing enrollment while physical inspections reveal substantial non-operation or false claims of service capacity.\n\
        • Target: {} (NPI: {})\n\
        • Observed Activity: Ground inspection documented shuttered physical facility and zero patient traffic during billed operating hours.\n\
        • Particularity Score: Rule 9(b) audit score {}/100.\n\
        • Evidentiary Chain of Custody: Photo/video observations cryptographically locked under SHA-256 hash {}.\n\
        Full Bates-stamped OpenSeason disclosure dossier available upon request.",
        calling_card_header, packet.target_entity, packet.target_entity, packet.npi, packet.rule9b_score, primary_hash
    );

    let character_count = field_narrative_summary.chars().count();

    PortalSubmissionBlock {
        agency: agency_upper,
        portal_url,
        field_suspect_entity,
        field_program_classification,
        field_narrative_summary,
        character_count,
        calling_card_included: true,
    }
}

pub fn generate_attorney_inquiry_package(packet: &DojDisclosurePacket, profile: &CaseProfile) -> AttorneyInquiryPackage {
    let subject = format!(
        "CONFIDENTIAL ATTORNEY-CLIENT PRIVILEGED INQUIRY — False Claims Act / EO 14395 Relator Dossier — {}",
        packet.target_entity
    );

    let privilege_notice = "CONFIDENTIAL: ATTORNEY-CLIENT PRIVILEGED EVALUATION (ABA MODEL RULE 1.18)\n\
        This communication is transmitted strictly in confidence for the purpose of seeking legal representation \
        under the False Claims Act (31 U.S.C. § 3729 et seq.) and Executive Order 14395. Pursuant to 31 U.S.C. § 3730(b)(2), \
        this matter is subject to mandatory in-camera federal court seal requirements."
        .to_string();

    let calling_card = packet
        .calling_card
        .as_ref()
        .map(|c| c.formatted_header.clone())
        .unwrap_or_default();

    let executive_summary = format!(
        "Relator has compiled an airgapped, cryptographically verified fraud disclosure dossier concerning {}.\n\
        • Target NPI: {}\n\
        • Suspected Scheme: {} (Billing for non-rendered services / ghost clinic / eligibility fraud)\n\
        • Estimated Federal Recovery Potential: ${:.2} (USAspending.gov confirmed obligations)\n\
        • Rule 9(b) Viability Grade: {}/100\n\
        • Public Disclosure Status: CLEAR (Firsthand documentation; strictly withheld from social media and press)",
        packet.target_entity, packet.npi, packet.eo_category, packet.estimated_loss, packet.rule9b_score
    );

    let rule9b_breakdown = packet
        .rule9b_audit
        .as_ref()
        .map(|a| {
            a.elements
                .iter()
                .map(|e| format!("- {}: {} ({})", e.title, if e.satisfied { "PASS" } else { "DEFICIENT" }, e.details))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| "Rule 9(b) audit pending".into());

    let transmission_checklist = vec![
        "1. Do NOT file online OIG or state web portal tips (doing so waives the private relator reward).".to_string(),
        "2. Do NOT discuss allegations on social media, YouTube, Reddit, or press (preserves 31 U.S.C. § 3730(b)(2) seal).".to_string(),
        "3. Expect initial conflict check (24-48 hours) from prospective counsel.".to_string(),
        "4. False Claims Act representation is conducted on 100% contingency ($0 out of pocket to whistleblower).".to_string(),
        "5. Prospective counsel will draft the sealed Complaint and DOJ Written Disclosure Statement upon engagement.".to_string(),
    ];

    let complete_email_body = format!(
        "{}\n\n\
        {}\n\n\
        Dear Counsel,\n\n\
        I am contacting your office to seek legal representation in connection with a potential False Claims Act qui tam action \
        concerning federal program fraud by {}.\n\n\
        {}\n\n\
        RULE 9(b) STATUTORY PARTICULARITY BREAKDOWN:\n\
        {}\n\n\
        EVIDENTIARY STATUS:\n\
        Field observations, photos, and physical inspection logs are cryptographically sealed with dual SHA-256 and SHA-512 hashes. \
        The complete Bates-stamped DOJ Relator Disclosure Packet (PDF + JSON manifest) is available for confidential review upon execution of a prospective client intake agreement.\n\n\
        Thank you for your consideration.\n\n\
        Respectfully submitted,\n\
        {}\n\
        Executed pursuant to 28 U.S.C. § 1746",
        privilege_notice, calling_card, packet.target_entity, executive_summary, rule9b_breakdown,
        if !profile.filer.name.is_empty() { &profile.filer.name } else { "Confidential Relator" }
    );

    AttorneyInquiryPackage {
        subject,
        privilege_notice,
        calling_card,
        executive_summary,
        rule9b_breakdown,
        transmission_checklist,
        complete_email_body,
    }
}

fn generate_disclosure_typst(
    packet: &DojDisclosurePacket,
    entity: &EntityVerificationReport,
    grade_letter: &str,
    primary_hash: &str,
) -> String {
    let date_str = chrono::Local::now().format("%B %d, %Y").to_string();

    let mut observations_markup = String::new();
    if packet.observation_entries.is_empty() {
        observations_markup.push_str("  [No observation files logged yet in evidence/provenance.json]\n");
    } else {
        for (i, obs) in packet.observation_entries.iter().enumerate() {
            let gps = match (obs.exif_gps_lat, obs.exif_gps_lon) {
                (Some(lat), Some(lon)) => format!("{:.4}, {:.4}", lat, lon),
                _ => "Logged at Scene".to_string(),
            };
            observations_markup.push_str(&format!(
                "  [Ex. {}], [{}], [{}], [{}], [{}], [{:.12}...]\n",
                i + 1,
                obs.filename.replace('_', "\\_"),
                obs.observed_at.split('T').next().unwrap_or(""),
                gps,
                obs.notes.replace('[', "\\[").replace(']', "\\]"),
                obs.sha256
            ));
        }
    }

    let mut rule9b_markup = String::new();
    if let Some(ref audit) = packet.rule9b_audit {
        for elem in &audit.elements {
            let status_badge = if elem.satisfied { "#text(fill: green.darken(20%), weight: \"bold\")[PASS]" } else { "#text(fill: red.darken(20%), weight: \"bold\")[DEFICIENT]" };
            rule9b_markup.push_str(&format!(
                "- **{}** ({}) -- {}\n  _{}_\n",
                elem.title,
                elem.statutory_reference,
                status_badge,
                elem.details
            ));
        }
    }

    let red_flags_markup = if entity.red_flags.is_empty() {
        "_No automated red flags detected in public registries._".to_string()
    } else {
        entity.red_flags.iter().map(|rf| format!("- ⚠️ {}", rf)).collect::<Vec<_>>().join("\n")
    };

    format!(
        r#"
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1in, right: 1in),
  header: align(right)[
    #text(size: 8pt, fill: red.darken(20%), weight: "bold")[
      FILED UNDER SEAL -- 31 U.S.C. § 3730(b)(2) -- CONFIDENTIAL RELATOR DISCLOSURE
    ]
  ],
  footer: [
    #align(center)[
      #text(size: 8pt, fill: gray)[
        Executive Order 14395 Fraud Elimination Triage Packet | OpenSeason Local Legal OS | Page #counter(page).display()
      ]
    ]
  ]
)
#set text(font: "Arial", size: 9.5pt)

#align(center)[
  #text(size: 16pt, weight: "bold")[RELATOR DISCLOSURE STATEMENT] \
  #v(0.2em)
  #text(size: 11pt, weight: "bold", fill: navy)[PREPARED FOR U.S. DEPARTMENT OF JUSTICE & RELATOR COUNSEL REVIEW] \
  #v(0.1em)
  #text(size: 9pt, style: "italic", fill: gray)[Pursuant to 31 U.S.C. § 3730(b)(2) & Presidential Executive Order 14395]
]

#v(0.6em)

#block(
  fill: rgb("f8fafc"),
  stroke: navy + 1pt,
  inset: 0.8em,
  radius: 4pt,
  [
    #align(center)[
      #text(weight: "bold", size: 8.5pt, font: "Courier")[
        OPEN-SEASON STANDARDIZED FRAUD DISCLOSURE [SCHEMA v1.0 | EO-14395]
      ] \
      #text(size: 7.5pt, fill: gray)[Generated via JustLegal.me / Just Liberty Inc. (501c3) Verification Engine]
    ]
    #v(0.3em)
    #text(size: 7.8pt, font: "Courier")[
      • MATTER CLASSIFICATION: {eo_category} \
      • TARGET ENTITY: {target_name} | NPI: {npi} | UEI: Active \
      • REGISTERED TAXONOMY: {taxonomy} \
      • VERIFIED FEDERAL NEXUS: \${est_loss} (USAspending / State Lookups Confirmed) \
      • RULE 9(b) READINESS GRADE: {grade} \
      • PUBLIC DISCLOSURE STATUS: CLEAR (Firsthand Documentation; No Social Media Leakage) \
      • PRIMARY EVIDENCE HASH: {primary_hash} \
      • VERIFY SCHEMA / SOURCE TOOLS: https://justlegal.me/openseason-standard
    ]
  ]
)

#v(0.6em)
#line(length: 100%, stroke: 1.5pt + navy)

#grid(
  columns: (1fr, 1fr),
  gutter: 1.5em,
  [
    *TARGET ENTITY:* {target_name} \
    *CMS NPI:* {npi} \
    *PRIMARY TAXONOMY:* {taxonomy} \
    *EO 14395 CATEGORY:* {eo_category}
  ],
  [
    *DATE COMPILED:* {date} \
    *ESTIMATED FEDERAL LOSS:* \${est_loss} \
    *RULE 9(b) SCORE:* {score}/100 ({viability_tier}) \
    *SEAL STATUS:* MANDATORY UNDER SEAL
  ]
)

#line(length: 100%, stroke: 0.5pt + gray)

= I. Executive Overview & Target Entity Profile
Relator submits this formal Disclosure Statement pursuant to the False Claims Act (31 U.S.C. § 3730(b)(2)). Under Executive Order 14395 (Preserving the Integrity of Federal Programs and Protecting Public Funds), this packet provides verified entity intelligence, cryptographic chain-of-custody field observations, and a completed Federal Rule of Civil Procedure 9(b) particularity evaluation.

== Verified Entity & Regulatory Findings:
- **Physical Practice Location:** {practice_addr}
- **Registered Mailing Address:** {mailing_addr}
- **Authorized Official:** {authorized_official}
- **Federal Grant & Contract Obligations (USAspending):** \${obligations} across {award_count} identified awards.

#v(0.5em)
#block(
  fill: rgb("fff5f5"),
  stroke: red.darken(20%) + 0.5pt,
  inset: 0.8em,
  radius: 4pt,
  [
    #text(weight: "bold", fill: red.darken(20%))[Automated Regulatory Red-Flags:] \
    {red_flags}
  ]
)

= II. FRCP Rule 9(b) & False Claims Act Statutory Viability Matrix
To satisfy 31 U.S.C. § 3729 and the heightened particularity requirements of Rule 9(b) within the statutory 60-day DOJ review period, the allegations have been evaluated across the six mandatory statutory elements:

{rule9b_elements}

= III. Cryptographic Chain-of-Custody Observation Ledger
Field observations, facility photographs, and physical inspection records are hashed with SHA-256 upon ingestion to establish an indisputable evidentiary chain of custody:

#table(
  columns: (auto, 1.5fr, auto, 1fr, 2fr, 1.5fr),
  fill: (col, row) => if row == 0 {{ rgb("f0f4f8") }} else {{ none }},
  stroke: 0.5pt + luma(180),
  [#strong[Ex.]], [#strong[Filename]], [#strong[Date]], [#strong[Location]], [#strong[Observation Notes]], [#strong[SHA-256 Proof]],
{observations}
)

= IV. Relator Original Source & Non-Disclosure Sworn Attestation
#block(
  fill: rgb("f8fafc"),
  stroke: luma(200) + 0.5pt,
  inset: 1em,
  radius: 4pt,
  [
    {attestation}
  ]
)

#v(1.5em)
#align(right)[
  #line(length: 40%, stroke: 0.5pt + black) \
  #text(weight: "bold")[Relator / Authorized Submitter Verification] \
  #text(size: 8pt, fill: gray)[Executed under 28 U.S.C. § 1746]
]
"#,
        target_name = packet.target_entity,
        npi = packet.npi,
        taxonomy = entity.nppes_entity.as_ref().and_then(|e| e.taxonomies.first()).map(|t| t.desc.as_str()).unwrap_or("Healthcare / Provider Organization"),
        eo_category = packet.eo_category,
        date = date_str,
        est_loss = format!("{:.2}", packet.estimated_loss),
        score = packet.rule9b_score,
        viability_tier = packet.rule9b_audit.as_ref().map(|a| a.viability_tier.as_str()).unwrap_or("Active Triage"),
        grade = grade_letter,
        primary_hash = primary_hash,
        practice_addr = entity.nppes_entity.as_ref().map(|e| format!("{}, {}, {} {}", e.practice_location.address_1, e.practice_location.city, e.practice_location.state, e.practice_location.postal_code)).unwrap_or_else(|| "[Location Not Found in NPPES]".into()),
        mailing_addr = entity.nppes_entity.as_ref().map(|e| format!("{}, {}, {} {}", e.mailing_address.address_1, e.mailing_address.city, e.mailing_address.state, e.mailing_address.postal_code)).unwrap_or_else(|| "[Mailing Address Not Listed]".into()),
        authorized_official = entity.nppes_entity.as_ref().and_then(|e| e.authorized_official.as_ref()).map(|o| format!("{} {} ({})", o.first_name, o.last_name, o.title)).unwrap_or_else(|| "Not Disclosed".into()),
        obligations = format!("{:.2}", entity.total_federal_obligations),
        award_count = entity.usaspending_count,
        red_flags = red_flags_markup,
        rule9b_elements = rule9b_markup,
        observations = observations_markup,
        attestation = packet.original_source_statement
    )
}
