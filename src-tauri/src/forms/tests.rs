//! Tests for forms catalog, pack installation, search, verification, and case attachment.

use std::fs;
use std::path::{Path, PathBuf};

use super::installer::*;
use super::library::*;
use super::registry::*;
use super::types::*;

fn temp() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("os-packs-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn fixture_catalog_uses_fake_urls() {
    let cat = bundled_catalog().unwrap();
    assert!(cat.source_note.contains("FIXTURE"));
    for pack in &cat.packs {
        assert!(
            pack.download_url.contains("example.test") || pack.download_url.is_empty(),
            "{}",
            pack.download_url
        );
    }
}

#[test]
fn install_local_fixture_verifies_sha256() {
    let root = temp();
    let fixture = repo_fixtures_catalog_dir().join("packs/nj-appeal");
    let report = install(InstallOpts {
        state: "NJ".into(),
        situation: Some("appeal".into()),
        catalog: None,
        file: Some(fixture),
        root: Some(root.clone()),
        allow_network: false,
    })
    .unwrap();
    assert!(report.ok);
    assert!(report.bytes_planned > 0);
    assert!(report.offline_after);
    assert!(report.files.iter().any(|f| f.contains("notice-of-appeal")));
    let reports = verify_pack(Some("nj-appeal-fixture"), Some(&root)).unwrap();
    assert!(reports[0].ok, "{reports:?}");
    assert!(reports[0].checked.iter().all(|r| r.ok));
    let removed = remove_pack("NJ", Some(&root)).unwrap();
    assert!(!Path::new(&removed).exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn zip_roundtrip_checks_hash() {
    let root = temp();
    let fixture = repo_fixtures_catalog_dir().join("packs/nj-appeal");
    let zip_path = root.join("nj-appeal-fixture.zip");
    let sha = write_zip_from_dir(&fixture, &zip_path).unwrap();
    assert_eq!(sha.len(), 64);
    let dest = root.join("out");
    let n = download_to(&zip_path.to_string_lossy(), &dest, &sha).unwrap();
    assert!(n > 0);
    assert!(dest.join("pack.json").is_file());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn justlegal_nj_sample_adapts() {
    let sample = r#"{
        "state":"NJ","stateName":"New Jersey","collectedAt":"2026-09-22",
        "forms":[{"id":"nj-cn-10837","title":"How to Appeal","formNumber":"CN 10837",
            "appliesToCourt":"Appellate Division","task":"appeal",
            "sourcePageUrl":"https://www.njcourts.gov/self-help/forms",
            "directFileUrl":"https://www.njcourts.gov/sites/default/files/forms/10837_appl_prose_kit.pdf",
            "mirror":{"url":"/mirrors/files/nj-cn-10837.pdf","sha256":"abc","bytes":10}}]
        ,"packets":[{"id":"nj-appeal","task":"appeal","title":"Appeal","court":"Appellate Division"}]
    }"#;
    let cat = justlegal_state_to_catalog(sample).unwrap();
    assert_eq!(cat.forms[0].id, "nj-cn-10837");
    assert_eq!(
        cat.forms[0].download_url,
        "https://justlegal.me/mirrors/files/nj-cn-10837.pdf"
    );
    assert_eq!(cat.forms[0].size_bytes, 10);
    assert_eq!(cat.packs[0].size_bytes, 10);
}

#[test]
fn network_refuses_example_test_urls() {
    let root = temp();
    let catalog_path = root.join("remote.json");
    fs::write(
        &catalog_path,
        r#"{
          "schema":"openseason.forms-catalog/v1",
          "id":"remote-fake",
          "never_files":true,
          "never_serves":true,
          "packs":[{
            "id":"xx-fake",
            "state":"XX",
            "title":"Fake",
            "size_bytes":12,
            "download_url":"https://example.test/packs/xx.zip",
            "sha256":"00",
            "situations":["appeal"]
          }]
        }"#,
    )
    .unwrap();
    let err = install(InstallOpts {
        state: "XX".into(),
        situation: Some("appeal".into()),
        catalog: Some(catalog_path),
        file: None,
        root: Some(root.clone()),
        allow_network: true,
    })
    .unwrap_err();
    assert!(
        err.contains("fixture")
            || err.contains("example.test")
            || err.contains("no real download source")
            || err.contains("--file"),
        "{err}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn install_bundled_fixture_without_file_flag() {
    let root = temp();
    let report = install(InstallOpts {
        state: "NJ".into(),
        situation: Some("appeal".into()),
        catalog: None,
        file: None,
        root: Some(root.clone()),
        allow_network: false,
    })
    .expect("bundled NJ fixture should copy from local_dir without a network fetch");
    assert!(report.ok);
    assert!(report.offline_after);
    assert!(report.files.iter().any(|f| f.contains("notice-of-appeal") || f.contains("pack.json")));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn catalog_pack_without_source_or_hash_is_unavailable() {
    let pack = CatalogPack {
        id: "xx-empty".into(),
        state: "XX".into(),
        court: "Sample".into(),
        title: "Empty pack".into(),
        situations: vec!["appeal".into()],
        size_bytes: 0,
        sha256: String::new(),
        download_url: String::new(),
        local_dir: String::new(),
        rule_pack_id: String::new(),
        source_id: String::new(),
    };
    let reason = pack_unavailable_reason(&pack, None);
    assert!(!reason.is_empty());
    assert!(!pack_is_installable(&pack, None));
}

#[test]
fn forms_search_and_recommendation_vertical_slice() {
    let hits = search_forms("appeal", Some("NJ"), None, None).expect("search should work");
    assert!(!hits.is_empty(), "Should find NJ appeal forms");
    assert!(hits.iter().any(|h| h.id.contains("appeal") || h.title.to_ascii_lowercase().contains("appeal")));

    // Test case recommendation
    let tmp = temp();
    let case_dir = tmp.join("test-case");
    let mut profile = crate::case_profile::CaseProfile::new_standard("Vertical Test v. State", "appellate_appeal");
    profile.governing_state = "NJ".into();
    profile.court.name = "Superior Court of New Jersey, Appellate Division".into();
    profile.docket_number = "A-001234-25".into();
    crate::case_profile::save_case(&case_dir, &profile).expect("case save should work");
    assert!(case_dir.join("case.json").is_file());

    let recs = recommend_forms_for_case(&case_dir, None).expect("recommendations should succeed");
    assert_eq!(recs.state, "NJ");
    assert!(!recs.recommendations.is_empty(), "Should recommend forms for NJ appellate appeal");

    // Test attach form as template to case drafts
    let first_form_id = &recs.recommendations[0].form.id;
    let attach_res = attach_form_to_case(first_form_id, &case_dir, None, true).expect("attach as template should work");
    assert!(attach_res.ok);
    assert_eq!(attach_res.action, "draft_created");
    assert!(Path::new(&attach_res.attached_path).is_file());
    let draft_content = fs::read_to_string(&attach_res.attached_path).unwrap();
    assert!(draft_content.contains("Vertical Test"));

    let _ = fs::remove_dir_all(tmp);
}
