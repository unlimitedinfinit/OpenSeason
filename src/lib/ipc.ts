import { sampleProfile, type CaseArtifact, type CaseProfile, type ExportResult, type ValidationReport, type WorkspaceCase } from "$lib/types/case";
import { session } from "$lib/session.svelte";

function isTauri(): boolean {
  return typeof window !== "undefined" && !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
}

type Store = {
  cases: CaseProfile[];
  artifacts: Record<string, CaseArtifact[]>;
  vaultCases: string[];
  saltReady: boolean;
};

function loadStore(): Store {
  if (typeof localStorage === "undefined") {
    return { cases: [sampleProfile()], artifacts: {}, vaultCases: [], saltReady: true };
  }
  const raw = localStorage.getItem("openseason-mock");
  if (!raw) {
    const sample = sampleProfile();
    const confidential = sampleProfile();
    confidential.id = "00000000-0000-4000-8000-000000000002";
    confidential.title = "Relator v. Sample Contractor";
    confidential.mode = "confidential";
    confidential.document_kind = "complaint";
    const initial: Store = {
      cases: [sample, confidential],
      artifacts: {
        [sample.id]: [
          { kind: "evidence", name: "order-granting-dismissal.pdf", path: "/mock/evidence/order-granting-dismissal.pdf" },
          { kind: "export", name: "notice-of-appeal-24-01000.docx", path: "/mock/exports/notice-of-appeal-24-01000.docx" },
        ],
      },
      vaultCases: [confidential.id],
      saltReady: true,
    };
    localStorage.setItem("openseason-mock", JSON.stringify(initial));
    return initial;
  }
  return JSON.parse(raw) as Store;
}

function saveStore(store: Store) {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("openseason-mock", JSON.stringify(store));
  }
}

function workspaceOf(profile: CaseProfile, store: Store): WorkspaceCase {
  return {
    profile,
    source: store.vaultCases.includes(profile.id) && profile.mode === "confidential" ? "vault" : "documents",
    has_vault: store.vaultCases.includes(profile.id) || profile.mode === "confidential",
    sealed: !!profile.sealed_at,
    folder: `Documents/JustLegal/Cases/${profile.id}`,
  };
}

