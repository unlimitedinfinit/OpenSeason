<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseProfile } from "$lib/types/case";
  import {
    type RecordInventoryReport,
    type RecordEntry,
    type MissingRecordItem,
    type EvidentiaryTier,
    type CourtRecordsRequestPacket,
    TIER_LABELS,
  } from "$lib/types/record";

  interface Props {
    caseId: string;
    profile?: CaseProfile | null;
    onRequestFormReady?: (packet: CourtRecordsRequestPacket) => void;
  }

  let { caseId, profile = null, onRequestFormReady }: Props = $props();

  let loading = $state(false);
  let report = $state<RecordInventoryReport | null>(null);
  let selectedTier = $state<EvidentiaryTier | "all">("all");
  let showDocketInput = $state(false);
  let docketText = $state("");
  let requestPacket = $state<CourtRecordsRequestPacket | null>(null);
  let copiedSha = $state<string | null>(null);
  let errorMessage = $state("");

  async function loadInventory() {
    if (!caseId) return;
    loading = true;
    errorMessage = "";
    try {
      report = await ipc<RecordInventoryReport>("record_inventory_cmd", { caseId });
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to scan case records.";
    } finally {
      loading = false;
    }
  }

  async function reconcileDocket() {
    if (!caseId || !docketText.trim()) return;
    loading = true;
    errorMessage = "";
    try {
      report = await ipc<RecordInventoryReport>("record_docket_match_cmd", {
        caseId,
        docketText: docketText.trim(),
      });
      showDocketInput = false;
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to reconcile docket.";
    } finally {
      loading = false;
    }
  }

  async function generateRequestForm() {
    if (!caseId) return;
    loading = true;
    errorMessage = "";
    try {
      const res = await ipc<CourtRecordsRequestPacket>("record_request_form_cmd", {
        caseId,
        state: profile?.governing_state || "NJ",
      });
      requestPacket = res;
      onRequestFormReady?.(res);
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to generate request form.";
    } finally {
      loading = false;
    }
  }

  function copyToClipboard(text: string, id: string) {
    if (navigator?.clipboard) {
      navigator.clipboard.writeText(text);
      copiedSha = id;
      setTimeout(() => {
        if (copiedSha === id) copiedSha = null;
      }, 2000);
    }
  }

  let filteredEntries = $derived(
    report?.entries.filter((e) => selectedTier === "all" || e.tier === selectedTier) || []
  );

  onMount(() => {
    loadInventory();
  });
</script>

