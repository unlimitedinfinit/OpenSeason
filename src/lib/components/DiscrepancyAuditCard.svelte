<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseProfile } from "$lib/types/case";
  import type { DiscrepancyReport, IssueItem, FactualPacketReport } from "$lib/types/record";

  interface Props {
    caseId: string;
    profile?: CaseProfile | null;
    onExploreRoutes?: (issueId?: string) => void;
    onPacketGenerated?: (packet: FactualPacketReport) => void;
  }

  let { caseId, profile = null, onExploreRoutes, onPacketGenerated }: Props = $props();

  let loading = $state(false);
  let report = $state<DiscrepancyReport | null>(null);
  let generatingPacketId = $state<string | null>(null);
  let packetResult = $state<FactualPacketReport | null>(null);
  let errorMessage = $state("");

  async function runAudit() {
    if (!caseId) return;
    loading = true;
    errorMessage = "";
    try {
      report = await ipc<DiscrepancyReport>("accountability_audit_cmd", { caseId });
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to audit judicial findings.";
    } finally {
      loading = false;
    }
  }

  async function generatePacket(issueId: string) {
    if (!caseId) return;
    generatingPacketId = issueId;
    errorMessage = "";
    try {
      const packet = await ipc<FactualPacketReport>("accountability_packet_cmd", {
        caseId,
        issueId,
      });
      packetResult = packet;
      onPacketGenerated?.(packet);
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to assemble factual packet.";
    } finally {
      generatingPacketId = null;
    }
  }

  function getStatusMeta(status: string) {
    switch (status) {
      case "contradicted_by_record":
        return {
          label: "Contradicted by Record Evidence",
          badge: "bg-rose-500/10 text-rose-700 dark:text-rose-400 border-rose-500/30",
        };
      case "unverified_oral_colloquy":
        return {
          label: "Unverified Oral Colloquy (Transcript Required)",
          badge: "bg-blue-500/10 text-blue-700 dark:text-blue-400 border-blue-500/30",
        };
      default:
        return {
          label: "Unsupported in Record (Missing Citation)",
          badge: "bg-amber-500/10 text-amber-700 dark:text-amber-400 border-amber-500/30",
        };
    }
  }

  onMount(() => {
    runAudit();
  });
</script>