async function mockInvoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const store = loadStore();
  switch (cmd) {
    case "get_cases_root":
      return "Documents/JustLegal/Cases" as T;
    case "list_workspace_cases":
    case "list_cases": {
      const list = store.cases.map((p) => workspaceOf(p, store));
      return (cmd === "list_cases" ? store.cases : list) as T;
    }
    case "create_case": {
      const title = String(args.title || "");
      if (!title.trim()) throw "A case title is required.";
      const mode = String(args.mode || "standard");
      if (mode === "confidential" && !session.unlocked) {
        throw "Unlock Confidential mode first (Legal Airlock and vault password).";
      }
      const profile = sampleProfile();
      profile.id = crypto.randomUUID();
      profile.title = title.trim();
      profile.document_kind = String(args.documentKind || "notice_of_appeal");
      profile.mode = mode === "confidential" ? "confidential" : "standard";
      profile.sealed_at = null;
      store.cases.unshift(profile);
      if (mode === "confidential") store.vaultCases.push(profile.id);
      saveStore(store);
      return profile as T;
    }
    case "get_case": {
      const id = String(args.caseId || "");
      const found = store.cases.find((c) => c.id === id);
      if (!found) throw "Case not found.";
      return found as T;
    }
    case "save_case": {
      const next = args.profile as CaseProfile;
      const idx = store.cases.findIndex((c) => c.id === next.id);
      if (idx < 0) throw "Case not found.";
      if (store.cases[idx].sealed_at) throw "This case is sealed. The Documents folder is a pointer only and cannot be edited.";
      next.updated_at = new Date().toISOString();
      store.cases[idx] = next;
      saveStore(store);
      return next as T;
    }
    case "validate_case_cmd": {
      const id = String(args.caseId || "");
      const profile = store.cases.find((c) => c.id === id);
      if (!profile) throw "Case not found.";
      const issues: ValidationReport["issues"] = [];
      if (!profile.court.name) issues.push({ field: "court.name", message: "Court name is required so the caption can be filled." });
      if (!profile.docket_number) issues.push({ field: "docket_number", message: "Docket number is required." });
      if (!profile.filer.signature_name) issues.push({ field: "filer.signature_name", message: "Signature name is required." });
      return { ok: issues.length === 0, issues } as T;
    }
    case "export_notice_of_appeal_cmd":
    case "export_pleading_cmd": {
      const id = String(args.caseId || "");
      const kind = String(args.kind || "notice_of_appeal");
      const profile = store.cases.find((c) => c.id === id);
      if (!profile) throw "Case not found.";
      if (profile.sealed_at) throw "This case is sealed. Export from the Documents pointer is refused.";
      const stem =
        kind === "complaint"
          ? `complaint-${profile.docket_number || "draft"}`
          : kind === "motion"
            ? `motion-${profile.docket_number || "draft"}`
            : `notice-of-appeal-${profile.docket_number || "draft"}`;
      const result: ExportResult = {
        docx: `Documents/JustLegal/Cases/${id}/exports/${stem}.docx`,
        pdf: `Documents/JustLegal/Cases/${id}/exports/${stem}.pdf`,
        notice:
          "This document was assembled by OpenSeason from the user's own case profile and draft text. It is a formatting aid, not legal advice. Review every line, caption, date, and citation before filing. Consider consulting a licensed attorney.",
      };
      store.artifacts[id] = store.artifacts[id] || [];
      store.artifacts[id].push(
        { kind: "export", name: `${stem}.docx`, path: result.docx },
        { kind: "export", name: `${stem}.pdf`, path: result.pdf },
      );
      saveStore(store);
      return result as T;
    }
    case "open_case_folder": {
      return sampleProfile() as T;
    }
    case "list_case_artifacts": {
      const id = String(args.caseId || "");
      return (store.artifacts[id] || []) as T;
    }
    case "add_case_evidence": {
      const id = String(args.caseId || "");
      const name = String(args.filename || "exhibit.bin");
      store.artifacts[id] = store.artifacts[id] || [];
      store.artifacts[id].push({
        kind: "evidence",
        name,
        path: `Documents/JustLegal/Cases/${id}/evidence/${name}`,
      });
      saveStore(store);
      return `Documents/JustLegal/Cases/${id}/evidence/${name}` as T;
    }
    case "get_salt":
      return "mock-salt" as T;
    case "unlock_vault": {
      const password = String(args.password || "");
      if (!password) throw "Password required.";
      session.unlocked = true;
      return true as T;
    }
    case "lock_vault":
      session.unlocked = false;
      return undefined as T;
    case "is_locked":
      return (!session.unlocked) as T;
    case "list_hunts":
      return store.cases
        .filter((c) => c.mode === "confidential")
        .map((c) => ({ id: c.id, name: c.title, created: c.created_at })) as T;
    case "import_hunt_cmd": {
      const imported = sampleProfile();
      imported.id = crypto.randomUUID();
      imported.title = "Imported confidential case";
      imported.mode = "confidential";
      store.cases.unshift(imported);
      store.vaultCases.push(imported.id);
      saveStore(store);
      return imported.id as T;
    }
    case "verify_target_cmd":
      return [
        { recipient_name: String(args.name || "Sample Agency"), award_amount: 120000, description: "Mock award (browser preview)" },
      ] as T;
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
        court_name: isCo
          ? "Colorado Court of Appeals"
          : "Superior Court of New Jersey, Appellate Division",
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
        deadline: args.triggerDate
          ? { due: "preview-only", math: "Mocked deadline", rule: String(args.rule || "") }
          : null,
        signoff: { ok: false, never_files: true, never_serves: true, items: [], issues: [] },
      } as T;
    case "sync_case_cmd":
      return {
        action: String(args.action || "backup"),
        mode: "standard",
        reason: "Account sync is not implemented in this build.",
        kind: "not_implemented",
      } as T;
    case "convert_case_to_confidential": {
      if (!session.unlocked) throw "Unlock Confidential mode first.";
      const id = String(args.caseId || "");
      const found = store.cases.find((c) => c.id === id);
      if (!found) throw "Case not found.";
      found.mode = "confidential";
      found.sealed_at = new Date().toISOString();
      store.vaultCases.push(id);
      saveStore(store);
      return found as T;
    }
    case "add_hunt_evidence_bytes": {
      const id = String(args.huntId || "");
      const found = store.cases.find((c) => c.id === id);
      if (found?.sealed_at && !String(args.password || "").length) {
        throw "This case is sealed. Re-enter the vault password before adding evidence.";
      }
      const name = String(args.filename || "exhibit.bin");
      store.artifacts[id] = store.artifacts[id] || [];
      if (store.artifacts[id].some((a) => a.kind === "evidence" && a.name === name)) {
        return "already_present" as T;
      }
      store.artifacts[id].push({
        kind: "evidence",
        name,
        path: `vaults/${id}/evidence/${name}`,
      });
      saveStore(store);
      return "added" as T;
    }
    case "add_hunt_event":
    case "delete_hunt_event":
    case "add_hunt_party":
    case "delete_hunt_party":
    case "save_complaint_section":
    case "save_disclosure_cmd":
    case "update_hunt":
    case "delete_hunt":
    case "delete_hunt_evidence":
    case "export_hunt_cmd":
    case "purge_vault_cache":
      return undefined as T;
    case "get_hunt_timeline":
    case "get_hunt_parties":
    case "get_hunt_evidence":
    case "get_complaint_sections":
      return [] as T;
    case "list_case_filings_cmd":
      return [] as T;
    case "save_case_filing_cmd":
      return args.filing as T;
    case "delete_case_filing_cmd":
      return true as T;
    case "export_case_filing_cmd":
      return { docx: "/mock/exports/filing.docx", pdf: "/mock/exports/filing.pdf" } as T;
    case "generate_certificate_of_service_cmd":
      return { ok: true, never_files: true, notice: "Mock cert", cert_pdf: "/mock/exports/cert.pdf", cert_text: "/mock/exports/cert.txt" } as T;
    case "generate_proposed_order_cmd":
      return { ok: true, never_files: true, notice: "Mock order", order_pdf: "/mock/exports/order.pdf", order_docx: "/mock/exports/order.docx" } as T;
    case "pdf_merge_cmd":
    case "pdf_split_cmd":
    case "pdf_rotate_cmd":
      return { ok: true, never_files: true, operation: cmd, output_path: "/mock/exports/output.pdf", page_count: 1, message: "Mock success" } as T;
    case "pdf_redact_cmd":
      return { ok: true, never_files: true, output_path: "/mock/exports/redacted.pdf", redactions_count: 3, message: "Mock redacted 3 items" } as T;
    case "get_signature_profile_cmd":
      return { full_name: "Mock User", initials: "MU", style: "script_formal", signature_data: null, court_s_block: "/s/ Mock User", updated_at: new Date().toISOString() } as T;
    case "save_signature_profile_cmd":
      return true as T;
    case "stamp_pdf_signature_cmd":
      return { ok: true, never_files: true, output_path: "/mock/exports/stamped.pdf", page_number: 1, stamped_text: "/s/ Mock", message: "Mock stamp applied" } as T;
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
    case "scoped_sections_cmd":
    case "scoped_fill_cmd":
      return { kind: String(args.kind || "motion"), sections: [] } as T;
    default:
      throw `Unknown command in browser preview: ${cmd}`;
  }
}

export async function ipc<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  return mockInvoke<T>(cmd, args || {});
}

export function runningInTauri(): boolean {
  return isTauri();
}
