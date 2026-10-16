<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { RecordRetrievalWizardPlan } from "$lib/types/case";
  import type { CourtRecordsRequestPacket } from "$lib/types/record";

  let {
    caseId,
    open = $bindable(false),
    onclose = () => {},
    onGenerated = () => {}
  } = $props<{
    caseId: string;
    open: boolean;
    onclose?: () => void;
    onGenerated?: () => void;
  }>();

  let loading = $state(false);
  let error = $state("");
  let plan = $state<RecordRetrievalWizardPlan | null>(null);
  let generatedPacket = $state<CourtRecordsRequestPacket | null>(null);
  let executing = $state(false);
  let copied = $state(false);

  $effect(() => {
    if (open && caseId) {
      loadPlan();
    }
  });

  async function loadPlan() {
    loading = true;
    error = "";
    generatedPacket = null;
    copied = false;
    try {
      plan = await ipc<RecordRetrievalWizardPlan>("record_retrieval_plan_cmd", { caseId });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function executeRequest() {
    executing = true;
    error = "";
    try {
      generatedPacket = await ipc<CourtRecordsRequestPacket>("record_retrieval_execute_cmd", {
        caseId,
        stateOverride: null,
      });
      onGenerated();
    } catch (e) {
      error = String(e);
    } finally {
      executing = false;
    }
  }

  function copyRequestText() {
    if (!generatedPacket) return;
    navigator.clipboard.writeText(generatedPacket.rendered_request_text);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 2000);
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-fade-in">
    <div class="relative w-full max-w-3xl rounded-xl border border-zinc-700 bg-zinc-900 text-zinc-100 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 p-5 bg-zinc-900/90">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="text-xs font-semibold px-2 py-0.5 rounded bg-blue-500/20 text-blue-300 border border-blue-500/30">
              Court Records & Rules Engine
            </span>
            <span class="text-xs font-mono text-zinc-400">
              Form {plan?.recommended_form_number || "Request"}
            </span>
          </div>
          <h2 class="text-lg font-bold text-zinc-100">Official Court Record Retrieval Wizard</h2>
        </div>
        <button
          onclick={() => { open = false; onclose(); }}
          class="rounded p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition"
          aria-label="Close modal"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 overflow-y-auto space-y-6 text-sm">
        {#if error}
          <div class="p-3.5 rounded-lg border border-red-500/30 bg-red-950/20 text-red-300 text-xs">
            {error}
          </div>
        {/if}

        {#if loading}
          <div class="py-12 text-center text-zinc-400 animate-pulse">
            Inspecting case inventory and retrieving court procedural rules...
          </div>
        {:else if plan}
          <!-- Court & Venue Info -->
          <div class="rounded-lg border border-zinc-800 bg-zinc-950/60 p-4 space-y-2">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-bold uppercase tracking-wider text-blue-400">Target Court Venue</h3>
              <span class="text-xs text-zinc-400 font-mono">14-Day Clerk Response Tracking</span>
            </div>
            <p class="font-semibold text-zinc-200">{plan.court_name}</p>
            <div class="text-xs text-zinc-400 space-y-1">
              <p><strong class="text-zinc-300">Recommended Form:</strong> {plan.recommended_form}</p>
              <p><strong class="text-zinc-300">Statutory Fee Schedule:</strong> {plan.estimated_cost_explanation}</p>
              {#if plan.fee_waiver_rule}
                <p><strong class="text-zinc-300">Fee Waiver Authority:</strong> <span class="text-amber-300">{plan.fee_waiver_rule}</span></p>
              {/if}
            </div>
          </div>

          <!-- Missing Records Checklist -->
          <div class="space-y-2">
            <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-400">
              Identified Record Gaps ({plan.missing_records.length} items to request)
            </h3>
            <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
              {#each plan.missing_records as item}
                <div class="p-3 rounded-lg border border-zinc-800 bg-zinc-950/40 flex items-start justify-between gap-3">
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-300">
                        {item.record_tier}
                      </span>
                      <p class="text-xs font-semibold text-zinc-200">{item.title}</p>
                    </div>
                    <p class="text-[11px] text-zinc-400 mt-1">{item.reason}</p>
                  </div>
                  <span class="text-[10px] text-zinc-400 shrink-0 border border-zinc-800 px-2 py-0.5 rounded">
                    {item.required_form}
                  </span>
                </div>
              {/each}
            </div>
          </div>

          <!-- Submission Channels -->
          <div class="rounded-lg border border-zinc-800 bg-zinc-950/40 p-4 space-y-2">
            <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Authorized Submission Routes</h3>
            <ul class="text-xs space-y-1 text-zinc-300">
              {#each plan.submission_channels as channel}
                <li class="flex items-center gap-2">
                  <span class="text-blue-400">✓</span>
                  <span>{channel}</span>
                </li>
              {/each}
            </ul>
          </div>

          <!-- Step-by-Step Instructions -->
          <div class="space-y-2">
            <h3 class="text-xs font-bold uppercase tracking-wider text-zinc-400">Step-by-Step Retrieval Procedure</h3>
            <div class="space-y-1.5 text-xs text-zinc-300">
              {#each plan.step_by_step_instructions as step}
                <p>{step}</p>
              {/each}
            </div>
          </div>

          <!-- Generated Request Output Display -->
          {#if generatedPacket}
            <div class="rounded-lg border border-emerald-500/30 bg-emerald-950/10 p-4 space-y-3">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-emerald-300 flex items-center gap-1.5">
                  <span>✓</span> Prefilled Form Ready for Submission
                </span>
                <button
                  onclick={copyRequestText}
                  class="text-xs px-2.5 py-1 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition"
                >
                  {copied ? "Copied to clipboard!" : "Copy request text"}
                </button>
              </div>
              <textarea
                readonly
                class="w-full h-40 font-mono text-xs p-3 rounded bg-black/60 border border-zinc-800 text-zinc-300 resize-none"
                value={generatedPacket.rendered_request_text}
              ></textarea>
              <div class="text-[11px] text-zinc-400">
                <p><strong>Instructions:</strong> {generatedPacket.instructions}</p>
                <p class="mt-1 text-emerald-300/80">Follow-up reminder registered in Case Tasks (due in 14 days).</p>
              </div>
            </div>
          {/if}
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="flex items-center justify-between border-t border-zinc-800 p-4 bg-zinc-900/90">
        <button
          onclick={() => { open = false; onclose(); }}
          class="text-xs text-zinc-400 hover:text-zinc-200 px-3 py-1.5 rounded transition"
        >
          Close
        </button>

        {#if !generatedPacket}
          <button
            onclick={executeRequest}
            disabled={executing || loading}
            class="text-xs px-4 py-2 rounded-md font-semibold bg-blue-600 hover:bg-blue-500 text-white shadow-md disabled:opacity-50 transition"
          >
            {executing ? "Generating prefilled form..." : "Generate Official Records Request"}
          </button>
        {:else}
          <button
            onclick={() => { open = false; onclose(); }}
            class="text-xs px-4 py-2 rounded-md font-semibold bg-emerald-600 hover:bg-emerald-500 text-white shadow-md transition"
          >
            Done
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
