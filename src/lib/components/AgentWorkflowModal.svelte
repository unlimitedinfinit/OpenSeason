<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { WorkflowPlan, WorkflowStepResult } from "$lib/types/workflow";

  interface Props {
    caseId: string;
    onClose: () => void;
  }

  let { caseId, onClose }: Props = $props();

  let activePlan = $state<WorkflowPlan | null>(null);
  let objectiveInput = $state("");
  let loading = $state(false);
  let advancing = $state(false);
  let error = $state("");
  let lastResult = $state<WorkflowStepResult | null>(null);

  const PRESET_OBJECTIVES = [
    { label: "File Notice of Appeal", text: "File Notice of Appeal with jurisdictional mail audit" },
    { label: "Reconstruct Missing Court Records", text: "Retrieve missing hearing transcripts and court orders" },
    { label: "Audit Order & Challenge Findings", text: "Audit court order findings against evidentiary record" },
  ];

  onMount(() => {
    loadStatus();
  });

  async function loadStatus() {
    loading = true;
    error = "";
    try {
      activePlan = await ipc<WorkflowPlan | null>("workflow_status_cmd", { caseId });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handlePlanWorkflow(obj?: string) {
    const objective = obj || objectiveInput.trim();
    if (!objective) return;
    loading = true;
    error = "";
    try {
      activePlan = await ipc<WorkflowPlan>("workflow_plan_cmd", { caseId, objective });
      lastResult = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleAdvance(approve = false) {
    if (!activePlan) return;
    advancing = true;
    error = "";
    try {
      const res = await ipc<WorkflowStepResult>("workflow_advance_cmd", { caseId, approve });
      lastResult = res;
      await loadStatus();
    } catch (e) {
      error = String(e);
    } finally {
      advancing = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 overflow-y-auto animate-fade-in">
  <div class="bg-card rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] flex flex-col border border-border">
    <div class="px-6 py-4 border-b border-border flex items-center justify-between bg-secondary/30">
      <div>
        <div class="flex items-center gap-2">
          <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-primary/10 text-primary border border-primary/20">
            Agent Workflow Coordinator
          </span>
          {#if activePlan?.is_complete}
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
              Completed
            </span>
          {/if}
        </div>
        <h2 class="text-xl font-bold text-stone-900 dark:text-stone-100 mt-1">Autonomous Procedural Steps</h2>
      </div>
      <button onclick={onClose} class="text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 text-2xl font-light leading-none">
        &times;
      </button>
    </div>

    <div class="p-6 overflow-y-auto flex-1 space-y-6">
      {#if error}
        <div class="p-3 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-sm text-rose-700 dark:text-rose-300">
          {error}
        </div>
      {/if}

      {#if !activePlan}
        <div class="space-y-4">
          <p class="text-sm text-stone-600 dark:text-stone-400">
            State your legal objective in plain language. The agent coordinator will compose a deterministic, multi-step execution plan with required human approval checkpoints and immutable receipt tracking.
          </p>

          <div class="space-y-2">
            <label for="obj-input" class="text-xs font-semibold text-stone-700 dark:text-stone-300 uppercase tracking-wider">Objective</label>
            <div class="flex gap-2">
              <input
                id="obj-input"
                type="text"
                bind:value={objectiveInput}
                placeholder="e.g. Prepare and file Notice of Appeal for recent dismissal order"
                class="flex-1 px-3 py-2 text-sm rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 text-stone-900 dark:text-stone-100 focus:outline-none focus:ring-2 focus:ring-primary"
              />
              <button
                onclick={() => handlePlanWorkflow()}
                disabled={loading || !objectiveInput.trim()}
                class="px-4 py-2 bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground text-sm font-medium rounded-lg transition-colors"
              >
                Formulate Plan
              </button>
            </div>
          </div>

          <div>
            <div class="text-xs font-medium text-stone-500 dark:text-stone-400 mb-2">Preset Objectives:</div>
            <div class="flex flex-wrap gap-2">
              {#each PRESET_OBJECTIVES as preset}
                <button
                  type="button"
                  onclick={() => handlePlanWorkflow(preset.text)}
                  disabled={loading}
                  class="text-xs px-3 py-1.5 rounded-full border border-stone-300 dark:border-stone-700 bg-stone-50 dark:bg-stone-800 hover:bg-stone-100 dark:hover:bg-stone-700 text-stone-700 dark:text-stone-300 transition-colors"
                >
                  {preset.label}
                </button>
              {/each}
            </div>
          </div>
        </div>
      {:else}
        <div class="space-y-4">
          <div class="flex items-center justify-between bg-stone-50 dark:bg-stone-800/60 p-4 rounded-xl border border-stone-200 dark:border-stone-700">
            <div>
              <div class="text-xs uppercase tracking-wider text-stone-500 dark:text-stone-400 font-semibold">Active Plan</div>
              <div class="text-base font-bold text-stone-900 dark:text-stone-100">{activePlan.objective}</div>
              <div class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                Step {Math.min(activePlan.current_step_index + 1, activePlan.steps.length)} of {activePlan.steps.length}
              </div>
            </div>
            <button
              onclick={() => handlePlanWorkflow()}
              class="text-xs text-stone-500 hover:text-stone-700 dark:hover:text-stone-300 underline"
            >
              Reset Plan
            </button>
          </div>

          {#if lastResult}
            <div class="p-4 rounded-xl border {lastResult.requires_approval ? 'bg-amber-50 dark:bg-amber-950/30 border-amber-300 dark:border-amber-700' : 'bg-emerald-50 dark:bg-emerald-950/30 border-emerald-300 dark:border-emerald-700'}">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold uppercase tracking-wider {lastResult.requires_approval ? 'text-amber-800 dark:text-amber-300' : 'text-emerald-800 dark:text-emerald-300'}">
                  {lastResult.requires_approval ? 'Checkpoint: Human Approval Required' : 'Step Executed'}
                </span>
                {#if lastResult.receipt_id}
                  <span class="text-xs font-mono text-stone-500 dark:text-stone-400">{lastResult.receipt_id}</span>
                {/if}
              </div>
              <p class="text-sm text-stone-800 dark:text-stone-200 mt-1">{lastResult.output_summary}</p>
              {#if lastResult.approval_prompt}
                <p class="text-sm font-semibold text-amber-900 dark:text-amber-200 mt-2">{lastResult.approval_prompt}</p>
              {/if}
            </div>
          {/if}

          <div class="space-y-3">
            <h3 class="text-xs font-bold uppercase tracking-wider text-stone-500 dark:text-stone-400">Step Sequence</h3>
            <div class="space-y-2">
              {#each activePlan.steps as step, idx}
                <div class="p-3 rounded-lg border {idx === activePlan.current_step_index && !activePlan.is_complete ? 'border-primary/60 bg-primary/5' : 'border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-800/30'} flex items-start gap-3">
                  <div class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold shrink-0 mt-0.5 {step.status === 'Completed' ? 'bg-emerald-500 text-white' : idx === activePlan.current_step_index ? 'bg-primary text-primary-foreground' : 'bg-stone-200 dark:bg-stone-700 text-stone-600 dark:text-stone-400'}">
                    {step.status === 'Completed' ? '✓' : idx + 1}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between gap-2">
                      <div class="text-sm font-semibold text-stone-900 dark:text-stone-100 truncate">{step.title}</div>
                      <div class="flex items-center gap-1.5 shrink-0">
                        {#if step.requires_human_approval}
                          <span class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-300">
                            Approval Gate
                          </span>
                        {/if}
                        <span class="text-[10px] uppercase font-bold px-1.5 py-0.5 rounded {step.status === 'Completed' ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300' : step.status === 'AwaitingApproval' ? 'bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-300' : 'bg-stone-100 text-stone-600 dark:bg-stone-800 dark:text-stone-400'}">
                          {step.status}
                        </span>
                      </div>
                    </div>
                    <p class="text-xs text-stone-600 dark:text-stone-400 mt-0.5">{step.description}</p>
                    <div class="flex items-center gap-4 text-[11px] text-stone-500 dark:text-stone-400 mt-2">
                      <span>Tool: <code class="font-mono text-primary">{step.tool_command}</code></span>
                      {#if step.receipt_ref}
                        <span>Receipt: <code class="font-mono">{step.receipt_ref}</code></span>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="px-6 py-4 border-t border-stone-200 dark:border-stone-800 flex items-center justify-between bg-stone-50 dark:bg-stone-900/50">
      <div class="text-xs text-stone-500 dark:text-stone-400">
        Administrative tool only. OpenSeason never files or serves.
      </div>
      <div class="flex items-center gap-3">
        <button
          onclick={onClose}
          class="px-4 py-2 border border-stone-300 dark:border-stone-700 text-stone-700 dark:text-stone-300 rounded-lg text-sm hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
        >
          Close
        </button>
        {#if activePlan && !activePlan.is_complete}
          {@const currStep = activePlan.steps[activePlan.current_step_index]}
          {#if currStep?.requires_human_approval && !currStep.approval_granted}
            <button
              onclick={() => handleAdvance(true)}
              disabled={advancing}
              class="px-4 py-2 bg-amber-600 hover:bg-amber-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors shadow-sm"
            >
              {advancing ? "Executing..." : "Approve & Execute Step"}
            </button>
          {:else}
            <button
              onclick={() => handleAdvance(false)}
              disabled={advancing}
              class="px-4 py-2 bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground rounded-lg text-sm font-medium transition-colors shadow-sm"
            >
              {advancing ? "Executing..." : "Advance Step"}
            </button>
          {/if}
        {/if}
      </div>
    </div>
  </div>
</div>
