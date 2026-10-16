import { ipc } from "$lib/ipc";
import type { CaseArtifact, CaseProfile } from "$lib/types/case";

export interface AddEvidenceResult {
  status?: string;
  error?: string;
  artifacts?: CaseArtifact[];
}

export function promptAndUploadEvidence(
  caseId: string,
  profile: CaseProfile | null,
  onComplete: (result: AddEvidenceResult) => void
) {
  const input = document.createElement("input");
  input.type = "file";
  input.onchange = async (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file || !profile) return;
    const desc = prompt("Short description for this exhibit:", file.name);
    if (desc === null) return;
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      let status = "";
      if (profile.mode === "confidential") {
        const password = prompt("If this case is sealed, re-enter the vault password. Leave empty only for an unsealed Confidential case.");
        const result = await ipc<string>("add_hunt_evidence_bytes", {
          huntId: caseId,
          filename: file.name,
          fileBytes: bytes,
          description: desc,
          password: password && password.length ? password : null,
        });
        status =
          result === "already_present"
            ? `This exhibit is already in the vault (same file contents). The existing sealed copy was not changed: ${file.name}`
            : `Encrypted exhibit stored in the vault: ${file.name}`;
      } else {
        await ipc("add_case_evidence", {
          caseId,
          filename: file.name,
          fileBytes: bytes,
          description: desc,
        });
        status = `Copied exhibit into the case evidence folder: ${file.name}`;
      }
      const artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => []);
      onComplete({ status, artifacts });
    } catch (err) {
      onComplete({ error: String(err) });
    }
  };
  input.click();
}
