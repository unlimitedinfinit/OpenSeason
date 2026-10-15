<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { PreFlightAudit } from "$lib/types/case";

  interface Props {
    caseId: string;
    onrefresh?: () => void;
  }

  let { caseId, onrefresh }: Props = $props();

  let audit = $state<PreFlightAudit | null>(null);
  let loading = $state(true);
  let error = $state("");

  onMount(loadAudit);

  export async function loadAudit() {
    loading = true;
    error = "";
    try {
      audit = await ipc<PreFlightAudit>("preflight_audit_cmd", { caseId });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="rounded-xl border {audit?.ok ? 'border-emerald-500/40 bg-emerald-950/15' : 'border-amber-500/40 bg-amber-950/15'} p-5 space-y-4 shadow-sm transition-all">
  <!-- Header -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-zinc-800/80 pb-3">
    <div class="flex items-center gap-3">
      <div class="h-8 w-8 rounded-lg {audit?.ok ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20' : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'} flex items-center justify-center">
        {#if audit?.ok}
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        {:else}
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        {/if}
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="font-semibold text-sm text-zinc-100">Court Readiness Pre-Flight Audit</h3>
          <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full {audit?.ok ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'}">
            {audit?.ok ? 'Ready to Export' : 'Incomplete Requirements'}
          </span>
        </div>
        <p class="text-xs text-zinc-400">Deterministic check for required court parts, deadline math, and fact/transcript grounding.</p>
      </div>
    </div>
    <button
      onclick={loadAudit}
      disabled={loading}
      class="text-xs px-3 py-1.5 rounded-md border border-zinc-700 hover:bg-zinc-800 text-zinc-300 flex items-center gap-1.5 transition-colors self-end sm:self-auto"
    >
      <svg class="h-3.5 w-3.5 {loading ? 'animate-spin' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      Refresh Audit
    </button>
  </div>

  {#if loading && !audit}
    <div class="text-xs text-zinc-500 py-3 text-center">Evaluating case components and record grounding...</div>
  {:else if error}
    <div class="text-xs text-red-400 py-2">Error running pre-flight check: {error}</div>
  {:else if audit}
    <!-- Checklist Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
      <!-- Caption -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.caption_ok ? 'text-emerald-400' : 'text-red-400'}">
          {audit.caption_ok ? '✓' : '✗'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Court & Caption</p>
          <p class="text-zinc-400 text-[11px]">{audit.caption_summary}</p>
        </div>
      </div>

      <!-- Docket -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.docket_ok ? 'text-emerald-400' : 'text-red-400'}">
          {audit.docket_ok ? '✓' : '✗'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Case / Docket Number</p>
          <p class="text-zinc-400 text-[11px]">{audit.docket_summary}</p>
        </div>
      </div>

      <!-- Filer Signature -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.filer_ok ? 'text-emerald-400' : 'text-red-400'}">
          {audit.filer_ok ? '✓' : '✗'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Filer Signature Block</p>
          <p class="text-zinc-400 text-[11px]">{audit.filer_summary}</p>
        </div>
      </div>

      <!-- Deadline Math -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.deadline_ok ? 'text-emerald-400' : 'text-amber-400'}">
          {audit.deadline_ok ? '✓' : '⚠'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Filing Deadline / Timeliness</p>
          <p class="text-zinc-400 text-[11px]">{audit.deadline_summary}</p>
        </div>
      </div>

      <!-- Facts Verification -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.facts_ok ? 'text-emerald-400' : 'text-amber-400'}">
          {audit.facts_ok ? '✓' : '⚠'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Fact Record Grounding</p>
          <p class="text-zinc-400 text-[11px]">
            {audit.facts_total_count === 0 ? 'No facts registered yet' : `${audit.facts_verified_count} of ${audit.facts_total_count} assertions cite verified exhibits`}
          </p>
        </div>
      </div>

      <!-- Authorities Check -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.authorities_ok ? 'text-emerald-400' : 'text-amber-400'}">
          {audit.authorities_ok ? '✓' : '⚠'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Legal Authorities Check</p>
          <p class="text-zinc-400 text-[11px]">
            {audit.authorities_total_count === 0 ? 'No citations in draft' : `${audit.authorities_verified_count} of ${audit.authorities_total_count} citations verified`}
          </p>
        </div>
      </div>

      <!-- Transcripts -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.transcripts_ok ? 'text-emerald-400' : 'text-amber-400'}">
          {audit.transcripts_ok ? '✓' : '⚠'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Hearing Record / Transcript Grounding</p>
          <p class="text-zinc-400 text-[11px]">{audit.transcripts_ok ? 'Transcripts indexed; citations grounded' : 'Transcript citations ungrounded or files missing'}</p>
        </div>
      </div>

      <!-- 5 Required Motion Parts -->
      <div class="flex items-start gap-2.5 p-2.5 rounded-lg bg-zinc-900/60 border border-zinc-800">
        <span class="mt-0.5 {audit.packet_parts_ready ? 'text-emerald-400' : 'text-zinc-400'}">
          {audit.packet_parts_ready ? '✓' : '○'}
        </span>
        <div>
          <p class="font-medium text-zinc-200">Five Required Parts</p>
          <p class="text-zinc-400 text-[11px]">{audit.packet_parts_ready ? 'Packet exports generated' : 'Notice, Motion, Cert, Order, Service ready to compile'}</p>
        </div>
      </div>
    </div>

    <!-- Missing Items Summary Callout -->
    {#if !audit.ok && audit.missing_requirements.length > 0}
      <div class="rounded-lg bg-amber-500/10 border border-amber-500/20 p-3 space-y-1.5 text-xs text-amber-200">
        <p class="font-medium flex items-center gap-1.5 text-amber-300">
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          Before filing, resolve the following items:
        </p>
        <ul class="list-disc list-inside space-y-0.5 text-zinc-300 pl-1">
          {#each audit.missing_requirements as req}
            <li>{req}</li>
          {/each}
        </ul>
      </div>
    {/if}
  {/if}
</div>
