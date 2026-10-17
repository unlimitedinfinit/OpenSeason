//! Types and data models for Qui Tam / False Claims Act (31 U.S.C. § 3729) fraud triage under EO 14395.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NppesAddress {
    pub purpose: String, // "LOCATION" or "MAILING"
    pub address_1: String,
    pub address_2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NppesTaxonomy {
    pub code: String,
    pub desc: String,
    pub primary: bool,
    pub state: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NppesOfficial {
    pub first_name: String,
    pub last_name: String,
    pub title: String,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct NppesEntity {
    pub npi: String,
    pub entity_type: String, // "NPI-1" (Individual) or "NPI-2" (Organization)
    pub organization_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub enumeration_date: String,
    pub last_updated: String,
    pub practice_location: NppesAddress,
    pub mailing_address: NppesAddress,
    pub taxonomies: Vec<NppesTaxonomy>,
    pub authorized_official: Option<NppesOfficial>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EntityVerificationReport {
    pub target_query: String,
    pub npi_matched: Option<String>,
    pub nppes_entity: Option<NppesEntity>,
    pub total_federal_obligations: f64,
    pub usaspending_count: usize,
    pub awards_summary: Vec<crate::usaspending::AwardSummary>,
    pub red_flags: Vec<String>,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule9bElement {
    pub id: String,
    pub title: String,
    pub statutory_reference: String,
    pub satisfied: bool,
    pub details: String,
    pub missing_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule9bAudit {
    pub case_id: String,
    pub score: u32, // 0 - 100
    pub viability_tier: String, // "High Viability (Attorney Ready)", "Developing (Claims Proof Needed)", "Preliminary / Speculative"
    pub elements: Vec<Rule9bElement>,
    pub missing_requirements: Vec<String>,
    pub next_investigative_steps: Vec<String>,
    pub public_disclosure_risk: String,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ObservationEntry {
    pub id: String,
    pub filename: String,
    pub file_path: String,
    pub sha256: String,
    pub sha512: String,
    pub size_bytes: u64,
    pub observed_at: String,
    pub exif_timestamp: Option<String>,
    pub exif_gps_lat: Option<f64>,
    pub exif_gps_lon: Option<f64>,
    pub exif_device: Option<String>,
    pub notes: String,
    pub chain_of_custody_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProvenanceManifest {
    pub case_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub entries: Vec<ObservationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RecordsPackRequest {
    pub state: String,
    pub statute_title: String,
    pub target_agency: String,
    pub target_entity_name: String,
    pub target_entity_address: String,
    pub records_items: Vec<String>,
    pub formal_letter_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DojDisclosurePacket {
    pub case_id: String,
    pub case_title: String,
    pub eo_category: String, // "Provider Enrollment", "Eligibility Self-Attestation", "Third-Party Intermediary"
    pub target_entity: String,
    pub npi: String,
    pub estimated_loss: f64,
    pub rule9b_score: u32,
    pub rule9b_audit: Option<Rule9bAudit>,
    pub observation_entries: Vec<ObservationEntry>,
    pub original_source_statement: String,
    pub compiled_at: String,
    pub json_manifest_path: Option<String>,
    pub pdf_export_path: Option<String>,
}
