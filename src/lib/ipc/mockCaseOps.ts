import { sampleProfile, type CaseProfile, type ExportResult, type ValidationReport } from "$lib/types/case";
import { session } from "$lib/session.svelte";
import { loadStore, saveStore, workspaceOf, type Store } from "./mockStore";

export function handleCaseOps<T>(cmd: string, args: Record<string, unknown>, store: Store): T | null {
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
    case "open_case_folder":
      return sampleProfile() as T;
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
    default:
      return null;
  }
}
