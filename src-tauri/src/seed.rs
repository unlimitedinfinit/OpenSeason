//! Sample case folders. Fake names only (Lucille Example).

use std::fs;
use std::path::Path;

use crate::case_profile::{self, Caption, CaseMode, CaseProfile, CourtInfo, Filer, Party, PleadingDraft};
use crate::guided::{self, GuidedDraft};

pub const LUCILLE_TITLE: &str = "Lucille Example v. Sample County Board";

pub fn lucille_profile() -> CaseProfile {
    let mut p = CaseProfile::new_standard(LUCILLE_TITLE, "reconsideration");
    p.id = "00000000-0000-4000-8000-000000000101".into();
    p.rule_pack_id = "nj-app-div".into();
    p.motion_kind = "reconsideration".into();
    p.court = CourtInfo {
        id: "nj-app-div".into(),
        name: "Superior Court of New Jersey, Appellate Division".into(),
        division: Some("Sample vicinage".into()),
        address_lines: Vec::new(),
    };
    p.docket_number = "A-01001-24".into();
    p.caption = Caption {
        plaintiffs: vec![Party {
            name: "Lucille Example".into(),
            role: "Appellant".into(),
            counsel: "Pro se".into(),
            address: "200 Sample Lane, Example City, ST 00000".into(),
            ..Default::default()
        }],
        defendants: vec![Party {
            name: "Sample County Board".into(),
            role: "Respondent".into(),
            counsel: "Counsel of record".into(),
            ..Default::default()
        }],
        appellants: vec![Party {
            name: "Lucille Example".into(),
            role: "Appellant".into(),
            counsel: "Pro se".into(),
            ..Default::default()
        }],
        appellees: vec![Party {
            name: "Sample County Board".into(),
            role: "Respondent".into(),
            ..Default::default()
        }],
    };
    p.filer = Filer {
        name: "Lucille Example".into(),
        role: "Appellant, proceeding without an attorney".into(),
        address_lines: vec!["200 Sample Lane".into(), "Example City, ST 00000".into()],
        phone: "555-0142".into(),
        email: "lucille.example@example.test".into(),
        signature_name: "Lucille Example".into(),
    };
    p.motion = PleadingDraft {
        title: "Motion for reconsideration".into(),
        user_text: String::new(),
    };
    p
}

pub fn seed_lucille(dest: &Path) -> Result<(std::path::PathBuf, CaseProfile), String> {
    if dest.join("case.json").exists() {
        return Err(format!(
            "A case already exists at {}. Pick an empty folder.",
            dest.display()
        ));
    }
    let (dir, mut profile) = case_profile::create_case_folder(dest, LUCILLE_TITLE, CaseMode::Standard, "reconsideration")?;
    let seeded = lucille_profile();
    profile.title = seeded.title.clone();
    profile.document_kind = seeded.document_kind.clone();
    profile.rule_pack_id = seeded.rule_pack_id.clone();
    profile.motion_kind = seeded.motion_kind.clone();
    profile.court = seeded.court.clone();
    profile.docket_number = seeded.docket_number.clone();
    profile.caption = seeded.caption.clone();
    profile.filer = seeded.filer.clone();
    profile.motion = seeded.motion.clone();
    case_profile::save_case(&dir, &profile)?;

    let mut draft = GuidedDraft {
        kind: "reconsideration".into(),
        sections: guided::templates_for("reconsideration"),
    };
    for section in &mut draft.sections {
        section.user_text = match section.id.as_str() {
            "order" => "The Appellate Division entered an order on January 9, 2026, dismissing the appeal as untimely.".into(),
            "rule" => "Relief is sought under R. 2:11-6. Confirm the current New Jersey Court Rules before relying on this.".into(),
            "grounds" => "The clerk mailed the order to the wrong address, which appears on the face of the returned envelope marked Exhibit A. This sentence is the user's own draft.".into(),
            "relief" => "Lucille Example asks the court to vacate the dismissal and restore the appeal. This app did not invent the request.".into(),
            _ => String::new(),
        };
    }
    guided::save(&dir, &draft)?;
    profile.motion.user_text = guided::assemble(&draft.sections);
    case_profile::save_case(&dir, &profile)?;

    fs::write(
        dir.join("evidence").join("returned-envelope.txt"),
        "FAKE EXHIBIT: Returned envelope marked not deliverable. Lucille Example only.\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        dir.join("evidence").join("returned-envelope.txt.txt"),
        "Ex. A. Returned envelope (fake).",
    )
    .map_err(|e| e.to_string())?;
    let _ = crate::exhibits::rebuild(&dir)?;
    let md = include_str!("../../fixtures/sample-lucille.md");
    let _ = crate::court_md::import_markdown(&dir, md);
    let loaded = case_profile::load_case(&dir)?;
    Ok((dir, loaded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn lucille_seed_is_obviously_fake() {
        let dir = std::env::temp_dir().join(format!("os-lucille-{}", Uuid::new_v4()));
        let (folder, profile) = seed_lucille(&dir).unwrap();
        assert_eq!(profile.title, LUCILLE_TITLE);
        assert!(profile.motion.user_text.contains("Exhibit A"));
        assert!(folder.join("evidence").join("returned-envelope.txt").is_file());
        assert!(folder.join("drafts").join("sections.json").is_file());
        let _ = fs::remove_dir_all(dir);
    }
}
