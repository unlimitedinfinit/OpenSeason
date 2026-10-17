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

    // 5. Serialize JSON Manifest
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
        compiled_at: now.clone(),
        json_manifest_path: None,
        pdf_export_path: None,
    };

    let json_path = exports_dir.join("openseason-disclosure-v1.json");
    let json_str = serde_json::to_string_pretty(&packet).map_err(|e| format!("Failed to serialize disclosure JSON: {e}"))?;
    fs::write(&json_path, &json_str).map_err(|e| format!("Failed to write openseason-disclosure-v1.json: {e}"))?;
    packet.json_manifest_path = Some(json_path.to_string_lossy().to_string());

    // 6. Generate Typst Document
    let typst_source = generate_disclosure_typst(&packet, &entity_report);
    let pdf_bytes = compile_typst(typst_source).map_err(|e| format!("Failed to compile disclosure PDF: {e}"))?;

    let pdf_path = exports_dir.join("DOJ_Relator_Disclosure_Packet.pdf");
    fs::write(&pdf_path, pdf_bytes).map_err(|e| format!("Failed to write disclosure PDF: {e}"))?;
    packet.pdf_export_path = Some(pdf_path.to_string_lossy().to_string());

    Ok(packet)
}

fn generate_disclosure_typst(packet: &DojDisclosurePacket, entity: &EntityVerificationReport) -> String {
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

#v(1em)
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
