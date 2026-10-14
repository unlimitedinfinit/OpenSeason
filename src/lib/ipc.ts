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