<div class="rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-sm overflow-hidden">
  <!-- Header Banner -->
  <div class="p-4 sm:p-5 border-b border-stone-100 dark:border-stone-800 flex flex-wrap items-center justify-between gap-3 bg-stone-50/50 dark:bg-stone-950/40">
    <div>
      <div class="flex items-center gap-2">
        <span class="inline-flex items-center justify-center w-6 h-6 rounded-md bg-teal-500/10 text-teal-600 dark:text-teal-400 font-bold text-xs">
          CR
        </span>
        <h3 class="text-sm font-semibold text-stone-900 dark:text-stone-100">
          Court Record Reconstruction & Evidentiary Tiers
        </h3>
      </div>
      <p class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
        Strict separation of official court filings, transcripts, personal archive, and unsubmitted materials.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button
        onclick={() => (showDocketInput = !showDocketInput)}
        class="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 hover:bg-stone-50 text-stone-700 dark:text-stone-200 transition"
      >
        {showDocketInput ? "Hide Docket Tool" : "Reconcile Docket"}
      </button>
      <button
        onclick={loadInventory}
        disabled={loading}
        class="px-2.5 py-1.5 text-xs font-medium rounded-lg bg-teal-600 hover:bg-teal-700 text-white transition disabled:opacity-50"
      >
        {loading ? "Scanning..." : "Rescan Workspace"}
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="m-4 p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 text-xs text-rose-700 dark:text-rose-400">
      {errorMessage}
    </div>
  {/if}

  <!-- Docket Reconciliation Drawer -->
  {#if showDocketInput}
    <div class="p-4 bg-teal-50/50 dark:bg-teal-950/20 border-b border-teal-100 dark:border-teal-900/50 space-y-2">
      <label for="docket-textarea" class="text-xs font-semibold text-teal-900 dark:text-teal-200">
        Paste Court Docket Sheet / Register of Actions (Plain Text)
      </label>
      <p class="text-[11px] text-teal-700/80 dark:text-teal-300/80">
        Reconstructs what the clerk actually entered, identifies missing documents, and matches files by date and title.
      </p>
      <textarea
        id="docket-textarea"
        bind:value={docketText}
        rows="4"
        placeholder="02/15/2026 ORDER GRANTING MOTION TO DISMISS&#10;01/14/2026 BRIEF IN OPPOSITION FILED BY PLAINTIFF&#10;01/05/2026 MOTION TO DISMISS..."
        class="w-full text-xs font-mono p-2.5 rounded-lg border border-teal-200 dark:border-teal-800 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
      ></textarea>
      <div class="flex justify-end gap-2 pt-1">
        <button
          onclick={() => (showDocketInput = false)}
          class="px-3 py-1 text-xs rounded-md border border-stone-300 dark:border-stone-700 text-stone-600 dark:text-stone-300 hover:bg-stone-100"
        >
          Cancel
        </button>
        <button
          onclick={reconcileDocket}
          disabled={!docketText.trim() || loading}
          class="px-3 py-1 text-xs rounded-md bg-teal-600 hover:bg-teal-700 text-white font-medium disabled:opacity-50"
        >
          Match Against Workspace Files
        </button>
      </div>
    </div>
  {/if}

  <!-- 4-Tier Metrics Grid -->
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 p-4 border-b border-stone-100 dark:border-stone-800 bg-stone-50/30 dark:bg-stone-900/50">
    <button
      onclick={() => (selectedTier = selectedTier === "official_court_record" ? "all" : "official_court_record")}
      class="text-left p-2.5 rounded-lg border transition {selectedTier === 'official_court_record' ? 'border-emerald-500 bg-emerald-50/50 dark:bg-emerald-950/20' : 'border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 hover:border-emerald-300'}"
    >
      <div class="text-[10px] font-semibold uppercase tracking-wider text-emerald-600 dark:text-emerald-400">Tier 1: Official</div>
      <div class="text-lg font-bold text-stone-900 dark:text-stone-100 mt-0.5">{report?.tier1_official_count ?? 0}</div>
      <div class="text-[10px] text-stone-400">Clerk Stamped & Filed</div>
    </button>

    <button
      onclick={() => (selectedTier = selectedTier === "hearing_and_evidentiary_record" ? "all" : "hearing_and_evidentiary_record")}
      class="text-left p-2.5 rounded-lg border transition {selectedTier === 'hearing_and_evidentiary_record' ? 'border-blue-500 bg-blue-50/50 dark:bg-blue-950/20' : 'border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 hover:border-blue-300'}"
    >
      <div class="text-[10px] font-semibold uppercase tracking-wider text-blue-600 dark:text-blue-400">Tier 2: Transcripts</div>
      <div class="text-lg font-bold text-stone-900 dark:text-stone-100 mt-0.5">{report?.tier2_hearing_count ?? 0}</div>
      <div class="text-[10px] text-stone-400">Certified Oral Records</div>
    </button>

    <button
      onclick={() => (selectedTier = selectedTier === "personal_case_archive" ? "all" : "personal_case_archive")}
      class="text-left p-2.5 rounded-lg border transition {selectedTier === 'personal_case_archive' ? 'border-amber-500 bg-amber-50/50 dark:bg-amber-950/20' : 'border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 hover:border-amber-300'}"
    >
      <div class="text-[10px] font-semibold uppercase tracking-wider text-amber-600 dark:text-amber-400">Tier 3: Archive</div>
      <div class="text-lg font-bold text-stone-900 dark:text-stone-100 mt-0.5">{report?.tier3_archive_count ?? 0}</div>
      <div class="text-[10px] text-stone-400">Internal Drafts & Work</div>
    </button>

    <button
      onclick={() => (selectedTier = selectedTier === "potential_additional_evidence" ? "all" : "potential_additional_evidence")}
      class="text-left p-2.5 rounded-lg border transition {selectedTier === 'potential_additional_evidence' ? 'border-rose-500 bg-rose-50/50 dark:bg-rose-950/20' : 'border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 hover:border-rose-300'}"
    >
      <div class="text-[10px] font-semibold uppercase tracking-wider text-rose-600 dark:text-rose-400">Tier 4: Unsubmitted</div>
      <div class="text-lg font-bold text-stone-900 dark:text-stone-100 mt-0.5">{report?.tier4_unsubmitted_count ?? 0}</div>
      <div class="text-[10px] text-stone-400">Extrinsic (Needs Motion)</div>
    </button>
  </div>

  <!-- Missing Records & Record Requests Section -->
  {#if report?.missing_records && report.missing_records.length > 0}
    <div class="p-4 bg-amber-500/5 dark:bg-amber-950/20 border-b border-amber-500/20 space-y-3">
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-amber-500 animate-pulse"></span>
          <h4 class="text-xs font-bold text-amber-800 dark:text-amber-300 uppercase tracking-wider">
            Critical Missing Records Identified ({report.missing_records.length})
          </h4>
        </div>
        <button
          onclick={generateRequestForm}
          disabled={loading}
          class="text-xs font-semibold px-2.5 py-1 rounded bg-amber-600 hover:bg-amber-700 text-white transition"
        >
          Generate Records Request Form (CN 10200)
        </button>
      </div>

      <div class="grid gap-2">
        {#each report.missing_records as item}
          <div class="p-2.5 rounded-lg border border-amber-200 dark:border-amber-800/60 bg-white dark:bg-stone-900 flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-xs">
            <div>
              <div class="font-semibold text-stone-900 dark:text-stone-100 flex items-center gap-2">
                <span>{item.title}</span>
                {#if item.docket_entry_ref}
                  <span class="px-1.5 py-0.5 rounded text-[10px] bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-300">
                    {item.docket_entry_ref}
                  </span>
                {/if}
              </div>
              <p class="text-[11px] text-stone-500 dark:text-stone-400 mt-0.5">{item.reason}</p>
              <p class="text-[10px] text-amber-700 dark:text-amber-400 mt-0.5 font-medium">{item.recommended_action} · {item.estimated_cost_notice}</p>
            </div>
            {#if item.request_form}
              <span class="shrink-0 px-2 py-0.5 rounded text-[10px] font-semibold bg-amber-100 dark:bg-amber-900/40 text-amber-800 dark:text-amber-300">
                {item.request_form}
              </span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Record Inventory Table -->
  <div class="p-4 space-y-3">
    <div class="flex items-center justify-between">
      <div class="text-xs font-semibold text-stone-700 dark:text-stone-300">
        Scanned Documents ({filteredEntries.length})
      </div>
      {#if selectedTier !== "all"}
        <button
          onclick={() => (selectedTier = "all")}
          class="text-xs text-teal-600 dark:text-teal-400 hover:underline"
        >
          Show All Tiers
        </button>
      {/if}
    </div>

    {#if filteredEntries.length === 0}
      <div class="text-center py-8 text-xs text-stone-400">
        No documents found in this tier. Add files to <code class="font-mono bg-stone-100 dark:bg-stone-800 px-1 py-0.5 rounded">orders/</code>, <code class="font-mono bg-stone-100 dark:bg-stone-800 px-1 py-0.5 rounded">evidence/</code>, or <code class="font-mono bg-stone-100 dark:bg-stone-800 px-1 py-0.5 rounded">drafts/</code>.
      </div>
    {:else}
      <div class="divide-y divide-stone-100 dark:divide-stone-800 border border-stone-200 dark:border-stone-800 rounded-lg overflow-hidden">
        {#each filteredEntries as entry}
          {@const meta = TIER_LABELS[entry.tier]}
          <div class="p-3 bg-white dark:bg-stone-900 hover:bg-stone-50/50 dark:hover:bg-stone-800/40 transition flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs">
            <div class="space-y-1 min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-semibold text-stone-900 dark:text-stone-100 truncate">{entry.title || entry.filename}</span>
                <span class="px-2 py-0.5 rounded-full text-[10px] font-medium border {meta.color}">
                  {meta.badge}
                </span>
                {#if entry.filing_stamp_detected}
                  <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
                    CLERK STAMP DETECTED
                  </span>
                {/if}
              </div>
              <div class="text-[11px] text-stone-500 font-mono truncate">{entry.relative_path}</div>
              <div class="text-[10px] text-stone-400 flex flex-wrap items-center gap-2">
                <span>Provenance: {entry.source_provenance}</span>
                {#if entry.docket_date}<span>· Date: {entry.docket_date}</span>{/if}
                <span>· {(entry.size_bytes / 1024).toFixed(1)} KB</span>
              </div>
            </div>

            <!-- SHA-256 Provenance Copy -->
            <div class="shrink-0 flex items-center gap-2">
              <button
                onclick={() => copyToClipboard(entry.sha256, entry.id)}
                title="Copy SHA-256 integrity hash for court citation"
                class="px-2 py-1 rounded bg-stone-100 dark:bg-stone-800 hover:bg-stone-200 dark:hover:bg-stone-700 text-[10px] font-mono text-stone-600 dark:text-stone-300 transition"
              >
                {copiedSha === entry.id ? "Copied Hash!" : `SHA: ${entry.sha256.slice(0, 8)}...`}
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Generated Request Packet Modal/Card -->
  {#if requestPacket}
    <div class="p-4 m-4 rounded-xl border border-teal-300 dark:border-teal-800 bg-teal-50/50 dark:bg-teal-950/30 space-y-2">
      <div class="flex items-center justify-between">
        <h4 class="text-xs font-bold text-teal-900 dark:text-teal-200">
          Prefilled Court Records Request (Form {requestPacket.state === 'NJ' ? 'CN 10200' : 'Records Access'})
        </h4>
        <button
          onclick={() => (requestPacket = null)}
          class="text-xs text-stone-400 hover:text-stone-600"
        >
          Dismiss
        </button>
      </div>
      <p class="text-[11px] text-stone-600 dark:text-stone-300">{requestPacket.instructions} · {requestPacket.statutory_fee_notice}</p>
      <pre class="p-3 rounded bg-white dark:bg-stone-900 border border-teal-200 dark:border-teal-800 text-[11px] font-mono text-stone-800 dark:text-stone-200 overflow-x-auto max-h-48 whitespace-pre-wrap">{requestPacket.rendered_request_text}</pre>
      <div class="flex justify-end gap-2 pt-1">
        <button
          onclick={() => copyToClipboard(requestPacket?.rendered_request_text || "", "packet")}
          class="px-3 py-1 text-xs rounded bg-teal-600 hover:bg-teal-700 text-white font-medium transition"
        >
          {copiedSha === "packet" ? "Copied Packet!" : "Copy Full Request Text"}
        </button>
      </div>
    </div>
  {/if}
</div>
