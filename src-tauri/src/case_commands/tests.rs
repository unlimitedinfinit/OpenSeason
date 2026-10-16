use std::fs;
use std::path::{Path, PathBuf};

use crate::case_profile;
use crate::db::HuntDatabase;
use super::filing_ops::export_markdown_draft_at;
use super::workspace::{list_case_artifacts_at, materialize_vault_case};

fn temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{prefix}-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn vault_only_hunt_evidence_appears_on_unified_case() {
    let hunt = temp_dir("os-vault-art");
    fs::create_dir_all(hunt.join("evidence")).unwrap();
    let db = HuntDatabase::open(hunt.join("metadata.db")).unwrap();
    db.conn
        .execute(
            "INSERT INTO evidence (description, file_path, encrypted_key_nonce, sha256_hash) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                "Invoice copy",
                "invoice.pdf",
                vec![0u8; 24],
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ],
        )
        .unwrap();
    let docs = temp_dir("os-docs-empty");
    let items = list_case_artifacts_at(&docs, Some(&hunt)).unwrap();
    assert!(
        items.iter().any(|a| a.kind == "evidence" && a.name == "Invoice copy"),
        "existing hunt exhibits must show on the unified case: {items:?}"
    );
    let _ = fs::remove_dir_all(&hunt);
    let _ = fs::remove_dir_all(&docs);
}

#[test]
fn materialize_unsealed_vault_hunt_writes_case_json() {
    let hunt = temp_dir("os-vault-open");
    HuntDatabase::open(hunt.join("metadata.db")).unwrap();
    let docs = temp_dir("os-docs-open").join("00000000-0000-4000-8000-000000000099");
    let profile = materialize_vault_case(
        &docs,
        &hunt,
        "00000000-0000-4000-8000-000000000099",
        "Relator v. Sample Contractor",
    )
    .unwrap();
    assert_eq!(profile.mode, crate::case_profile::CaseMode::Confidential);
    assert!(profile.sealed_at.is_none());
    assert!(docs.join("case.json").is_file());
    let again = materialize_vault_case(
        &docs,
        &hunt,
        "00000000-0000-4000-8000-000000000099",
        "ignored",
    )
    .unwrap();
    assert_eq!(again.title, "Relator v. Sample Contractor");
    let _ = fs::remove_dir_all(&hunt);
    let _ = fs::remove_dir_all(docs.parent().unwrap());
}

#[test]
fn export_markdown_draft_cmd_path_enforces_cos_date_and_verification_rules() {
    let root_ok = temp_dir("os-ui-md-ok");
    let root_incomplete = temp_dir("os-ui-md-inc");
    let root_leak = temp_dir("os-ui-md-leak");
    let root_dated = temp_dir("os-ui-md-dated");

    let nj = include_str!("../../../fixtures/sample-reconsideration.md");
    let ok = export_markdown_draft_at(&root_ok, nj.to_string(), None, None).unwrap();
    assert!(Path::new(&ok.pdf).is_file());
    assert!(Path::new(&ok.docx).is_file());
    let pdf = fs::read(&ok.pdf).unwrap_or_default();
    let pdf_lossy = String::from_utf8_lossy(&pdf).to_ascii_lowercase();
    assert!(!pdf_lossy.contains("i will serve"));
    assert!(!pdf_lossy.contains("needs_verification"));

    let incomplete = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
defendant: Sample County Clerk
case_no: A-01000-24
governing_state: NJ
---
# Body
Hello.
::: signature
/s/ Jordan Example
:::
::: certificate
I will serve a copy of this paper myself.
:::
"#;
    let err = export_markdown_draft_at(&root_incomplete, incomplete.into(), None, None).unwrap_err();
    assert!(err.contains("missing the recipient, method and service date"));
    assert!(err.contains("Add the recipient, method and service date"));

    let leak = r#"---
court: Colorado Court of Appeals
plaintiff: Jordan Example
defendant: Sample County Clerk
case_no: 24CA1000
governing_state: CO
---
# Body
Relief is sought under C.A.R. 40 (needs_verification).
::: signature
/s/ Jordan Example
:::
::: certificate
I certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail.
:::
"#;
    let err = export_markdown_draft_at(&root_leak, leak.into(), None, None).unwrap_err();
    assert!(err.contains("needs_verification"));
    assert!(err.contains("body line"));
    let preview = crate::court_md::preview_markdown(
        &case_profile::CaseProfile::new_standard("Preview", "motion"),
        leak,
    );
    assert!(
        preview.lines.join("\n").contains("needs_verification"),
        "preview must not silently rewrite the user's sentence"
    );
    assert!(preview.notice.contains("needs_verification"));

    let dated = r#"---
court: Superior Court of New Jersey, Appellate Division
plaintiff: Jordan Example
defendant: Sample County Clerk
case_no: A-01000-24
governing_state: NJ
---
# Body
Hello.
::: signature
/s/ Jordan Example
:::
::: certificate
I certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail.
:::
"#;
    let ok = export_markdown_draft_at(&root_dated, dated.into(), None, None).unwrap();
    let xml = {
        let zip = fs::File::open(&ok.docx).unwrap();
        let mut archive = zip::ZipArchive::new(zip).unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("word/document.xml").unwrap(), &mut s)
            .unwrap();
        s
    };
    assert!(xml.contains("I served a copy of this motion on Sample County Clerk by electronic mail"));
    assert!(!xml.to_ascii_lowercase().contains("i will serve"));
    let today = chrono::Local::now().format("%B %d, %Y").to_string();
    if today != "October 16, 2026" {
        assert!(
            !xml.contains(&today),
            "desktop export must not stamp today's date as the signing or service date"
        );
    }
    assert!(xml.contains("Date: ____________") || xml.contains("____________"));

    let _ = fs::remove_dir_all(&root_ok);
    let _ = fs::remove_dir_all(&root_incomplete);
    let _ = fs::remove_dir_all(&root_leak);
    let _ = fs::remove_dir_all(&root_dated);
}
