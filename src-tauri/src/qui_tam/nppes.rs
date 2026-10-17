//! CMS NPPES v2.1 Provider Registry client and Entity Verification against USAspending under EO 14395.

use chrono::Utc;
use reqwest::blocking::Client;
use serde_json::Value;

use super::types::*;
use crate::usaspending;

pub const NPPES_API_URL: &str = "https://npiregistry.cms.hhs.gov/api/?version=2.1";

/// Search CMS NPPES registry by NPI number or by organization/provider name.
pub fn lookup_nppes(query: &str, state: Option<&str>) -> Result<Vec<NppesEntity>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let trimmed = query.trim();
    let is_numeric_npi = trimmed.len() == 10 && trimmed.chars().all(|c| c.is_ascii_digit());

    let mut request = client.get(NPPES_API_URL);
    if is_numeric_npi {
        request = request.query(&[("number", trimmed)]);
    } else {
        request = request.query(&[
            ("organization_name", trimmed),
            ("limit", "10"),
        ]);
        if let Some(st) = state {
            if !st.trim().is_empty() {
                request = request.query(&[("state", st.trim().to_uppercase().as_str())]);
            }
        }
    }

    let response = request
        .send()
        .map_err(|e| format!("NPPES API request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("NPPES returned HTTP status {}", response.status()));
    }

    let body_text = response
        .text()
        .map_err(|e| format!("Failed to read NPPES response: {e}"))?;

    let root: Value = serde_json::from_str(&body_text)
        .map_err(|e| format!("Failed to parse NPPES JSON: {e}"))?;

    let results = root.get("results").and_then(|r| r.as_array());
    let mut entities = Vec::new();

    if let Some(items) = results {
        for item in items {
            if let Some(parsed) = parse_nppes_item(item) {
                entities.push(parsed);
            }
        }
    }

    Ok(entities)
}

fn parse_nppes_item(item: &Value) -> Option<NppesEntity> {
    let npi = item.get("number")?.as_u64()?.to_string();
    let entity_type = item.get("enumeration_type")?.as_str().unwrap_or("NPI-2").to_string();

    let basic = item.get("basic");
    let organization_name = basic.and_then(|b| b.get("organization_name")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let first_name = basic.and_then(|b| b.get("first_name")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let last_name = basic.and_then(|b| b.get("last_name")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let enumeration_date = basic.and_then(|b| b.get("enumeration_date")).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let last_updated = basic.and_then(|b| b.get("last_updated")).and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut practice_location = NppesAddress::default();
    let mut mailing_address = NppesAddress::default();

    if let Some(addresses) = item.get("addresses").and_then(|a| a.as_array()) {
        for addr in addresses {
            let purpose = addr.get("address_purpose").and_then(|v| v.as_str()).unwrap_or("");
            let parsed_addr = NppesAddress {
                purpose: purpose.to_string(),
                address_1: addr.get("address_1").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                address_2: addr.get("address_2").and_then(|v| v.as_str()).map(|s| s.to_string()),
                city: addr.get("city").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                state: addr.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                postal_code: addr.get("postal_code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                phone: addr.get("telephone_number").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };

            if purpose == "LOCATION" {
                practice_location = parsed_addr;
            } else if purpose == "MAILING" {
                mailing_address = parsed_addr;
            }
        }
    }

    let mut taxonomies = Vec::new();
    if let Some(tax_array) = item.get("taxonomies").and_then(|t| t.as_array()) {
        for t in tax_array {
            taxonomies.push(NppesTaxonomy {
                code: t.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                desc: t.get("desc").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                primary: t.get("primary").and_then(|v| v.as_bool()).unwrap_or(false),
                state: t.get("state").and_then(|v| v.as_str()).map(|s| s.to_string()),
                license: t.get("license").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });
        }
    }

    let mut authorized_official = None;
    if let Some(b) = basic {
        if let (Some(fn_val), Some(ln_val)) = (b.get("authorized_official_first_name"), b.get("authorized_official_last_name")) {
            authorized_official = Some(NppesOfficial {
                first_name: fn_val.as_str().unwrap_or("").to_string(),
                last_name: ln_val.as_str().unwrap_or("").to_string(),
                title: b.get("authorized_official_title_or_position").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                phone: b.get("authorized_official_telephone_number").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
    }

    Some(NppesEntity {
        npi,
        entity_type,
        organization_name,
        first_name,
        last_name,
        enumeration_date,
        last_updated,
        practice_location,
        mailing_address,
        taxonomies,
        authorized_official,
    })
}

/// Verifies an entity across both CMS NPPES and USAspending, returning red-flags and obligations.
pub fn verify_entity(query: &str, state: Option<&str>) -> Result<EntityVerificationReport, String> {
    let now = Utc::now().to_rfc3339();
    let nppes_results = lookup_nppes(query, state).unwrap_or_default();
    let primary_entity = nppes_results.into_iter().next();

    // Query USAspending using entity name or query
    let target_name = if let Some(ref ent) = primary_entity {
        ent.organization_name
            .clone()
            .or_else(|| {
                match (&ent.first_name, &ent.last_name) {
                    (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                    _ => None,
                }
            })
            .unwrap_or_else(|| query.to_string())
    } else {
        query.to_string()
    };

    let awards = usaspending::check_target(&target_name).unwrap_or_default();
    let total_obligations: f64 = awards.iter().map(|a| a.total_obligation).sum();

    let mut red_flags = Vec::new();

    if let Some(ref ent) = primary_entity {
        // Red flag 1: Telehealth / dispatch taxonomy vs physical expectation
        for tax in &ent.taxonomies {
            let desc_lower = tax.desc.to_lowercase();
            if desc_lower.contains("dispatch") || desc_lower.contains("telehealth") || desc_lower.contains("transport") {
                red_flags.push(format!(
                    "Taxonomy Warning: Entity registered as '{}' (Code {}), which signifies remote/dispatch rather than dedicated physical facility.",
                    tax.desc, tax.code
                ));
            }
        }

        // Red flag 2: PO Box or Commercial Mail Receiving Agency as physical practice location
        let practice_lower = ent.practice_location.address_1.to_lowercase();
        if practice_lower.contains("po box") || practice_lower.contains("p.o. box") || practice_lower.contains("suite pmbe") {
            red_flags.push("Address Warning: Physical practice location is registered as a Post Office Box or CMRA.".into());
        }

        // Red flag 3: Discrepancy between practice location state and registered tax state
        if !ent.practice_location.state.is_empty() {
            for tax in &ent.taxonomies {
                if let Some(ref tax_state) = tax.state {
                    if !tax_state.is_empty() && tax_state != &ent.practice_location.state {
                        red_flags.push(format!(
                            "Jurisdiction Discrepancy: Physical location is in {}, but license taxonomy is in {}.",
                            ent.practice_location.state, tax_state
                        ));
                    }
                }
            }
        }
    } else {
        red_flags.push("No active CMS NPI registration found matching search criteria.".into());
    }

    if awards.is_empty() {
        red_flags.push("USAspending: No direct federal contracts, assistance awards, or loans identified under this exact entity name.".into());
    }

    Ok(EntityVerificationReport {
        target_query: query.to_string(),
        npi_matched: primary_entity.as_ref().map(|e| e.npi.clone()),
        nppes_entity: primary_entity,
        total_federal_obligations: total_obligations,
        usaspending_count: awards.len(),
        awards_summary: awards,
        red_flags,
        verified_at: now,
    })
}
