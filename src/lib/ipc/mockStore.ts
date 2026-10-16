import { sampleProfile, type CaseArtifact, type CaseProfile, type WorkspaceCase } from "$lib/types/case";

export type Store = {
  cases: CaseProfile[];
  artifacts: Record<string, CaseArtifact[]>;
  vaultCases: string[];
  saltReady: boolean;
};

export function loadStore(): Store {
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

export function saveStore(store: Store): void {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("openseason-mock", JSON.stringify(store));
  }
}

export function workspaceOf(profile: CaseProfile, store: Store): WorkspaceCase {
  return {
    profile,
    source: store.vaultCases.includes(profile.id) && profile.mode === "confidential" ? "vault" : "documents",
    has_vault: store.vaultCases.includes(profile.id) || profile.mode === "confidential",
    sealed: !!profile.sealed_at,
    folder: `Documents/JustLegal/Cases/${profile.id}`,
  };
}
