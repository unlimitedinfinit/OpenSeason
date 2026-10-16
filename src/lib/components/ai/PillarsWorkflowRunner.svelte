<script lang="ts">
  import { ipc } from "$lib/ipc";
  import type { WorkflowPlan, WorkflowStepResult } from "$lib/types/workflow";
  import type { BaselineExtensionAssessment } from "$lib/types/consultation";
  import type { CaseContextBundle } from "$lib/types/case";

  interface Props {
    caseId?: string;
    activePlan: WorkflowPlan | null;
    caseContext: CaseContextBundle | null;
    extensionAssessment: BaselineExtensionAssessment | null;
    onPlanUpdated: () => void;
  }

  let {
    caseId = "",
    activePlan,
    caseContext,
    extensionAssessment,
    onPlanUpdated,
  }: Props = $props();

  let advancing = $state(false);
  let error = $state("");
  let lastResult = $state<WorkflowStepResult | null>(null);
  let copiedText = $state("");

  async function handleAdvance(approve = false) {
    if (!caseId) return;
    advancing = true;
    error = "";
    try {
      const res = await ipc<WorkflowStepResult>("workflow_advance_cmd", { caseId, approve });
      lastResult = res;
      onPlanUpdated();
    } catch (e) {
      error = String(e);
    } finally {
      advancing = false;
    }
  }

  function copy(text: string, label: string) {
    navigator.clipboard.writeText(text);
    copiedText = label;
    setTimeout(() => (copiedText = ""), 3000);
  }
</script>

