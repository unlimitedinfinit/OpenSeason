use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::case_profile::{self, CaseMode, CaseProfile};
use crate::crypto::{self, SessionKey, NONCE_LEN};
use crate::db::HuntDatabase;

pub const SEAL_MARKER_NAME: &str = ".sealed";
pub const SEAL_MARKER_MAGIC: &str = "OpenSeason-sealed";
pub const SEAL_README_NAME: &str = "SEALED.txt";

const ENCRYPT_SUBDIRS: &[&str] = &["orders", "filings", "evidence", "drafts", "exports"];

pub fn seal_marker_path(case_dir: &Path) -> PathBuf {
    case_dir.join(SEAL_MARKER_NAME)
}

pub fn is_dir_sealed(case_dir: &Path) -> bool {
    let path = seal_marker_path(case_dir);
    if !path.is_file() {
        return false;
    }
    fs::read_to_string(&path)
        .map(|text| text.starts_with(SEAL_MARKER_MAGIC))
        .unwrap_or(false)
}

pub fn write_seal_marker(case_dir: &Path, case_id: &str, sealed_at: &str) -> Result<(), String> {
    fs::create_dir_all(case_dir).map_err(|e| e.to_string())?;
    let body = format!(
        "{magic}\nid={id}\nsealed_at={when}\n",
        magic = SEAL_MARKER_MAGIC,
        id = case_id,
        when = sealed_at
    );
    fs::write(seal_marker_path(case_dir), body).map_err(|e| e.to_string())
}

pub fn read_marker_sealed_at(case_dir: &Path) -> Option<String> {
    let text = fs::read_to_string(seal_marker_path(case_dir)).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("sealed_at=").map(|s| s.to_string()))
}

pub fn apply_disk_seal(profile: &mut CaseProfile, case_dir: &Path) {
    if !is_dir_sealed(case_dir) {
        return;
    }
    profile.mode = CaseMode::Confidential;
    if profile.sealed_at.is_none() {
        profile.sealed_at = Some(
            read_marker_sealed_at(case_dir).unwrap_or_else(|| "sealed".to_string()),
        );
    }
}

/// CLI and library seal: write the on-disk marker and mark case.json.
/// Does not encrypt files (no vault password on this path).
pub fn seal_case_on_disk(case_dir: &Path) -> Result<CaseProfile, String> {
    let mut profile = case_profile::load_case(case_dir)?;
    profile.convert_to_confidential()?;
    let sealed_at = profile
        .sealed_at
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    write_seal_marker(case_dir, &profile.id, &sealed_at)?;
    case_profile::save_case(case_dir, &profile)?;
    case_profile::load_case(case_dir)
}

