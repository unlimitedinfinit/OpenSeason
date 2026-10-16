import { sampleProfile } from "$lib/types/case";
import { saveStore, type Store } from "./mockStore";

export function handleFeatureOps<T>(cmd: string, args: Record<string, unknown>, store: Store): T | null {
  switch (cmd) {
    case "list_rule_packs":
      return [
        { id: "nj-app-div", name: "New Jersey Appellate Division", jurisdiction: "NJ" },
        { id: "co-mandamus", name: "Colorado original proceeding (C.A.R. 21)", jurisdiction: "CO" },
        { id: "generic-us-appellate", name: "Generic United States appellate", jurisdiction: "US" },
      ] as T;
    case "help_links_cmd":
      return {
        confirm: "This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?",
        offline_note: "Skip these links and the app still works offline.",
        source_note: "Browser preview uses the bundled JustLegal URLs.",
        verified_on: "2026-10-14",
        opens_browser: false,
        links: [
          { id: "find-legal-help", kind: "get-legal-help", label: "Get legal help", description: "Find free help.", url: "https://justlegal.me/find-legal-help", verified: true },
          { id: "knowledge-hub", kind: "find-resources", label: "Find resources", description: "Knowledge hub.", url: "https://justlegal.me/resources/knowledge-hub", verified: true },
          { id: "registry", kind: "online-database", label: "Online forms database", description: "Registry index.", url: "https://justlegal.me/registry/index.json", verified: true },
        ],
      } as T;
    case "guide_questions_cmd":
      return {
        disclaimer: "Legal information, not legal advice.",
        legal_information_not_advice: true,
        confirm_before_open: "This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?",
        help_links: [],
        questions: [
          { id: "state", prompt: "Which state?", why: "", choices: [{ id: "NJ", label: "NJ — New Jersey Appellate Division", detail: "" }] },
          { id: "court", prompt: "Which court?", why: "", choices: [{ id: "nj-app-div", label: "Appellate Division", detail: "" }] },
          { id: "situation", prompt: "What happened?", why: "", choices: [{ id: "reconsideration", label: "I lost and want the same court to look again", detail: "" }] },
          { id: "trigger_date", prompt: "Date?", why: "", choices: [] },
        ],
      } as T;
    case "guide_find_cmd":
      return {
        disclaimer: "Legal information, not legal advice.",
        never_files: true,
        legal_information_not_advice: true,
        state: "NJ",
        court: "Appellate Division",
        situation: "reconsideration",
        situation_label: "I lost and want the same court to look again",
        pack_id: "nj-app-div",
        why: "Browser preview of the NJ reconsideration situation.",
        forms: [
          {
            id: "nj-reconsideration-motion",
            title: "Motion for reconsideration",
            form_number: "",
            state: "NJ",
            court: "Appellate Division",
            situation: "reconsideration",
            size_bytes: 0,
            download_url: "",
            official_url: "",
            sha256: "",
            citation: "R. 2:11-6",
            citation_status: "pack_source",
            teach: {
              what_it_is: "A request that the same court look again.",
              when_used: "After an Appellate Division order is entered.",
              common_mistakes: ["Counting from the day you opened the mail."],
              learn_more: "Confirm R. 2:11-6 on the court site.",
            },
          },
        ],
        steps: [],
        deadline: args.triggerDate ? { due: "preview-only", rule: "R. 2:11-6", math: "Mocked" } : null,
        citation: "R. 2:11-6",
        citation_status: "pack_source",
        unverified: [],
        stuck: false,
        teach: { what_it_is: "", when_used: "", common_mistakes: [], learn_more: "" },
        help_links: [
          { id: "find-legal-help", kind: "get-legal-help", label: "Get legal help", description: "Find free help.", url: "https://justlegal.me/find-legal-help", verified: true },
        ],
        confirm_before_open: "This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?",
        notice: "Browser preview.",
      } as T;
    case "list_form_packs_cmd":
      return {
        packs_root: "Documents/JustLegal/Packs",
        catalog_id: "openseason-fixture",
        notice: "Fixture catalog. Official PDFs are not bundled.",
        available: [
          {
            id: "nj-appeal-fixture",
            state: "NJ",
            court: "Appellate Division",
            title: "New Jersey appeal fixture pack",
            situations: ["appeal"],
            size_bytes: 120,
            size_label: "120 B",
            sha256: "",
            installed: false,
            rule_pack_id: "nj-app-div",
            source_note: "FIXTURE",
            installable: true,
            unavailable_reason: "",
          },
        ],
        installed: [],
        rule_packs: [
          { id: "nj-app-div", name: "New Jersey Appellate Division", jurisdiction: "NJ", forms: 4, situations: 4 },
        ],
      } as T;
    case "install_form_pack_cmd":
      return {
        ok: true,
        pack_id: "nj-appeal-fixture",
        state: "NJ",
        bytes_planned: 120,
        size_label: "120 B",
        dest: "/mock/packs/nj-appeal-fixture",
        files: ["pack.json"],
        sha256_ok: true,
        offline_after: true,
        notice: "Browser preview install.",
      } as T;
    case "remove_form_pack_cmd":
      return "/mock/removed" as T;
    case "verify_form_pack_cmd":
      return [{ pack_id: "nj-appeal-fixture", ok: true }] as T;
    case "import_markdown_cmd":
      return { meta: { doc_title: "Imported draft" }, body: String(args.markdown || ""), lines: [] } as T;
    case "preview_markdown_cmd": {
      const md = String(args.markdown || "");
      const isCo = md.toLowerCase().includes("colorado");
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        title: isCo ? "Motion for reconsideration" : "Motion for reconsideration",
        caption_kind: isCo ? "co-boxed" : "nj-boxed",
        court_name: isCo ? "Colorado Court of Appeals" : "Superior Court of New Jersey, Appellate Division",
        docket_number: isCo ? "24CA1000" : "A-01000-24",
        trial_court: isCo ? "Denver District Court" : "Superior Court of New Jersey, Law Division, Sample County",
        trial_docket: isCo ? "23CV1000" : "L-01000-24",
        lines: [
          isCo ? "Colorado Court of Appeals" : "Superior Court of New Jersey, Appellate Division",
          "COURT USE ONLY",
          isCo ? "Case Number: 24CA1000" : "Appellate: A-01000-24",
          isCo ? "Lower Court: Denver District Court" : "Trial Court: L-01000-24",
          "MOTION FOR RECONSIDERATION",
          "Preliminary statement",
          "This is a fake draft for tests.",
          "/s/ Jordan Example",
          "CERTIFICATE OF SERVICE",
        ],
        has_signature: true,
        has_certificate: true,
        markup_stripped: true,
        notice: "Preview only. Nothing was filed or served.",
      } as T;
    }
    case "export_markdown_draft_cmd":
      return {
        docx: "Documents/JustLegal/Cases/preview/exports/court-md-A-01000-24.docx",
        pdf: "Documents/JustLegal/Cases/preview/exports/court-md-A-01000-24.pdf",
        notice: "Review every line before filing.",
      } as T;
    case "seed_sample_cmd": {
      const sample = sampleProfile();
      sample.id = "00000000-0000-4000-8000-000000000101";
      sample.title = "Lucille Example v. Sample County Board";
      sample.document_kind = "reconsideration";
      store.cases.unshift(sample);
      saveStore(store);
      return sample as T;
    }
    case "guided_sections_cmd":
      return {
        kind: "reconsideration",
        sections: [
          { id: "order", heading: "The order", hint: "Date and what it did.", user_text: "" },
          { id: "grounds", heading: "Grounds", hint: "Your own words.", user_text: "" },
        ],
      } as T;
    case "guided_fill_cmd":
    case "exhibit_index_cmd":
      return { never_files: true, items: [{ letter: "A", filename: "order.txt", description: "Dkt. 12", path: "/mock/A" }] } as T;
    case "link_fact_cmd":
    case "set_fact_cite_cmd":
    case "set_authority_cmd":
    case "set_signoff_cmd":
    case "export_packet_cmd":
    case "export_ics_cmd":
    case "gap_add_cmd":
    case "gap_export_cmd":
    case "field_help_cmd":
    case "playbook_cmd":
      return (cmd === "playbook_cmd" ? "Mock playbook" : cmd === "export_ics_cmd" || cmd === "gap_export_cmd" ? "/mock/out" : { ok: false, items: [], notes: [], checklist: [] }) as T;
    case "check_case_cmd":
    case "deadline_cmd":
      return {
        ok: false,
        never_files: true,
        never_serves: true,
        notice: "Browser preview uses mocked checks. Run the CLI or the desktop app for real results.",
        issues: [{ field: "preview", message: "Mocked invoke. Use npm run cli -- check all <case>." }],
        fact: { ok: false, never_files: true, rows: [], issues: [] },
        authority: { ok: false, never_files: true, invented: false, rows: [], issues: [] },
        deadline: args.triggerDate ? { due: "preview-only", math: "Mocked deadline", rule: String(args.rule || "") } : null,
        signoff: { ok: false, never_files: true, never_serves: true, items: [], issues: [] },
      } as T;
    case "sync_case_cmd":
      return {
        action: String(args.action || "backup"),
        mode: "standard",
        reason: "Account sync is not implemented in this build.",
        kind: "not_implemented",
      } as T;
    case "portfolio_status_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        total_matters: store.cases.length,
        global_filer: {
          name: "Jordan Example",
          role: "Proceeding without an attorney",
          address_lines: ["100 Sample Street", "Example City, ST 00000"],
          phone: "555-0100",
          email: "jordan.example@example.test",
          signature_name: "Jordan Example",
        },
        matters: store.cases.map((c) => ({
          case_id: c.id,
          title: c.title,
          matter_tier: c.matter_tier || "litigation",
          document_kind: c.document_kind,
          venue_code: c.governing_state || "NJ",
          docket_number: c.docket_number,
          completion_percent: 80,
          next_question_prompt: null,
          urgent_branch_title: "Motion for reconsideration",
          urgent_deadline: null,
          days_until_due: null,
          parent_matter_id: c.parent_matter_id || null,
          urgent_status: "needs_trigger_date",
          urgent_rule: "N.J. Ct. R. 4:49-2",
          urgent_trigger_date: null,
          urgent_math: null,
          unverified: true,
        })),
      } as T;
    case "get_global_filer_cmd":
    case "save_global_filer_cmd":
      return (args.filer || {
        name: "Jordan Example",
        role: "Proceeding without an attorney",
        address_lines: ["100 Sample Street", "Example City, ST 00000"],
        phone: "555-0100",
        email: "jordan.example@example.test",
        signature_name: "Jordan Example",
      }) as T;
    case "intake_status_cmd":
    case "intake_answer_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        case_id: String(args.idOrPath || "mock"),
        case_title: "Example Matter",
        matter_tier: "litigation",
        document_kind: "reconsideration",
        venue_code: "NJ",
        completed_steps: 4,
        total_steps: 5,
        completion_percent: 80,
        next_question: {
          id: "section:grounds",
          step_number: 5,
          category: "draft_section",
          prompt: "Why should the court reconsider (in your own words)?",
          why: "Explain what the court overlooked or misapprehended.",
          current_value: "",
          answered: false,
          choices: [],
          candidates: [],
        },
        questions: [],
        scan: { scanned_files: [], candidate_dockets: [], candidate_dates: [], candidate_courts: [] },
      } as T;
    case "intake_scan_cmd":
      return { scanned_files: [], candidate_dockets: [], candidate_dates: [], candidate_courts: [] } as T;
    case "case_next_steps_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: "Formatting and assembly engine only.",
        case_id: String(args.idOrPath || "mock"),
        case_title: "Example Matter",
        matter_tier: "litigation",
        venue_code: "NJ",
        detected_triggers: ["Order entered (2026-03-01)"],
        branches: [
          {
            id: "reconsideration",
            title: "Motion for Reconsideration",
            rule_citation: "NJ R. 4:49-2",
            trigger_event_title: "Order entered",
            trigger_date: "2026-03-01",
            trigger_kind: "service",
            base_days: 20,
            mail_offset_days: 0,
            clock_interaction: "pause_remaining",
            tolled_by_event: null,
            days_elapsed_before_toll: null,
            days_remaining_after_toll: null,
            computed_deadline: "2026-03-23",
            days_until_due: 14,
            status: "active",
            template_kind: "reconsideration",
            escalation_target: null,
            explanation: "20 days from service of order. Tolls (pauses) remaining appeal clock under R. 2:4-3(e).",
          },
        ],
      } as T;
    case "check_venue_competence_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        matter_tier: "litigation",
        document_kind: "reconsideration",
        venue: {
          venue_code: "NJ",
          display_name: "New Jersey State Courts (Superior / Appellate Division)",
          sworn_statement_title: "Certification Pursuant to R. 1:4-4(b)",
          sworn_statement_citation: "N.J. Ct. R. 1:4-4(b)",
          mandatory_oath_text: "I certify that the foregoing statements made by me are true. I am aware that if any of the foregoing statements made by me are willfully false, I am subject to punishment.",
          requires_conferral_certificate: false,
          conferral_rule: "Not required for standard NJ motions",
          appeal_days: 45,
          reconsideration_days: 20,
          reconsideration_trigger: "service",
          tolling_mode: "pause_remaining",
          mail_service_adder_days: 3,
          packet_manifest: ["Notice of Motion", "Certification (R. 1:4-4(b))", "Brief", "Proposed Order", "Certificate of Service"],
        },
        issues: [],
        recommendations: [],
      } as T;
    case "escalate_case_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        parent_case_id: String(args.idOrPath || "mock"),
        new_case_dir: "/mock/escalated",
        new_profile: sampleProfile(),
        copied_record_files: 2,
        message: "Escalated case created.",
      } as T;
    case "agent_manifest_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: "This app never files or serves.",
        case_folder: "Documents/JustLegal/Cases/{caseId}/",
        mcp_launch: { command: "openseason", args: ["mcp"], cwd: "(repository root)" },
        tools: [
          { name: "playbook", cli: "docs playbook", mcp: "playbook", summary: "Read the offline motion playbook first." },
          { name: "import_folder", cli: "case import-folder", mcp: "import_folder", summary: "Copy drafts and orders. Does not auto-commit guesses." },
        ],
        starter_prompt: "You are helping with OpenSeason. Never invent facts, citations, or deadlines. Never file or serve.",
      } as T;
    case "import_folder_cmd":
      return {
        ok: true,
        never_files: true,
        never_serves: true,
        notice: "Copied files. Proposed names were NOT written to case.json.",
        source_folder: String(args.from || ""),
        case_dir: "Documents/JustLegal/Cases/imported",
        case_id: store.cases[0]?.id || "00000000-0000-4000-8000-000000000001",
        copied_files: [],
        proposals: { party_names: [], docket_numbers: [], order_dates: [], court_names: [] },
        auto_committed: false,
      } as T;
    case "pdf_extract_text_cmd":
      return { ok: true, never_files: true, path: String(args.pdfPath || ""), page_count: 1, total_chars: 42, pages: [{ page_number: 1, text: "Sample extracted text" }], full_text: "Sample extracted text" } as T;
    case "record_inventory_cmd":
    case "record_docket_match_cmd":
      return {
        case_id: String(args.caseId || "mock"),
        court_name: "Superior Court of New Jersey - Law Division",
        docket_number: "ESX-L-001234-24",
        total_files_scanned: 4,
        tier1_official_count: 2,
        tier2_hearing_count: 1,
        tier3_archive_count: 1,
        tier4_unsubmitted_count: 0,
        entries: [
          {
            id: "rec-1",
            filename: "Order_Dismissing_Complaint.pdf",
            relative_path: "orders/Order_Dismissing_Complaint.pdf",
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            size_bytes: 142500,
            tier: "official_court_record",
            docket_date: "2026-02-15",
            filing_stamp_detected: true,
            source_provenance: "Clerk Stamp Verified",
            title: "Order Granting Motion to Dismiss",
            notes: "Filed by Clerk on 02/15/2026",
            verified: true,
          },
          {
            id: "rec-2",
            filename: "Hearing_Transcript_2026-02-10.pdf",
            relative_path: "evidence/Hearing_Transcript_2026-02-10.pdf",
            sha256: "b4c8996fb92427ae41e4649b934ca495991b7852b855e3b0c44298fc1c149afb",
            size_bytes: 845200,
            tier: "hearing_and_evidentiary_record",
            docket_date: "2026-02-10",
            filing_stamp_detected: false,
            source_provenance: "Certified Court Reporter Transcription",
            title: "Oral Argument Transcript",
            notes: "Certified stenographic transcript",
            verified: true,
          },
        ],
        missing_records: [
          {
            id: "missing-1",
            title: "Certified Hearing Audio / Transcript (Motion Hearing)",
            record_type: "transcript",
            reason: "Judicial finding references oral colloquy not reflected in written order",
            docket_entry_ref: "Entry #14",
            recommended_action: "Request audio CD or certified transcript from Vicinage Transcript Office",
            estimated_cost_notice: "NJ Judiciary standard rates: $3.00/page overnight, $1.50/page 30-day",
            request_form: "NJ Form CN 10200",
          },
        ],
        reconciliation_summary: "4 items scanned across 4 tiers. 1 potential gap identified.",
      } as T;
    case "record_missing_cmd":
      return [
        {
          id: "missing-1",
          title: "Certified Hearing Audio / Transcript (Motion Hearing)",
          record_type: "transcript",
          reason: "Judicial finding references oral colloquy not reflected in written order",
          docket_entry_ref: "Entry #14",
          recommended_action: "Request audio CD or certified transcript from Vicinage Transcript Office",
          estimated_cost_notice: "NJ Judiciary standard rates: $3.00/page overnight, $1.50/page 30-day",
          request_form: "NJ Form CN 10200",
        },
      ] as T;
    case "record_request_form_cmd":
      return {
        state: "NJ",
        court_name: "Superior Court of New Jersey - Law Division",
        division: "Civil / Law Division",
        docket_number: "ESX-L-001234-24",
        caption: "Jordan Example v. Sample County Clerk",
        requester_name: "Jordan Example",
        requester_address: "123 Main St, Newark, NJ 07102",
        requester_phone: "973-555-0100",
        requester_email: "jordan@example.com",
        records_requested: ["Certified Hearing Audio / Transcript (Motion Hearing)"],
        preferred_format: "Electronic (Email / PDF link)",
        statutory_fee_notice: "NJ Judiciary Form CN 10200 rates: $0.05/letter page, $0.07/legal page.",
        submission_method: "Email to Vicinage Records Custodian or eCourts portal.",
        instructions: "Submit Form CN 10200 to Court Clerk. Fees payable upon fulfillment.",
        rendered_request_text: "NEW JERSEY JUDICIARY RECORDS REQUEST FORM (CN 10200)...",
      } as T;
    case "accountability_audit_cmd":
      return {
        case_id: String(args.caseId || "mock"),
        court_name: "Superior Court of New Jersey - Law Division",
        issues: [
          {
            id: "iss-1",
            title: "Finding of 'No Timely Opposition Filed' Disproven by Filed Stamped Brief",
            date_identified: "2026-03-01",
            finding_source: {
              document_id: "rec-1",
              document_name: "Order_Dismissing_Complaint.pdf",
              page: 2,
              quoted_text: "Plaintiff failed to file written opposition within the statutory return window.",
            },
            record_support_status: "contradicted_by_record",
            contradicting_evidence_local: ["evidence/Stamped_Opposition_Brief.pdf"],
            contradicting_evidence_filed: ["Stamped Opposition Brief filed 01/14/2026"],
            analysis_notes: "The court's finding is contradicted by the docket record showing timely receipt by clerk.",
            available_routes: [
              {
                route_kind: "reconsideration_interlocutory",
                title: "Interlocutory Reconsideration (R. 4:42-2 - Lawson v. Dewar)",
                rule_citation: "N.J. Ct. R. 4:42-2; Lawson v. Dewar, 468 N.J. Super. 128 (App. Div. 2021)",
                governing_standard: "Interlocutory orders may be reviewed at any time prior to final judgment in the interests of justice.",
                deadline_notice: "Any time before final judgment; not strictly barred by R. 4:49-2 20-day limit.",
                is_disciplinary: false,
                warning_notice: "Filing a motion for reconsideration tolls or alters review deadlines. File in the trial court.",
                official_form_name: "Motion for Reconsideration",
                submission_portal_url: "https://www.njcourts.gov/public/reconsideration",
              },
            ],
            resolved: false,
          },
        ],
        non_tolling_warning: "IMPORTANT: Disciplinary or administrative filings DO NOT toll, stay, or extend court appeal deadlines.",
        summary: "1 discrepancy identified between judicial findings and verified record evidence.",
      } as T;
    case "accountability_routes_cmd":
      return [
        {
          route_kind: "reconsideration_interlocutory",
          title: "Interlocutory Reconsideration (R. 4:42-2 - Lawson v. Dewar)",
          rule_citation: "N.J. Ct. R. 4:42-2; Lawson v. Dewar, 468 N.J. Super. 128 (App. Div. 2021)",
          governing_standard: "Interlocutory orders may be reviewed at any time prior to final judgment in the interests of justice.",
          deadline_notice: "Any time before final judgment; not strictly barred by R. 4:49-2 20-day limit.",
          is_disciplinary: false,
          warning_notice: "Filing a motion for reconsideration tolls or alters review deadlines. File in the trial court.",
          official_form_name: "Motion for Reconsideration",
          submission_portal_url: "https://www.njcourts.gov/public/reconsideration",
        },
      ] as T;
    case "accountability_resources_cmd":
      return [
        {
          id: "nj-acjc-11166",
          title: "New Jersey ACJC Judicial Misconduct Complaint",
          agency: "Advisory Committee on Judicial Conduct",
          category: "judicial",
          state: "NJ",
          form_number: "CN 11166",
          direct_pdf_url: "https://www.njcourts.gov/sites/default/files/forms/11166_acjc_complaint_form.pdf",
          portal_url: "https://www.njcourts.gov/public/acjc",
          fee_schedule: "No filing fee.",
          jurisdictional_scope: "Investigates ethical violations by NJ state judges (bias, conflict of interest, harassment, improper ex parte communications). Cannot change rulings or overturn orders.",
          non_appeal_warning: "DOES NOT TOLL COURT APPEAL DEADLINES. An adverse ruling or factual error is an appellate issue, NOT judicial misconduct.",
        },
      ] as T;
    case "accountability_packet_cmd":
      return {
        case_id: String(args.caseId || "mock"),
        title: "Objective Factual Record Discrepancy Packet",
        issue_id: String(args.issueId || "iss-1"),
        date_generated: "2026-03-01",
        factual_chronology: ["Order Dismissing entered 02/15/2026", "Opposition Brief stamped 01/14/2026"],
        record_provenance: ["Order SHA256: e3b0c4...", "Opposition SHA256: b4c899..."],
        rendered_markdown: "# Factual Record Discrepancy Exhibit Packet\n\n...",
        notice: "Contains verified SHA-256 hashes and docket timestamps. Never file emotional rhetoric.",
      } as T;
    case "ada_petition_cmd":
      return {
        state: "NJ",
        court_name: "Superior Court of New Jersey - Law Division",
        docket_number: "ESX-L-001234-24",
        ada_coordinator_title: "Vicinage ADA Title II Coordinator / Trial Court Administrator",
        statutory_authorities: [
          "Title II of the Americans with Disabilities Act, 42 U.S.C. § 12132",
          "28 C.F.R. Part 35 (Nondiscrimination on the Basis of Disability in State and Local Government Services)",
          "NJ Judiciary Directive #01-19 & Rule 1:33",
        ],
        rendered_petition_text: "PETITION FOR REASONABLE ACCOMMODATION UNDER TITLE II OF THE AMERICANS WITH DISABILITIES ACT...",
        submission_guidelines: "Submit directly to the Vicinage ADA Coordinator. Confidential under federal regulations.",
      } as T;
    case "order_decode_cmd":
      return {
        title: "Order to Show Cause",
        court_name: "Colorado Court of Appeals",
        docket_number: "2026CA765",
        order_date: "2026-10-06",
        signer: { kind: "ByTheCourt", detail: { division: "Screening / Motions Division" } },
        is_boilerplate_notice: false,
        operative_clauses: [
          "ORDERED that appellant shall show cause in writing within 21 days of the date of this order why this appeal should not be dismissed for lack of appellate jurisdiction under C.A.R. 4(a).",
        ],
        candidate_response_window_days: 21,
        candidate_deadline: "2026-10-27",
        governing_rules_cited: ["C.A.R. 4(a)", "C.R.C.P. 59"],
        status: "needs_verification",
        uncertainty_notes: [
          "Order signed 'BY THE COURT': Originated from staff attorneys/screening division, not a single named judge.",
          "Candidate deadline unverified: Confirm whether response window counts calendar or court business days.",
        ],
        source_page: 2,
        suggested_actions: [
          "Review governing rule and check tolling exceptions",
          "Check the existing record for postmark vs. clerk receipt dates",
          "Explore show-cause response options",
          "Prepare response to show cause",
        ],
      } as T;
    case "timeline_audit_mail_cmd":
      return {
        case_id: String(args.caseId || "example-co-appeal"),
        order_entry_date: String(args.orderEntryDate || "2026-10-06"),
        mailing_postmark: "2026-10-08",
        delivery_receipt_date: "2026-10-15",
        clerk_receipt_date: "2026-10-16",
        jurisdiction_receipt_rule: "physical_clerk_receipt",
        mail_delay_days: 7,
        holiday_or_weekend_impact: true,
        jurisdictional_traps: [
          "Colorado Physical Clerk Receipt Rule: Colorado courts require physical clerk receipt, not postmark dates.",
          "Dobler v. CDOC (Colo. App. 2026) Tolling Trap: An untimely Rule 59 postjudgment motion fails to toll the 49-day notice of appeal clock under C.A.R. 4(a).",
          "Unique Circumstances Doctrine Strictly Limited (Estate of Ybarra, 2024): Discretionary exceptions cannot salvage missed jurisdictional deadlines.",
        ],
        recommended_actions: [
          "CRITICAL: 8-day gap detected between postmark and clerk receipt. Verify whether district court docket shows timely receipt.",
          "Order reached litigant 7 days after entry. Promptly file a Motion for Enlargement of Time under C.A.R. 26(b) citing mail transmission delay.",
        ],
        status: "flagged_risk",
      } as T;
    case "case_bootstrap_cmd":
      return {
        case_id: "case-rescued-001",
        case_path: "Documents/JustLegal/Cases/case-rescued-001",
        title: String(args.caseTitle || "Rescued Matter"),
        files_scanned: 6,
        orders_imported: 2,
        filings_imported: 2,
        evidence_imported: 1,
        drafts_imported: 1,
        detected_docket_number: "2026CA765",
        detected_court: "Colorado Court of Appeals",
        pending_obligations: ["2 court order(s) imported: decode orders to verify candidate deadlines."],
        summary: "Successfully bootstrapped case with 6 files (2 orders, 2 filings, 1 exhibits, 1 drafts).",
      } as T;
    case "court_access_directory_cmd":
      return [
        {
          court_id: "nj-app-div",
          court_name: "Superior Court of New Jersey, Appellate Division",
          jurisdiction_state: "NJ",
          physical_address: "Richard J. Hughes Justice Complex, 25 W. Market St., Trenton, NJ 08625",
          clerk_counter_hours: "Monday through Friday, 8:30 a.m. to 4:30 p.m. ET",
          phone: "(609) 815-2950",
          electronic_portal_name: "JEDS / eCourts",
          electronic_portal_url: "https://www.njcourts.gov/public/jeds",
          pro_se_efiling_allowed: true,
          docket_search_url: "https://www.njcourts.gov/public/case-search",
          records_access_rule: "New Jersey Court Rule 1:38",
          transcript_request_form: "NJ Judiciary Form CN 10200",
          transcript_deposit_rules: "R. 2:5-3(d) estimated transcript deposit required",
          statutory_fees_summary: "5¢/page letter, 7¢/page legal; free electronic transmission",
          indigent_fee_waiver_available: true,
          special_instructions: ["Submit Form CN 10200 to Appellate Clerk or through JEDS."],
        },
      ] as T;
    case "court_access_profile_cmd":
      return {
        court_id: "nj-app-div",
        court_name: "Superior Court of New Jersey, Appellate Division",
        jurisdiction_state: "NJ",
        physical_address: "Richard J. Hughes Justice Complex, 25 W. Market St., Trenton, NJ 08625",
        clerk_counter_hours: "Monday through Friday, 8:30 a.m. to 4:30 p.m. ET",
        phone: "(609) 815-2950",
        electronic_portal_name: "JEDS / eCourts",
        electronic_portal_url: "https://www.njcourts.gov/public/jeds",
        pro_se_efiling_allowed: true,
        docket_search_url: "https://www.njcourts.gov/public/case-search",
        records_access_rule: "New Jersey Court Rule 1:38",
        transcript_request_form: "NJ Judiciary Form CN 10200",
        transcript_deposit_rules: "R. 2:5-3(d) transcript deposit required",
        statutory_fees_summary: "5¢/letter page, 7¢/legal page; free electronic transmission",
        indigent_fee_waiver_available: true,
        special_instructions: ["Submit Form CN 10200 to Appellate Clerk or through JEDS."],
      } as T;
    case "record_retrieval_plan_cmd":
      return {
        case_id: String(args.caseId || "example-case"),
        court_id: "nj-app-div",
        court_name: "Superior Court of New Jersey, Appellate Division",
        missing_records: [
          {
            title: "Hearing Transcript (Oral Argument)",
            reason: "Required to demonstrate issue preservation",
            record_tier: "Tier2Hearing",
            target_agency_or_court: "Superior Court of New Jersey, Appellate Division",
            required_form: "NJ Judiciary Form CN 10200",
          },
          {
            title: "Official Docket Register of Actions",
            reason: "Verify entered orders and dates",
            record_tier: "Tier1Official",
            target_agency_or_court: "Superior Court of New Jersey, Appellate Division",
            required_form: "CN 10200 Records Request",
          },
        ],
        recommended_form: "NJ Judiciary Form CN 10200",
        recommended_form_number: "CN 10200",
        submission_channels: [
          "Electronic Portal: JEDS (https://www.njcourts.gov/public/jeds)",
          "Clerk Counter: Richard J. Hughes Justice Complex, 25 W. Market St., Trenton, NJ (8:30 a.m. - 4:30 p.m. ET)",
        ],
        estimated_cost_explanation: "5¢/page letter, 7¢/page legal; free electronic transmission under CN 10200",
        fee_waiver_rule: "R. 1:13-2 (Indigent Filer Fee Waiver Application)",
        followup_reminder_days: 14,
        step_by_step_instructions: [
          "1. Verify case number and caption.",
          "2. Select requested records.",
          "3. Submit via JEDS or mail Form CN 10200.",
          "4. Track 14-day clerk response window.",
        ],
      } as T;
    case "record_retrieval_execute_cmd":
      return {
        state: "NJ",
        court_name: "Superior Court of New Jersey, Appellate Division",
        division: "Appellate Division",
        docket_number: "A-001234-24",
        caption: "Example v. Clerk",
        requester_name: "Jordan Example",
        requester_address: "123 Main St, Trenton, NJ 08608",
        requester_phone: "555-0199",
        requester_email: "jordan@example.com",
        records_requested: [
          "Hearing Transcript (Oral Argument)",
          "Official Docket Register of Actions",
        ],
        preferred_format: "Electronic (Email / PDF link)",
        statutory_fee_notice: "5¢/letter page; free electronic transmission.",
        submission_method: "JEDS / Clerk Office",
        instructions: "Submit to Trial Court Administrator or Appellate Clerk.",
        rendered_request_text: "NEW JERSEY JUDICIARY RECORDS REQUEST FORM (CN 10200)\nDocket: A-001234-24\nRequester: Jordan Example\nItems: Hearing Transcript, Official Docket Sheet",
      } as T;

    default:
      return null;
  }
}
