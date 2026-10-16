import { ipc } from "$lib/ipc";
import type { CaseArtifact, CaseProfile, DecodedOrder, MailAuditReport } from "$lib/types/case";

export interface DecodeOrderResult {
  order: DecodedOrder;
  mailAudit: MailAuditReport | null;
}

export async function decodeOrderForCaseHelper(
  caseId: string,
  profile: CaseProfile | null,
  artifacts: CaseArtifact[],
  pdfPath?: string
): Promise<DecodeOrderResult | null> {
  let targetPath = pdfPath;
  if (!targetPath) {
    const orderArt = artifacts.find(
      (a) =>
        a.kind === "orders" ||
        a.name.toLowerCase().includes("order") ||
        a.name.toLowerCase().includes("dismiss") ||
        a.name.toLowerCase().includes("ruling")
    );
    if (orderArt) {
      targetPath = orderArt.path;
    }
  }
  if (!targetPath) {
    targetPath = prompt("Enter path to court order or notice PDF:") || undefined;
  }
  if (!targetPath) {
    return null;
  }

  const order = await ipc<DecodedOrder>("order_decode_cmd", { pdfPath: targetPath });

  let mailAudit: MailAuditReport | null = null;
  if (profile) {
    const stateCode = profile.court.name?.toLowerCase().includes("colorado")
      ? "CO"
      : profile.court.name?.toLowerCase().includes("new jersey")
        ? "NJ"
        : undefined;
    const entryDate = order.candidate_deadline
      ? order.candidate_deadline.slice(0, 10)
      : new Date().toISOString().slice(0, 10);
    mailAudit = await ipc<MailAuditReport>("timeline_audit_mail_cmd", {
      caseId,
      orderEntryDate: entryDate,
      mailingPostmark: null,
      deliveryReceiptDate: null,
      clerkReceiptDate: null,
      governingState: stateCode,
    }).catch(() => null);
  }

  return { order, mailAudit };
}