pub fn write_stripped_stub(case_dir: &Path, profile: &CaseProfile) -> Result<(), String> {
    for sub in ["orders", "filings", "evidence", "drafts", "exports"] {
        let path = case_dir.join(sub);
        if path.exists() {
            let _ = fs::remove_dir_all(&path);
        }
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    let sealed_at = profile
        .sealed_at
        .clone()
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    write_seal_marker(case_dir, &profile.id, &sealed_at)?;
    let stub = serde_json::json!({
        "schema_version": profile.schema_version,
        "id": profile.id,
        "title": "",
        "mode": "confidential",
        "document_kind": profile.document_kind,
        "created_at": "",
        "updated_at": Utc::now().to_rfc3339(),
        "sealed_at": sealed_at,
        "court": { "id": "", "name": "" },
        "docket_number": "",
        "caption": { "plaintiffs": [], "defendants": [], "appellants": [], "appellees": [] },
        "filer": { "name": "", "role": "", "address_lines": [], "phone": "", "email": "", "signature_name": "" },
        "notice_of_appeal": {
            "judgment_date": "",
            "judgment_description": "",
            "trial_court_name": "",
            "trial_court_docket": "",
            "user_text": ""
        }
    });
    fs::write(
        case_dir.join("case.json"),
        serde_json::to_string_pretty(&stub).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        case_dir.join(SEAL_README_NAME),
        "This folder is a pointer only. The working copy is in the local Confidential vault.\nOpen Confidential mode and unlock the vault to continue.\nThis file does not name the case, court, or docket.\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let src = entry.path();
        let dest = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir_recursive(&src, &dest)?;
        } else {
            fs::copy(&src, &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn should_skip_encrypt(path: &Path) -> bool {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if name == SEAL_MARKER_NAME || name == SEAL_README_NAME || name == "metadata.db" || name == ".gitkeep"
    {
        return true;
    }
    if path.extension().and_then(|s| s.to_str()) == Some("enc") {
        return true;
    }
    false
}

/// Encrypt one file: nonce is prefixed to the ciphertext. Decrypt-verify, then delete plaintext.
pub fn encrypt_file_replace(path: &Path, key: &SessionKey) -> Result<PathBuf, String> {
    let bytes = fs::read(path).map_err(|e| format!("Read {}: {}", path.display(), e))?;
    let scrubbed = crypto::strip_metadata(&bytes);
    let blob = crypto::encrypt_blob(&scrubbed, key)?;
    let verified = crypto::decrypt_blob(&blob, key)?;
    if verified != scrubbed {
        return Err(format!(
            "Decrypt verify failed for {}",
            path.display()
        ));
    }
    let enc_path = path.with_file_name(format!(
        "{}.enc",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("file")
    ));
    fs::write(&enc_path, &blob).map_err(|e| e.to_string())?;
    let on_disk = fs::read(&enc_path).map_err(|e| e.to_string())?;
    let verified_disk = crypto::decrypt_blob(&on_disk, key)?;
    if verified_disk != scrubbed {
        let _ = fs::remove_file(&enc_path);
        return Err(format!(
            "On-disk decrypt verify failed for {}",
            enc_path.display()
        ));
    }
    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(enc_path)
}

pub fn encrypt_plain_evidence(evidence_dir: &Path, key: &SessionKey) -> Result<usize, String> {
    if !evidence_dir.exists() {
        return Ok(0);
    }
    let db_path = evidence_dir
        .parent()
        .map(|p| p.join("metadata.db"));
    let db = match &db_path {
        Some(path) => HuntDatabase::open(path).ok(),
        None => None,
    };

    let mut count = 0;
    let entries: Vec<_> = fs::read_dir(evidence_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let path = entry.path();
        if !path.is_file() || should_skip_encrypt(&path) {
            continue;
        }
        let original_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let scrubbed = crypto::strip_metadata(&bytes);
        let mut hasher = Sha256::new();
        hasher.update(&scrubbed);
        let hash_hex = format!("{:x}", hasher.finalize());

        let blob = crypto::encrypt_blob(&scrubbed, key)?;
        let verified = crypto::decrypt_blob(&blob, key)?;
        if verified != scrubbed {
            return Err(format!("Decrypt verify failed for {}", original_name));
        }

        let enc_path = evidence_dir.join(format!("{}.enc", hash_hex));
        fs::write(&enc_path, &blob).map_err(|e| e.to_string())?;
        let on_disk = fs::read(&enc_path).map_err(|e| e.to_string())?;
        let verified_disk = crypto::decrypt_blob(&on_disk, key)?;
        if verified_disk != scrubbed {
            let _ = fs::remove_file(&enc_path);
            return Err(format!("On-disk decrypt verify failed for {}", original_name));
        }

        if let Some(db) = &db {
            let nonce = &on_disk[..NONCE_LEN];
            db.insert_evidence(&original_name, &original_name, nonce, &hash_hex)
                .map_err(|e| e.to_string())?;
        }

        fs::remove_file(&path).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// Encrypt case.json plus orders/filings/drafts/exports (and leftover evidence) in a vault copy.
pub fn encrypt_vault_payloads(vault_dir: &Path, key: &SessionKey) -> Result<usize, String> {
    let mut count = 0;
    let case_json = vault_dir.join("case.json");
    if case_json.is_file() {
        encrypt_file_replace(&case_json, key)?;
        count += 1;
    }
    count += encrypt_plain_evidence(&vault_dir.join("evidence"), key)?;
    for sub in ENCRYPT_SUBDIRS {
        if *sub == "evidence" {
            continue;
        }
        let dir = vault_dir.join(sub);
        if !dir.is_dir() {
            continue;
        }
        let entries: Vec<_> = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .collect();
        for entry in entries {
            let path = entry.path();
            if path.is_file() && !should_skip_encrypt(&path) {
                encrypt_file_replace(&path, key)?;
                count += 1;
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::{ensure_case_layout, sample_appeal_profile};
    use crate::sync::{self, SyncAction};

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("os-seal-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_key() -> SessionKey {
        let salt = crypto::generate_salt();
        crypto::derive_key("fixture-seal-password", &salt).unwrap()
    }

    #[test]
    fn cli_save_case_rejects_unseal_payload() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        let profile = sample_appeal_profile();
        case_profile::save_case(&dir, &profile).unwrap();
        seal_case_on_disk(&dir).unwrap();
        assert!(is_dir_sealed(&dir));

        let mut bypass = sample_appeal_profile();
        bypass.mode = CaseMode::Standard;
        bypass.sealed_at = None;
        let err = case_profile::save_case(&dir, &bypass).unwrap_err();
        assert!(
            err.to_lowercase().contains("sealed"),
            "CLI save path must refuse unseal: {err}"
        );
        assert!(is_dir_sealed(&dir));
        let loaded = case_profile::load_case(&dir).unwrap();
        assert!(loaded.is_sync_forbidden());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tauri_save_case_rejects_unseal_payload() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        case_profile::save_case(&dir, &sample_appeal_profile()).unwrap();
        seal_case_on_disk(&dir).unwrap();

        let mut bypass = sample_appeal_profile();
        bypass.mode = CaseMode::Standard;
        bypass.sealed_at = None;
        let err = crate::case_commands::save_case_inner(&dir, bypass).unwrap_err();
        assert!(
            err.to_lowercase().contains("sealed"),
            "Tauri save path must refuse unseal: {err}"
        );
        let refusal = sync::describe_sync_for_dir(&dir, SyncAction::Publish).unwrap();
        assert_eq!(refusal.kind, "confidential_forbidden");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn edited_case_json_cannot_unseal_sync() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        case_profile::save_case(&dir, &sample_appeal_profile()).unwrap();
        seal_case_on_disk(&dir).unwrap();

        let mut fake = sample_appeal_profile();
        fake.mode = CaseMode::Standard;
        fake.sealed_at = None;
        fs::write(
            dir.join("case.json"),
            serde_json::to_string_pretty(&fake).unwrap(),
        )
        .unwrap();

        let loaded = case_profile::load_case(&dir).unwrap();
        assert_eq!(loaded.mode, CaseMode::Confidential);
        assert!(loaded.sealed_at.is_some());
        let refusal = sync::describe_sync_for_dir(&dir, SyncAction::Backup).unwrap();
        assert_eq!(refusal.kind, "confidential_forbidden");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn seal_then_decrypt_round_trip_multiple_evidence_files() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        let evidence = dir.join("evidence");
        fs::write(evidence.join("note-one.txt"), b"alpha exhibit one").unwrap();
        fs::write(evidence.join("note-two.txt"), b"beta exhibit two").unwrap();
        fs::write(evidence.join("binary.dat"), [7u8, 8, 9, 10, 11]).unwrap();
        HuntDatabase::open(dir.join("metadata.db")).unwrap();

        let key = test_key();
        let n = encrypt_plain_evidence(&evidence, &key).unwrap();
        assert_eq!(n, 3);
        assert!(!evidence.join("note-one.txt").exists());
        assert!(!evidence.join("note-two.txt").exists());
        assert!(!evidence.join("binary.dat").exists());

        let mut recovered = Vec::new();
        for entry in fs::read_dir(&evidence).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) == Some("enc") {
                let blob = fs::read(&path).unwrap();
                recovered.push(crypto::decrypt_blob(&blob, &key).unwrap());
            }
        }
        recovered.sort();
        assert!(recovered.iter().any(|b| b == b"alpha exhibit one"));
        assert!(recovered.iter().any(|b| b == b"beta exhibit two"));
        assert!(recovered.iter().any(|b| b.as_slice() == [7u8, 8, 9, 10, 11]));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sealed_stub_omits_identifying_fields() {
        let dir = temp_dir();
        ensure_case_layout(&dir).unwrap();
        let profile = sample_appeal_profile();
        case_profile::save_case(&dir, &profile).unwrap();
        let mut sealed = profile.clone();
        sealed.convert_to_confidential().unwrap();
        write_stripped_stub(&dir, &sealed).unwrap();

        let text = fs::read_to_string(dir.join("case.json")).unwrap();
        assert!(!text.contains("Jordan Example"));
        assert!(!text.contains("Sample County Clerk"));
        assert!(!text.contains("24-01000"));
        assert!(!text.contains("Example v. Sample"));
        assert!(!text.contains("Sample Circuit"));
        assert!(is_dir_sealed(&dir));
        let _ = fs::remove_dir_all(&dir);
    }
}