<div class="space-y-6">
  {#if error}
    <div class="p-3 bg-destructive/10 border border-destructive/30 rounded-xl text-xs text-destructive">
      {error}
    </div>
  {/if}

  {#if copiedText}
    <div class="p-2.5 bg-primary/10 border border-primary/30 rounded-lg text-xs text-primary flex items-center justify-between">
      <span>{copiedText} copied to clipboard.</span>
      <span class="text-[10px] text-muted-foreground">Ready to paste</span>
    </div>
  {/if}

  <!-- The 4 Pillars Dashboard -->
  <div class="grid gap-4 md:grid-cols-2">
    <!-- Pillar 1: Understand My Case -->
    <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-3.5 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-blue-500"></span>
          <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">1. Understand Case</h4>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20">Record Audit</span>
      </div>

      {#if caseContext}
        {@const urgentBranch = caseContext.next_steps?.branches?.find((b) => b.days_until_due != null) || caseContext.next_steps?.branches?.[0]}
        <div class="space-y-2">
          <div class="text-sm font-semibold text-foreground truncate">{caseContext.profile?.title || "Selected Case Matter"}</div>
          <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
            <span class="bg-secondary px-2 py-0.5 rounded font-mono text-[11px]">{caseContext.profile?.docket_number || "No docket #"}</span>
            <span class="bg-secondary px-2 py-0.5 rounded">{caseContext.profile?.document_kind || "reconsideration"}</span>
            <span class="bg-secondary px-2 py-0.5 rounded font-mono">{caseContext.exhibits?.items?.length || 0} evidence items</span>
          </div>

          {#if caseContext.venue_competence?.recommendations && caseContext.venue_competence.recommendations.length > 0}
            <div class="mt-2 p-2.5 rounded-xl bg-amber-500/10 border border-amber-500/25 space-y-1">
              <span class="text-[11px] font-semibold text-amber-300 flex items-center gap-1">
                ⚠️ Record Audit Recommendations ({caseContext.venue_competence.recommendations.length})
              </span>
              <ul class="text-[11px] text-amber-200/80 list-disc list-inside">
                {#each caseContext.venue_competence.recommendations.slice(0, 2) as rec}
                  <li class="truncate">{rec}</li>
                {/each}
              </ul>
            </div>
          {:else}
            <p class="text-xs text-emerald-400 flex items-center gap-1 pt-1">
              ✓ Core pleadings and evidence baseline identified.
            </p>
          {/if}
        </div>
      {:else}
        <p class="text-xs text-muted-foreground">Select an active case above or describe a situation to inspect the record.</p>
      {/if}
    </div>

    <!-- Pillar 2: Understand The Rules -->
    <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-3.5 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-purple-500"></span>
          <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">2. Understand Rules</h4>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-purple-500/10 text-purple-400 border border-purple-500/20">Jurisdiction</span>
      </div>

      {#if caseContext}
        {@const branch = caseContext.next_steps?.branches?.find((b) => b.days_until_due != null) || caseContext.next_steps?.branches?.[0]}
        <div class="space-y-2">
          <div class="text-sm font-semibold text-foreground truncate">{caseContext.venue_competence?.venue?.display_name || "Governing Court Rules"}</div>
          <div class="text-xs text-muted-foreground space-y-1">
            <p>Court: <strong class="text-foreground">{caseContext.profile?.court?.name || caseContext.venue_competence?.venue?.venue_code || "Trial Court"}</strong></p>
            {#if branch && branch.days_until_due != null}
              <p class="flex items-center gap-1.5 pt-1">
                <span class="w-2 h-2 rounded-full {branch.days_until_due <= 7 ? 'bg-red-400 animate-ping' : 'bg-amber-400'}"></span>
                <span>Deadline: <strong class="text-foreground">{branch.days_until_due}d remaining</strong> ({branch.title})</span>
              </p>
            {:else if branch?.computed_deadline}
              <p class="flex items-center gap-1.5 pt-1">
                <span class="w-2 h-2 rounded-full bg-amber-400"></span>
                <span>Deadline: <strong class="text-foreground">{branch.computed_deadline}</strong></span>
              </p>
            {/if}
          </div>
        </div>
      {:else}
        <p class="text-xs text-muted-foreground">Identifies governing court rules, tolling provisions, and mail-delay traps.</p>
      {/if}


    </div>

    <!-- Pillar 3: Figure Out Next Steps -->
    <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-3.5 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-amber-500"></span>
          <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">3. Next Steps</h4>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-400 border border-amber-500/20">Safeguards</span>
      </div>

      {#if extensionAssessment?.is_extension_recommended}
        <div class="space-y-2.5 p-3 rounded-xl bg-amber-500/10 border border-amber-500/25">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-amber-300">30-Day Extension Safeguard</span>
            <span class="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-amber-400/20 text-amber-200">Recommended</span>
          </div>
          <p class="text-[11px] text-amber-200/90 leading-relaxed">
            Record has missing items ({extensionAssessment.missing_elements.join(", ")}). Rule {extensionAssessment.governing_rule} permits a 30-day enlargement.
          </p>
          <div class="flex gap-2 pt-1">
            <button
              class="text-xs px-2.5 py-1 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border border-amber-500/40 transition-colors"
              onclick={() => copy(extensionAssessment.conferral_letter_draft, "Opposing Counsel Email")}
            >
              Copy Conferral Email
            </button>
            <button
              class="text-xs px-2.5 py-1 rounded bg-amber-500/20 hover:bg-amber-500/30 text-amber-200 border border-amber-500/40 transition-colors"
              onclick={() => copy(extensionAssessment.formal_motion_draft, "Formal Motion Draft")}
            >
              Copy Motion
            </button>
          </div>
        </div>
      {:else}
        <p class="text-xs text-muted-foreground">Predicts procedural paths, required filings, and baseline extensions.</p>
      {/if}
    </div>

    <!-- Pillar 4: Help Me Get It Done -->
    <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-3.5 shadow-sm">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
          <h4 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">4. Get It Done</h4>
        </div>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">Deterministic</span>
      </div>

      {#if activePlan}
        <div class="space-y-2">
          <div class="flex justify-between items-center text-xs">
            <span class="text-muted-foreground">Step {Math.min(activePlan.current_step_index + 1, activePlan.steps.length)} of {activePlan.steps.length}</span>
            <span class="font-medium {activePlan.is_complete ? 'text-emerald-400' : 'text-primary'}">
              {activePlan.is_complete ? 'All Steps Complete' : 'Executing Plan'}
            </span>
          </div>
          <div class="w-full bg-secondary h-2 rounded-full overflow-hidden">
            <div
              class="bg-primary h-full transition-all duration-300"
              style={`width: ${(Math.min(activePlan.current_step_index, activePlan.steps.length) / activePlan.steps.length) * 100}%`}
            ></div>
          </div>
          <p class="text-xs font-medium text-foreground truncate">{activePlan.objective}</p>
        </div>
      {:else}
        <p class="text-xs text-muted-foreground">Formulate a goal to generate court-formatted drafts, packets, and forms.</p>
      {/if}
    </div>
  </div>

  <!-- Active Plan Steps Execution Feed -->
  {#if activePlan && activePlan.steps.length > 0}
    <div class="rounded-2xl border border-border/70 bg-card/70 backdrop-blur-xl p-6 space-y-5 shadow-sm">
      <div class="flex items-center justify-between border-b border-border/50 pb-3">
        <div>
          <h3 class="text-base font-bold text-foreground">Action Roadmap: {activePlan.objective}</h3>
          <p class="text-xs text-muted-foreground mt-0.5">Single-engine verifiable tool execution with approval checkpoints.</p>
        </div>
        {#if !activePlan.is_complete}
          <button
            onclick={() => handleAdvance(false)}
            disabled={advancing}
            class="px-4 py-2 bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground text-xs font-semibold rounded-xl shadow-sm transition-opacity flex items-center gap-1.5"
          >
            {#if advancing}
              <span class="animate-spin text-sm">⏳</span> Executing...
            {:else}
              ▶ Advance Next Step
            {/if}
          </button>
        {/if}
      </div>

      <div class="space-y-3">
        {#each activePlan.steps as step, idx}
          {@const isCurrent = idx === activePlan.current_step_index}
          {@const isPast = idx < activePlan.current_step_index}
          <div class="rounded-xl border p-4 transition-all {isCurrent ? 'border-primary/60 bg-primary/5 shadow-sm' : isPast ? 'border-border/40 bg-background/40 opacity-70' : 'border-border/30 bg-background/20'}">
            <div class="flex items-start justify-between gap-3">
              <div class="flex items-start gap-3">
                <span class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-mono font-bold shrink-0 mt-0.5 {isPast ? 'bg-emerald-500/20 text-emerald-300' : isCurrent ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'}">
                  {isPast ? '✓' : idx + 1}
                </span>
                <div>
                  <h5 class="text-sm font-semibold text-foreground">{step.title}</h5>
                  <p class="text-xs text-muted-foreground mt-0.5">{step.description}</p>
                  {#if step.output_summary}
                    <div class="mt-2 text-xs font-mono p-2 rounded bg-background/80 border border-border/50 text-foreground/90">
                      {step.output_summary}
                    </div>
                  {/if}
                </div>
              </div>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full uppercase shrink-0 {isPast ? 'bg-emerald-500/10 text-emerald-400' : isCurrent ? 'bg-primary/20 text-primary border border-primary/30' : 'bg-muted text-muted-foreground'}">
                {step.status}
              </span>
            </div>

            {#if isCurrent && step.requires_human_approval && !step.approval_granted}
              <div class="mt-3 p-3 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-between gap-3">
                <div class="text-xs text-amber-300">
                  ⚠️ <strong>Human Sign-Off Required:</strong> {step.approval_prompt || 'Review step before proceeding.'}
                </div>
                <button
                  onclick={() => handleAdvance(true)}
                  disabled={advancing}
                  class="px-3 py-1.5 bg-amber-500 hover:bg-amber-600 text-stone-900 font-semibold text-xs rounded-lg transition-colors shrink-0"
                >
                  Confirm & Approve
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
