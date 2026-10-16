#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::case_profile::{sample_appeal_profile, REVIEW_NOTICE};
    use crate::legal_os::NJ_R_1_4_4B_OATH;
    use std::io::Read;

    #[test]
    fn sanitize_strips_markdown_fences_and_typst() {
        let md = "# Preliminary statement\nHello.\n::: signature\n/s/ Jordan Example\n:::\n#v(0.5em)\n::: certificate\nI certify that on ____________ I served a copy of this paper on ____________ by ____________.\n:::";
        let (body, sig, cert, _decl) = sanitize_draft(md);
        let headings: Vec<_> = body
            .iter()
            .filter_map(|b| match b {
                BodyBlock::Heading(h) => Some(h.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(headings, ["Preliminary statement"]);
        assert!(sig.iter().any(|s| s.contains("/s/ Jordan Example")));
        assert!(cert.iter().any(|s| s.contains("I served a copy of this paper")));
        let blob = format!("{body:?}");
        assert!(!blob.contains(":::"));
        assert!(!blob.contains("#v("));
        assert!(!blob.contains("# Preliminary"));
    }

    #[test]
    fn nj_motion_uses_verbatim_r_1_4_4b_and_omits_disclaimer() {
        let mut p = sample_appeal_profile();
        p.rule_pack_id = "nj-app-div".into();
        p.governing_state = "NJ".into();
        p.court.name = "Superior Court of New Jersey, Appellate Division".into();
        p.docket_number = "A-01000-24".into();
        p.notice_of_appeal.trial_court_docket = "L-01000-24".into();
        p.notice_of_appeal.trial_court_name =
            "Superior Court of New Jersey, Law Division, Sample County".into();
        p.motion.user_text = "# Preliminary statement\nThe trial court entered an order.\n::: signature\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "reconsideration", "Motion for reconsideration", &p.motion.user_text, CaptionStyle::Boxed);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains(NJ_R_1_4_4B_OATH));
        assert!(!lines.to_ascii_lowercase().contains("to the best of my knowledge"));
        assert!(!lines.contains(REVIEW_NOTICE));
        assert!(!lines.contains("not legal advice"));
        assert!(!contains_forbidden_markup(&lines));
        assert!(lines.contains("COURT USE ONLY") || lines.contains("Appellate"));
        assert!(lines.contains("A-01000-24") || lines.contains(&p.docket_number));
        assert!(lines.contains("CERTIFICATE OF SERVICE"));
        assert!(lines.contains("/s/ Jordan Example"));
        let cert_at = lines.find("CERTIFICATION IN LIEU OF OATH").unwrap();
        let sig_at = lines.find("/s/ Jordan Example").unwrap();
        let cos_at = lines.find("CERTIFICATE OF SERVICE").unwrap();
        assert!(cert_at < sig_at, "certification belongs above the signature");
        assert!(sig_at < cos_at, "certificate of service follows the signature");
        assert!(!lines.contains("Division / Courtroom"));
        let appellate_hits = lines.matches("Appellate Division").count();
        assert_eq!(appellate_hits, 1, "court name already says Appellate Division");
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("#pagebreak()"));
        assert!(!typst.contains("#pagebreak(weak"));
        assert!(typst.contains("breakable: false"));
        assert!(typst.contains("below: 0.85em"));
        assert!(typst.contains("leading: 0.85em"));
        assert!(typst.contains("spacing: 0.85em"));
        assert!(typst.contains("leading: 0.35em"));
        assert!(lines.contains("Case Number:"));
        assert!(lines.contains(DATE_BLANK));
        assert!(!lines.to_ascii_lowercase().contains("needs_verification"));
    }

    #[test]
    fn filer_address_and_email_appear_in_caption() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.court.name = "Superior Court of New Jersey, Appellate Division".into();
        p.court.division = Some("Appellate Division".into());
        p.filer.address_lines = vec!["100 Sample Street".into(), "Example City, ST 00000".into()];
        p.filer.email = "jordan.example@example.test".into();
        p.filer.phone = "555-0100".into();
        let fixture = include_str!("../../../fixtures/sample-reconsideration.md").replace("\r\n", "\n");
        p.motion.user_text = fixture
            .split("\n---\n")
            .last()
            .unwrap_or("The trial court entered an order.")
            .to_string();
        let doc = assemble(
            &p,
            "reconsideration",
            "Motion for reconsideration",
            &p.motion.user_text,
            CaptionStyle::Boxed,
        );
        assert!(doc.filer_address.iter().any(|a| a.contains("100 Sample Street")));
        assert_eq!(doc.filer_email, "jordan.example@example.test");
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("100 Sample Street"));
        assert!(lines.contains("jordan.example@example.test"));
        assert!(!lines.contains("Division / Courtroom"));
        assert_eq!(doc.court_division, "");
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("100 Sample Street"));
        assert!(
            typst.contains("jordan.example\\@example.test")
                || typst.contains("jordan.example@example.test")
        );
        assert!(!typst.contains("Division / Courtroom"));
    }

    #[test]
    fn co_motion_uses_lower_court_box_and_user_declaration() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.court.name = "Colorado Court of Appeals".into();
        p.court.division = Some("Court of Appeals".into());
        p.court.address_lines = vec!["2 East 14th Avenue".into(), "Denver, CO 80203".into()];
        p.docket_number = "24CA1000".into();
        p.notice_of_appeal.trial_court_name = "Denver District Court".into();
        p.notice_of_appeal.trial_court_docket = "23CV1000".into();
        p.caption.appellants[0].role = "Appellant".into();
        p.caption.appellees[0].role = "Appellee".into();
        p.filer.address_lines = vec!["100 Sample Street".into(), "Example City, ST 00000".into()];
        p.filer.email = "jordan.example@example.test".into();
        let fixture = include_str!("../../../fixtures/sample-co-reconsideration.md").replace("\r\n", "\n");
        p.motion.user_text = fixture
            .split("\n---\n")
            .last()
            .unwrap_or("The district court entered an order.")
            .to_string();
        let doc = assemble(
            &p,
            "motion",
            "Motion for reconsideration",
            &p.motion.user_text,
            CaptionStyle::Boxed,
        );
        assert_eq!(doc.venue, CaptionVenue::Colorado);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("24CA1000"));
        assert!(lines.contains("Denver District Court"));
        assert!(lines.contains("23CV1000"));
        assert!(lines.contains("Lower Court"));
        assert!(lines.contains("COURT USE ONLY"));
        assert!(lines.contains("I declare under penalty of perjury under the law of Colorado"));
        assert!(!lines.to_ascii_lowercase().contains("needs_verification"));
        assert!(!lines.contains("DECLARATION UNDER C.R.S."));
        assert!(!lines.contains("Certificate of Conferral (C.R.C.P. 121"));
        assert!(lines.contains("CERTIFICATE OF SERVICE"));
        assert!(lines.contains("/s/ Jordan Example"));
        assert!(lines.contains("100 Sample Street"));
        assert!(lines.contains("jordan.example@example.test"));
        assert_eq!(doc.court_division, "");
        let decl_at = lines.find("I declare under penalty of perjury").unwrap();
        let sig_at = lines.find("/s/ Jordan Example").unwrap();
        let cos_at = lines.find("CERTIFICATE OF SERVICE").unwrap();
        assert!(decl_at < sig_at);
        assert!(sig_at < cos_at);
        assert!(!contains_forbidden_markup(&lines));
        assert!(!lines.to_ascii_lowercase().contains("to the best of my knowledge"));
        assert!(!lines.contains(REVIEW_NOTICE));
        let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
        assert!(typst.contains("#pagebreak()"));
        assert!(!typst.contains("#pagebreak(weak"));
        assert!(typst.contains("leading: 0.85em"));
        assert!(lines.contains("Case Number:"));
        assert!(lines.contains("I certify that on October 2, 2026 I served a copy of this motion on Sample County Clerk by electronic mail."));
        assert!(!lines.to_ascii_lowercase().contains("i will serve"));
        assert!(doc.signature_lines.iter().any(|l| l == DATE_BLANK));
    }

    #[test]
    fn rendered_paper_omits_needs_verification() {
        for (state, court, md) in [
            (
                "NJ",
                "Superior Court of New Jersey, Appellate Division",
                include_str!("../../../fixtures/sample-reconsideration.md").replace("\r\n", "\n"),
            ),
            (
                "CO",
                "Colorado Court of Appeals",
                include_str!("../../../fixtures/sample-co-reconsideration.md").replace("\r\n", "\n"),
            ),
        ] {
            let mut p = sample_appeal_profile();
            p.governing_state = state.into();
            p.court.name = court.into();
            p.motion.user_text = md.split("\n---\n").last().unwrap_or(&md).to_string();
            let doc = assemble(
                &p,
                "motion",
                "Motion for reconsideration",
                &p.motion.user_text,
                CaptionStyle::Boxed,
            );
            let lines = document_lines(&doc).join("\n").to_ascii_lowercase();
            assert!(
                !lines.contains("needs_verification"),
                "{state} document_lines leaked needs_verification"
            );
            let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic()).to_ascii_lowercase();
            assert!(
                !typst.contains("needs_verification"),
                "{state} typst leaked needs_verification"
            );
        }
    }

    #[test]
    fn signature_date_is_blank_unless_user_entered_one() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        assert!(doc.signature_lines.iter().any(|l| l == DATE_BLANK));
        let today = chrono::Local::now().format("%B %d, %Y").to_string();
        assert!(
            !doc.signature_lines.iter().any(|l| l.contains(&today)),
            "export date must not be used as the signing date"
        );

        p.motion.user_text =
            "# Body\nHello.\n::: signature\nDate: January 2, 2026\n/s/ Jordan Example\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        assert!(doc.signature_lines.iter().any(|l| l.contains("January 2, 2026")));
        assert!(!doc.signature_lines.iter().any(|l| l == DATE_BLANK));
    }

    #[test]
    fn nj_and_co_share_court_use_only_and_case_number_label() {
        let mut nj = sample_appeal_profile();
        nj.governing_state = "NJ".into();
        nj.court.name = "Superior Court of New Jersey, Appellate Division".into();
        nj.docket_number = "A-01000-24".into();
        let nj_doc = assemble(&nj, "motion", "Motion", "Body.\n::: signature\n/s/ Jordan Example\n:::", CaptionStyle::Boxed);
        let nj_typst = to_typst(&nj_doc, &crate::court_rules::CourtRules::generic());
        assert!(nj_typst.contains("[COURT USE ONLY]"));
        assert!(!nj_typst.contains("COURT USE ONLY▲"));
        assert!(!nj_typst.contains("▲ COURT USE ONLY"));
        assert!(nj_typst.contains("Case Number:"));

        let mut co = sample_appeal_profile();
        co.governing_state = "CO".into();
        co.court.name = "Colorado Court of Appeals".into();
        co.docket_number = "24CA1000".into();
        let co_doc = assemble(&co, "motion", "Motion", "Body.\n::: signature\n/s/ Jordan Example\n:::", CaptionStyle::Boxed);
        let co_typst = to_typst(&co_doc, &crate::court_rules::CourtRules::generic());
        assert!(co_typst.contains("[COURT USE ONLY]"));
        assert!(!co_typst.contains("▲ COURT USE ONLY"));
        assert!(co_typst.contains("Case Number:"));
    }

    #[test]
    fn incomplete_certificate_is_fill_in_for_preview_and_blocks_export() {
        let mut p = sample_appeal_profile();
        p.governing_state = "NJ".into();
        p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::\n::: certificate\nI will serve this myself.\n:::".into();
        let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        let lines = document_lines(&doc).join("\n");
        assert!(lines.contains("I certify that on ____________ I served a copy of this motion on ____________ by ____________."));
        assert!(!lines.to_ascii_lowercase().contains("i will serve"));
        let err = require_complete_certificate(&doc).unwrap_err();
        assert!(err.contains("missing the recipient, method and service date"));
        assert!(err.contains("Add the recipient, method and service date"));

        p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::\n::: certificate\nI certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by ____________.\n:::".into();
        let missing_method = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        let err = require_complete_certificate(&missing_method).unwrap_err();
        assert!(err.contains("missing the method"));
        assert!(err.contains("Add the method"));
        assert!(!err.contains("recipient, method"));

        let fixture = include_str!("../../../fixtures/sample-reconsideration.md").replace("\r\n", "\n");
        p.motion.user_text = fixture
            .split("\n---\n")
            .last()
            .unwrap()
            .to_string();
        let complete = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
        require_complete_certificate(&complete).unwrap();
        let complete_lines = document_lines(&complete).join("\n");
        assert!(complete_lines.contains("I certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail."));
    }

    #[test]
    fn user_authored_needs_verification_is_not_rewritten_and_blocks_export() {
        let mut p = sample_appeal_profile();
        p.governing_state = "CO".into();
        p.court.name = "Colorado Court of Appeals".into();
        let md = "# Body\nRelief is sought under C.A.R. 40 (needs_verification).\n::: signature\n/s/ Jordan Example\n:::\n::: certificate\nI certify that on October 16, 2026 I served a copy of this motion on Sample County Clerk by electronic mail.\n:::";
        let doc = assemble(&p, "motion", "Motion", md, CaptionStyle::Boxed);
        let lines = document_lines(&doc).join("\n");
        assert!(
            lines.contains("needs_verification"),
            "preview must not silently rewrite the user's sentence"
        );
        let err = require_exportable(&doc, md).unwrap_err();
        assert!(err.contains("needs_verification"));
        assert!(err.contains("body line"));
        assert!(err.contains("will not rewrite"));
    }

    #[test]
    fn nj_and_co_body_is_left_aligned_ragged_right() {
        for (state, court) in [
            ("NJ", "Superior Court of New Jersey, Appellate Division"),
            ("CO", "Colorado Court of Appeals"),
        ] {
            let mut p = sample_appeal_profile();
            p.governing_state = state.into();
            p.court.name = court.into();
            p.motion.user_text = "# Body\nHello.\n::: signature\n/s/ Jordan Example\n:::".into();
            let doc = assemble(&p, "motion", "Motion", &p.motion.user_text, CaptionStyle::Boxed);
            let typst = to_typst(&doc, &crate::court_rules::CourtRules::generic());
            assert!(
                typst.contains("justify: false"),
                "{state} typst body must be ragged right"
            );
            assert!(
                !typst.contains("justify: true"),
                "{state} typst must not justify body text"
            );
            let docx = to_docx(&doc, &crate::court_rules::CourtRules::generic()).unwrap();
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
            let mut xml = String::new();
            archive
                .by_name("word/document.xml")
                .unwrap()
                .read_to_string(&mut xml)
                .unwrap();
            assert!(
                xml.contains("w:jc w:val=\"left\""),
                "{state} docx body must be left aligned"
            );
            assert!(
                !xml.contains("w:jc w:val=\"both\""),
                "{state} docx must not justify body text"
            );
        }
    }

    #[test]
    fn venue_language_marks_co_labels_needs_verification() {
        let raw = include_str!("../../../templates/venue-language.json");
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        assert_eq!(v["venues"]["NJ"]["oath"]["citation_status"], "pack_source");
        assert_eq!(v["venues"]["CO"]["oath"]["citation_status"], "needs_verification");
        assert_eq!(
            v["venues"]["CO"]["conferral"]["citation_status"],
            "needs_verification"
        );
        assert!(v["venues"]["CO"]["conferral"]["note"]
            .as_str()
            .unwrap()
            .contains("C.A.R."));
        assert!(v["venues"]["CO"]["oath"]["note"]
            .as_str()
            .unwrap()
            .contains("date and place"));
    }
}