<div class="rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm overflow-hidden">
  <!-- Card Header -->
  <div class="p-4 sm:p-5 border-b border-stone-100 dark:border-stone-800 flex flex-wrap items-center justify-between gap-3 bg-stone-50/50 dark:bg-stone-950/40">
    <div>
      <div class="flex items-center gap-2">
        <span class="inline-flex items-center justify-center w-6 h-6 rounded-md bg-amber-500/10 text-amber-600 dark:text-amber-400 font-bold text-xs">
          DA
        </span>
        <h3 class="text-sm font-semibold text-stone-900 dark:text-stone-100">
          Judicial Finding vs. Record Discrepancy Audit
        </h3>
      </div>
      <p class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
        Cross-checks operative order clauses against verified docket filings to flag unsupported findings.
      </p>
    </div>
    <button
      onclick={runAudit}
      disabled={loading}
      class="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 hover:bg-stone-50 text-stone-700 dark:text-stone-200 transition disabled:opacity-50"
    >
      {loading ? "Auditing..." : "Re-Audit Orders"}
    </button>
  </div>

  {#if errorMessage}
    <div class="m-4 p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 text-xs text-rose-700 dark:text-rose-400">
      {errorMessage}
    </div>
  {/if}

  <!-- Mandatory Non-Tolling Alert Banner -->
  <div class="px-4 py-3 bg-rose-500/5 dark:bg-rose-950/20 border-b border-rose-500/20 text-xs flex items-start gap-2.5">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
    <div class="space-y-0.5">
      <span class="font-bold text-rose-900 dark:text-rose-300">Mandatory Procedural Invariant:</span>
      <p class="text-[11px] text-rose-800/90 dark:text-rose-300/80 leading-relaxed">
        {report?.non_tolling_warning || "Disciplinary grievances (ACJC, CJD, OAE) and public record requests DO NOT toll court appeal deadlines. Factual errors in orders must be addressed through timely reconsideration or notice of appeal."}
      </p>
    </div>
  </div>

  <!-- Issues List -->
  <div class="p-4 space-y-3">
    {#if !report || report.issues.length === 0}
      <div class="text-center py-6 text-xs text-stone-400">
        No active discrepancies logged. Scan case orders or import docket filings to verify findings.
      </div>
    {:else}
      <div class="space-y-3">
        {#each report.issues as issue}
          {@const meta = getStatusMeta(issue.record_support_status)}
          <div class="p-3.5 rounded-lg border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 space-y-3 text-xs">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <span class="font-semibold text-stone-900 dark:text-stone-100">{issue.title}</span>
              <span class="px-2 py-0.5 rounded-full text-[10px] font-medium border {meta.badge}">
                {meta.label}
              </span>
            </div>

            <!-- Finding Source & Quoted Text -->
            <div class="p-2.5 rounded bg-stone-50 dark:bg-stone-950/50 border border-stone-200 dark:border-stone-800/80 space-y-1">
              <div class="text-[10px] uppercase font-bold text-stone-400">
                Operative Finding ({issue.finding_source.document_name}{#if issue.finding_source.page}, p. {issue.finding_source.page}{/if}):
              </div>
              <p class="italic text-stone-800 dark:text-stone-200 font-serif text-[11px]">
                "{issue.finding_source.quoted_text}"
              </p>
            </div>

            <!-- Contradicting Record Evidence -->
            {#if issue.contradicting_evidence_filed.length > 0 || issue.contradicting_evidence_local.length > 0}
              <div class="space-y-1">
                <div class="text-[10px] uppercase font-bold text-teal-600 dark:text-teal-400">Verified Evidence in Record:</div>
                <ul class="list-disc list-inside space-y-0.5 text-[11px] text-stone-600 dark:text-stone-300">
                  {#each issue.contradicting_evidence_filed as evidence}
                    <li>{evidence}</li>
                  {/each}
                  {#each issue.contradicting_evidence_local as local}
                    <li class="font-mono text-[10px] text-stone-400">{local}</li>
                  {/each}
                </ul>
              </div>
            {/if}

            <!-- Action Controls -->
            <div class="flex flex-wrap items-center justify-between gap-2 pt-1 border-t border-stone-100 dark:border-stone-800">
              <div class="flex flex-wrap gap-1.5">
                {#each issue.available_routes as route}
                  <span class="px-2 py-0.5 rounded text-[10px] bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
                    {route.title}
                  </span>
                {/each}
              </div>
              <div class="flex items-center gap-2">
                <button
                  onclick={() => generatePacket(issue.id)}
                  disabled={generatingPacketId === issue.id}
                  class="px-2.5 py-1 text-xs rounded border border-stone-300 dark:border-stone-700 hover:bg-stone-50 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-200 font-medium transition disabled:opacity-50"
                >
                  {generatingPacketId === issue.id ? "Assembling..." : "Generate Factual Packet"}
                </button>
                <button
                  onclick={() => onExploreRoutes?.(issue.id)}
                  class="px-2.5 py-1 text-xs rounded bg-teal-600 hover:bg-teal-700 text-white font-medium transition"
                >
                  Procedural Routes &rarr;
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Factual Discrepancy Packet Preview Modal/Snippet -->
  {#if packetResult}
    <div class="p-4 m-4 rounded-xl border border-teal-300 dark:border-teal-800 bg-teal-50/50 dark:bg-teal-950/30 space-y-2">
      <div class="flex items-center justify-between">
        <h4 class="text-xs font-bold text-teal-900 dark:text-teal-200">
          Objective Factual Discrepancy Packet Generated
        </h4>
        <button
          onclick={() => (packetResult = null)}
          class="text-xs text-stone-400 hover:text-stone-600"
        >
          Dismiss
        </button>
      </div>
      <p class="text-[11px] text-stone-600 dark:text-stone-300">{packetResult.notice}</p>
      <pre class="p-3 rounded bg-white dark:bg-stone-900 border border-teal-200 dark:border-teal-800 text-[11px] font-mono text-stone-800 dark:text-stone-200 overflow-x-auto max-h-48 whitespace-pre-wrap">{packetResult.rendered_markdown}</pre>
    </div>
  {/if}
</div>
