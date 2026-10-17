//! Cryptographic Evidence Ingestion and EXIF Chain of Custody Logger.

use chrono::Utc;
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::path::Path;
use uuid::Uuid;

use super::types::*;

/// Ingests an observational photo, video, or PDF into the case workspace,
/// extracts EXIF metadata, computes SHA-256 & SHA-512 hashes, and logs to `provenance.json`.
pub fn ingest_observation(
    case_dir: &Path,
    source_file: &Path,
    notes: &str,
    observer_name: Option<&str>,
) -> Result<ObservationEntry, String> {
    if !source_file.exists() {
        return Err(format!("Source observation file not found: {}", source_file.display()));
    }

    let evidence_dir = case_dir.join("evidence");
    fs::create_dir_all(&evidence_dir).map_err(|e| format!("Failed to create evidence dir: {e}"))?;

    let file_bytes = fs::read(source_file).map_err(|e| format!("Failed to read source file: {e}"))?;
    let size_bytes = file_bytes.len() as u64;

    // 1. Compute SHA-256 and SHA-512 hashes
    let mut hasher256 = Sha256::new();
    hasher256.update(&file_bytes);
    let sha256 = hex_encode(&hasher256.finalize());

    let mut hasher512 = Sha512::new();
    hasher512.update(&file_bytes);
    let sha512 = hex_encode(&hasher512.finalize());

    // 2. Extract EXIF metadata if image/jpeg
    let (exif_timestamp, exif_lat, exif_lon, exif_device) = extract_basic_exif(&file_bytes);

    let _original_filename = source_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("observation.dat");

    // Avoid filename collisions by prefixing date & short hash
    let safe_stem = source_file.file_stem().and_then(|s| s.to_str()).unwrap_or("obs");
    let ext = source_file.extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let dest_filename = format!("{}_{:.8}.{}", safe_stem, &sha256, ext);
    let dest_path = evidence_dir.join(&dest_filename);

    // Copy to evidence directory
    fs::copy(source_file, &dest_path).map_err(|e| format!("Failed to copy file to evidence: {e}"))?;

    // Make read-only on disk to preserve evidentiary integrity
    if let Ok(mut perms) = fs::metadata(&dest_path).map(|m| m.permissions()) {
        perms.set_readonly(true);
        let _ = fs::set_permissions(&dest_path, perms);
    }

    let now = Utc::now().to_rfc3339();
    let obs_id = format!("obs-{}", Uuid::new_v4());

    // 3. Load or initialize provenance.json to compute chained hash
    let provenance_path = evidence_dir.join("provenance.json");
    let mut manifest = load_provenance(&provenance_path, case_dir);

    let last_hash = manifest
        .entries
        .last()
        .map(|e| e.chain_of_custody_hash.clone())
        .unwrap_or_else(|| "0000000000000000000000000000000000000000000000000000000000000000".to_string());

    let mut chain_hasher = Sha256::new();
    chain_hasher.update(last_hash.as_bytes());
    chain_hasher.update(sha256.as_bytes());
    chain_hasher.update(now.as_bytes());
    chain_hasher.update(observer_name.unwrap_or("Relator / Citizen Investigator").as_bytes());
    let chain_hash = hex_encode(&chain_hasher.finalize());

    let entry = ObservationEntry {
        id: obs_id,
        filename: dest_filename,
        file_path: format!("evidence/{}", dest_path.file_name().unwrap().to_string_lossy()),
        sha256,
        sha512,
        size_bytes,
        observed_at: now,
        exif_timestamp,
        exif_gps_lat: exif_lat,
        exif_gps_lon: exif_lon,
        exif_device,
        notes: notes.to_string(),
        chain_of_custody_hash: chain_hash,
    };

    manifest.entries.push(entry.clone());
    manifest.updated_at = Utc::now().to_rfc3339();

    save_provenance(&provenance_path, &manifest)?;

    Ok(entry)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn load_provenance(path: &Path, case_dir: &Path) -> ProvenanceManifest {
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(manifest) = serde_json::from_str::<ProvenanceManifest>(&content) {
                return manifest;
            }
        }
    }

    let case_id = case_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("case")
        .to_string();

    ProvenanceManifest {
        case_id,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
        entries: Vec::new(),
    }
}

fn save_provenance(path: &Path, manifest: &ProvenanceManifest) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize provenance: {e}"))?;
    fs::write(path, json).map_err(|e| format!("Failed to write provenance.json: {e}"))
}

/// Extracts basic EXIF tags from JPEG byte buffer without external C dependencies.
fn extract_basic_exif(bytes: &[u8]) -> (Option<String>, Option<f64>, Option<f64>, Option<String>) {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return (None, None, None, None); // Not a JPEG
    }

    let mut cursor = 2;
    while cursor + 4 < bytes.len() {
        if bytes[cursor] != 0xFF {
            break;
        }
        let marker = bytes[cursor + 1];
        if marker == 0xDA || marker == 0xD9 {
            // Start of scan or end of image
            break;
        }

        let length = ((bytes[cursor + 2] as usize) << 8) | (bytes[cursor + 3] as usize);
        if cursor + 2 + length > bytes.len() {
            break;
        }

        // APP1 marker is 0xE1 (EXIF / XMP)
        if marker == 0xE1 && length > 6 {
            let app1_data = &bytes[cursor + 4..cursor + 2 + length];
            if app1_data.starts_with(b"Exif\0\0") {
                return parse_exif_payload(&app1_data[6..]);
            }
        }

        cursor += 2 + length;
    }

    (None, None, None, None)
}

fn parse_exif_payload(payload: &[u8]) -> (Option<String>, Option<f64>, Option<f64>, Option<String>) {
    if payload.len() < 8 {
        return (None, None, None, None);
    }

    let _is_le = match &payload[0..2] {
        b"II" => true,  // Intel Little Endian
        b"MM" => false, // Motorola Big Endian
        _ => return (None, None, None, None),
    };

    // Fallback: look for ASCII timestamp pattern "YYYY:MM:DD HH:MM:SS"
    let mut timestamp = None;
    let mut device = None;

    if let Ok(ascii_str) = std::str::from_utf8(payload) {
        // Regex-free scan for 19-char timestamp "202X:XX:XX XX:XX:XX"
        for window in ascii_str.as_bytes().windows(19) {
            if window.len() == 19
                && window[4] == b':'
                && window[7] == b':'
                && window[10] == b' '
                && window[13] == b':'
                && window[16] == b':'
                && (window[0] == b'2' && window[1] == b'0')
            {
                if let Ok(ts) = std::str::from_utf8(window) {
                    timestamp = Some(ts.to_string());
                    break;
                }
            }
        }

        // Scan for common phone/camera model markers
        for needle in &["iPhone", "Pixel", "Galaxy", "Canon", "Nikon", "Sony"] {
            if let Some(pos) = ascii_str.find(needle) {
                let end = (pos + 24).min(ascii_str.len());
                let snippet = &ascii_str[pos..end];
                let clean: String = snippet.chars().take_while(|c| c.is_alphanumeric() || *c == ' ' || *c == '-').collect();
                if !clean.is_empty() {
                    device = Some(clean.trim().to_string());
                    break;
                }
            }
        }
    }

    (timestamp, None, None, device)
}
