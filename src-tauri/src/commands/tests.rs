use std::fs;
use std::path::PathBuf;

use crate::crypto;
use crate::db::HuntDatabase;
use crate::seal;
use super::evidence::{add_hunt_evidence_bytes_at, delete_hunt_evidence_at};

fn sealed_hunt_with_evidence(password: &str) -> (PathBuf, i64, String, Vec<u8>, PathBuf) {
    let dir = std::env::temp_dir().join(format!("os-evdel-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(dir.join("evidence")).unwrap();
    let salt = crypto::generate_salt();
    let key = crypto::derive_key(password, &salt).unwrap();
    let verifier = crypto::create_password_verifier(&key).unwrap();
    seal::write_seal_marker(&dir, "00000000-0000-4000-8000-000000000099", "2024-01-01T00:00:00Z")
        .unwrap();
    let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fs::write(dir.join("evidence").join(format!("{}.enc", hash)), b"only-copy").unwrap();
    let db = HuntDatabase::open(dir.join("metadata.db")).unwrap();
    let id = db.insert_evidence("exhibit", "photo.jpg", &[0u8; 24], hash).unwrap();
    drop(db);
    let enc = dir.join("evidence").join(format!("{}.enc", hash));
    (dir, id, salt, verifier, enc)
}

#[test]
fn sealed_evidence_delete_refused_without_password() {
    let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
    let err = delete_hunt_evidence_at(&dir, id, None, Some(&salt), Some(&verifier)).unwrap_err();
    assert!(
        err.to_lowercase().contains("password") || err.to_lowercase().contains("sealed"),
        "sealed evidence delete without password must be refused: {err}"
    );
    assert!(enc.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sealed_evidence_delete_refused_with_wrong_password() {
    let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
    let err = delete_hunt_evidence_at(
        &dir,
        id,
        Some("wrong-password"),
        Some(&salt),
        Some(&verifier),
    )
    .unwrap_err();
    assert!(
        err.to_lowercase().contains("wrong password"),
        "wrong password must not delete sealed evidence: {err}"
    );
    assert!(enc.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sealed_evidence_delete_allowed_with_correct_password() {
    let (dir, id, salt, verifier, enc) = sealed_hunt_with_evidence("correct-ev-password");
    delete_hunt_evidence_at(
        &dir,
        id,
        Some("correct-ev-password"),
        Some(&salt),
        Some(&verifier),
    )
    .unwrap();
    assert!(!enc.exists());
    let _ = fs::remove_dir_all(&dir);
}

fn sealed_hunt_for_add(password: &str) -> (PathBuf, crypto::SessionKey, String, Vec<u8>) {
    let dir = std::env::temp_dir().join(format!("os-evadd-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(dir.join("evidence")).unwrap();
    let salt = crypto::generate_salt();
    let key = crypto::derive_key(password, &salt).unwrap();
    let verifier = crypto::create_password_verifier(&key).unwrap();
    seal::write_seal_marker(&dir, "00000000-0000-4000-8000-000000000088", "2024-01-01T00:00:00Z")
        .unwrap();
    HuntDatabase::open(dir.join("metadata.db")).unwrap();
    (dir, key, salt, verifier)
}

#[test]
fn sealed_evidence_add_refused_without_password() {
    let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
    let err = add_hunt_evidence_bytes_at(
        &dir,
        &key,
        "photo.jpg",
        b"new exhibit bytes",
        "sealed add",
        None,
        Some(&salt),
        Some(&verifier),
    )
    .unwrap_err();
    assert!(
        err.to_lowercase().contains("password") || err.to_lowercase().contains("sealed"),
        "sealed evidence add without password must be refused: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sealed_evidence_add_refused_with_wrong_password() {
    let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
    let err = add_hunt_evidence_bytes_at(
        &dir,
        &key,
        "photo.jpg",
        b"new exhibit bytes",
        "sealed add",
        Some("wrong-password"),
        Some(&salt),
        Some(&verifier),
    )
    .unwrap_err();
    assert!(
        err.to_lowercase().contains("wrong password"),
        "wrong password must not add sealed evidence: {err}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sealed_evidence_add_allowed_with_correct_password() {
    let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
    let status = add_hunt_evidence_bytes_at(
        &dir,
        &key,
        "photo.jpg",
        b"new exhibit bytes",
        "sealed add",
        Some("correct-add-password"),
        Some(&salt),
        Some(&verifier),
    )
    .unwrap();
    assert_eq!(status, "added");
    let entries: Vec<_> = fs::read_dir(dir.join("evidence"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.iter().any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("enc")),
        "correct password must write an encrypted exhibit"
    );
    let _ = fs::remove_dir_all(&dir);
}

fn evidence_row_count(dir: &std::path::Path) -> i64 {
    let conn = rusqlite::Connection::open(dir.join("metadata.db")).unwrap();
    conn.query_row("SELECT COUNT(*) FROM evidence", [], |row| row.get(0))
        .unwrap()
}

fn only_enc_file(dir: &std::path::Path) -> std::path::PathBuf {
    let mut encs: Vec<_> = fs::read_dir(dir.join("evidence"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("enc"))
        .collect();
    assert_eq!(encs.len(), 1, "expected exactly one .enc, got {encs:?}");
    encs.remove(0)
}

#[test]
fn sealed_evidence_duplicate_hash_does_not_overwrite() {
    let (dir, key, salt, verifier) = sealed_hunt_for_add("correct-add-password");
    let payload = b"identical sealed exhibit bytes";
    assert_eq!(
        add_hunt_evidence_bytes_at(
            &dir,
            &key,
            "photo.jpg",
            payload,
            "first add",
            Some("correct-add-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap(),
        "added"
    );
    let enc = only_enc_file(&dir);
    let before = fs::read(&enc).unwrap();
    assert!(!before.is_empty());
    assert_eq!(
        add_hunt_evidence_bytes_at(
            &dir,
            &key,
            "photo-copy.jpg",
            payload,
            "second add of the same bytes",
            Some("correct-add-password"),
            Some(&salt),
            Some(&verifier),
        )
        .unwrap(),
        "already_present"
    );
    assert_eq!(
        fs::read(&enc).unwrap(),
        before,
        "re-adding identical content must not rewrite the sealed .enc"
    );
    assert_eq!(only_enc_file(&dir), enc);
    assert_eq!(evidence_row_count(&dir), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unsealed_evidence_duplicate_hash_is_noop() {
    let dir = std::env::temp_dir().join(format!("os-evdup-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(dir.join("evidence")).unwrap();
    HuntDatabase::open(dir.join("metadata.db")).unwrap();
    let salt = crypto::generate_salt();
    let key = crypto::derive_key("unsealed-add-password", &salt).unwrap();
    let payload = b"same unsealed exhibit twice";
    assert_eq!(
        add_hunt_evidence_bytes_at(&dir, &key, "a.bin", payload, "one", None, None, None).unwrap(),
        "added"
    );
    let enc = only_enc_file(&dir);
    let before = fs::read(&enc).unwrap();
    assert_eq!(
        add_hunt_evidence_bytes_at(&dir, &key, "b.bin", payload, "two", None, None, None).unwrap(),
        "already_present"
    );
    assert_eq!(fs::read(&enc).unwrap(), before);
    assert_eq!(evidence_row_count(&dir), 1);
    let _ = fs::remove_dir_all(&dir);
}
