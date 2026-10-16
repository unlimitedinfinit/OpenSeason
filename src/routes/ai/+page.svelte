<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { ipc } from "$lib/ipc";
  import type { WorkspaceCase, CaseContextBundle } from "$lib/types/case";
  import type { WorkflowPlan } from "$lib/types/workflow";
  import type { BaselineExtensionAssessment } from "$lib/types/consultation";
  import PillarsWorkflowRunner from "$lib/components/ai/PillarsWorkflowRunner.svelte";
  import McpDeveloperTab from "$lib/components/ai/McpDeveloperTab.svelte";

  interface FormRecommendation {
    form_id: string;
    title: string;
    category: string;
    jurisdiction: string;
    reason: string;
  }

  interface Manifest {
    never_files?: boolean;
    never_serves?: boolean;
    notice?: string;
    case_folder?: string;
    mcp_launch?: { command: string; args: string[]; cwd: string };
    tools?: { name: string; cli: string; mcp: string; summary: string }[];
    starter_prompt?: string;
  }

  let cases = $state<WorkspaceCase[]>([]);
  let selectedCaseId = $state("");
  let caseContext = $state<CaseContextBundle | null>(null);
  let activePlan = $state<WorkflowPlan | null>(null);
  let extensionAssessment = $state<BaselineExtensionAssessment | null>(null);
  let manifest = $state<Manifest | null>(null);
  let activeTab = $state<"studio" | "developer">("studio");

  let queryInput = $state("");
  let loading = $state(false);
  let error = $state("");
  let recommendedForms = $state<FormRecommendation[]>([]);

  const QUICK_PROMPTS = [
    { label: "⏱️ 30-Day Extension", text: "Assess baseline 30-day extension safeguard and draft opposing counsel conferral email" },
    { label: "📁 Reconstruct Record", text: "Retrieve missing hearing transcripts and reconcile official docket" },
    { label: "⚖️ Audit Court Order", text: "Audit court order findings and test factual discrepancies against evidence" },
    { label: "📝 Notice of Appeal", text: "Prepare and file Notice of Appeal with jurisdictional mail audit" },
    { label: "🔍 Find Legal Forms", text: "Search official court and legal forms for eviction defense, custody, or probate" },
  ];

  onMount(async () => {
    loading = true;
    try {
      cases = await ipc<WorkspaceCase[]>("list_workspace_cases").catch(() => []);
      manifest = await ipc<Manifest>("agent_manifest_cmd").catch(() => null);

      const urlCase = $page.url.searchParams.get("case");
      const urlGoal = $page.url.searchParams.get("goal");

      if (urlCase && cases.some((c) => c.profile.id === urlCase)) {
        selectedCaseId = urlCase;
      } else if (cases.length > 0) {
        selectedCaseId = cases[0].profile.id;
      }

      if (selectedCaseId) {
        await loadCaseData(selectedCaseId);
      }

      if (urlGoal) {
        queryInput = urlGoal;
        await handleExecutePrompt(urlGoal);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function onCaseChanged() {
    if (!selectedCaseId) {
      caseContext = null;
      activePlan = null;
      extensionAssessment = null;
      return;
    }
    loading = true;
    error = "";
    try {
      await loadCaseData(selectedCaseId);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadCaseData(id: string) {
    caseContext = await ipc<CaseContextBundle>("case_context_cmd", { caseId: id }).catch(() => null);
    activePlan = await ipc<WorkflowPlan | null>("workflow_status_cmd", { caseId: id }).catch(() => null);
    extensionAssessment = await ipc<BaselineExtensionAssessment>("extension_suggest_cmd", { caseId: id }).catch(() => null);
  }

  async function handleExecutePrompt(overrideText?: string) {
    const text = (overrideText || queryInput).trim();
    if (!text) return;
    loading = true;
    error = "";
    recommendedForms = [];

    try {
      // Check if query is looking for forms
      const lower = text.toLowerCase();
      if (lower.includes("form") || lower.includes("eviction") || lower.includes("probate") || lower.includes("custody") || lower.includes("lease")) {
        const res = await ipc<{ forms?: FormRecommendation[] }>("recommend_forms_cmd", { query: text }).catch(() => ({ forms: [] }));
        recommendedForms = res.forms || [];
      }

      // If a case is selected, formulate an active deterministic plan
      if (selectedCaseId) {
        activePlan = await ipc<WorkflowPlan>("workflow_plan_cmd", {
          caseId: selectedCaseId,
          objective: text,
        });
        await loadCaseData(selectedCaseId);
      } else if (cases.length > 0) {
        // Automatically attach to the first case
        selectedCaseId = cases[0].profile.id;
        activePlan = await ipc<WorkflowPlan>("workflow_plan_cmd", {
          caseId: selectedCaseId,
          objective: text,
        });
        await loadCaseData(selectedCaseId);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<main class="container mx-auto p-6 md:p-8 space-y-6 max-w-5xl">
  <!-- Top Navigation & Header -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-border/50 pb-5">
    <div class="space-y-1">
      <div class="flex items-center gap-2.5">
        <span class="w-3 h-3 rounded-full bg-primary animate-pulse"></span>
        <h1 class="text-2xl md:text-3xl font-bold tracking-tight text-foreground">JustLegal Assistant Studio</h1>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-primary/20 text-primary border border-primary/30">
          Agent-Operable OS
        </span>
      </div>
      <p class="text-xs text-muted-foreground">
        Tell the assistant what you want to achieve. The assistant operates the 4 procedural pillars and manages the court record.
      </p>
    </div>

    <div class="flex items-center gap-2 bg-secondary/80 p-1 rounded-xl border border-border/60">
      <button
        onclick={() => (activeTab = "studio")}
        class="text-xs font-semibold px-3 py-1.5 rounded-lg transition-all {activeTab === 'studio' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
      >
        Assistant Studio
      </button>
      <button
        onclick={() => (activeTab = "developer")}
        class="text-xs font-semibold px-3 py-1.5 rounded-lg transition-all {activeTab === 'developer' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
      >
        MCP & Agent Engine
      </button>
    </div>
  </div>

  {#if error}
    <div class="p-3.5 bg-destructive/10 border border-destructive/30 rounded-xl text-xs text-destructive flex items-center justify-between">
      <span>{error}</span>
      <button class="text-destructive font-bold text-sm px-2" onclick={() => (error = "")}>&times;</button>
    </div>
  {/if}

  {#if activeTab === "studio"}
    <!-- Case Context Selector Ribbon -->
    <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 shadow-sm">
      <div class="flex items-center gap-3 w-full sm:w-auto">
        <label for="case-selector" class="text-xs font-bold uppercase tracking-wider text-muted-foreground whitespace-nowrap">
          Target Case:
        </label>
        <select
          id="case-selector"
          bind:value={selectedCaseId}
          onchange={onCaseChanged}
          class="flex-1 sm:flex-none text-xs bg-background border border-border rounded-xl px-3 py-2 text-foreground focus:outline-none focus:ring-2 focus:ring-primary font-medium"
        >
          {#if cases.length === 0}
            <option value="">No Active Cases (General Intake)</option>
          {:else}
            {#each cases as item}
              <option value={item.profile.id}>
                {item.profile.title || "Untitled Matter"} {item.profile.docket_number ? `(${item.profile.docket_number})` : ""}
              </option>
            {/each}
          {/if}
        </select>
      </div>

      <div class="flex items-center gap-2">
        {#if selectedCaseId}
          <button
            onclick={() => goto(`/case/${selectedCaseId}`)}
            class="text-xs border border-border/80 bg-secondary/80 hover:border-primary/50 text-foreground px-3 py-1.5 rounded-xl transition-colors"
          >
            Open Full Case Workspace ↗
          </button>
        {/if}
        <button
          onclick={() => goto("/case/new")}
          class="text-xs bg-primary/10 text-primary hover:bg-primary/20 border border-primary/30 px-3 py-1.5 rounded-xl transition-colors font-medium"
        >
          + New Matter
        </button>
      </div>
    </div>

    <!-- Hero Interactive Command Bar -->
    <div class="rounded-3xl border border-primary/40 bg-gradient-to-b from-card/90 to-card/60 backdrop-blur-2xl p-6 space-y-4 shadow-xl">
      <div class="space-y-1">
        <label for="assistant-input" class="text-sm font-semibold text-foreground flex items-center gap-2">
          <span>What are you trying to accomplish?</span>
          {#if loading}
            <span class="text-xs text-primary animate-pulse font-normal">Analyzing legal parameters...</span>
          {/if}
        </label>
        <div class="flex gap-2">
          <input
            id="assistant-input"
            type="text"
            bind:value={queryInput}
            onkeydown={(e) => e.key === "Enter" && handleExecutePrompt()}
            placeholder="e.g. I need a 30-day extension, or I have a folder of court papers to organize, or I need eviction defense forms..."
            class="flex-1 px-4 py-3 text-sm rounded-2xl border border-border bg-background/90 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary shadow-inner"
          />
          <button
            onclick={() => handleExecutePrompt()}
            disabled={loading || !queryInput.trim()}
            class="px-5 py-3 bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground font-semibold text-sm rounded-2xl shadow-md transition-opacity flex items-center gap-2 shrink-0"
          >
            {#if loading}
              <span class="animate-spin text-base">⚙️</span>
            {:else}
              <span>Ask Assistant</span>
            {/if}
          </button>
        </div>
      </div>

      <!-- Quick Action Prompt Pills -->
      <div class="flex flex-wrap items-center gap-2 pt-1">
        <span class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mr-1">Quick Goals:</span>
        {#each QUICK_PROMPTS as prompt}
          <button
            type="button"
            onclick={() => {
              queryInput = prompt.text;
              handleExecutePrompt(prompt.text);
            }}
            class="text-xs px-3 py-1.5 rounded-full border border-border/80 bg-secondary/60 hover:border-primary/60 hover:bg-card text-foreground transition-all shadow-sm"
          >
            {prompt.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Forms Recommendation Results (if query matched forms) -->
    {#if recommendedForms.length > 0}
      <div class="rounded-2xl border border-border/70 bg-card/70 backdrop-blur-xl p-5 space-y-3 shadow-sm">
        <div class="flex items-center justify-between">
          <h4 class="text-sm font-bold text-foreground">Recommended Forms for "{queryInput}"</h4>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-secondary text-muted-foreground">Official Pack Catalog</span>
        </div>
        <div class="grid gap-3 sm:grid-cols-2 md:grid-cols-3">
          {#each recommendedForms as form}
            <div class="p-3 rounded-xl border border-border/60 bg-background/60 space-y-1.5">
              <span class="text-[10px] font-mono uppercase px-1.5 py-0.5 rounded bg-primary/10 text-primary border border-primary/20">{form.jurisdiction} · {form.category}</span>
              <h5 class="text-xs font-semibold text-foreground truncate">{form.title}</h5>
              <p class="text-[11px] text-muted-foreground line-clamp-2">{form.reason}</p>
              <button
                onclick={() => goto(`/forms?search=${encodeURIComponent(form.title)}`)}
                class="text-xs text-primary font-medium hover:underline pt-1 block"
              >
                Inspect & Fill Form →
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Live 4-Pillar Pipeline Runner -->
    <PillarsWorkflowRunner
      caseId={selectedCaseId}
      {activePlan}
      {caseContext}
      {extensionAssessment}
      onPlanUpdated={() => selectedCaseId && loadCaseData(selectedCaseId)}
    />
  {:else}
    <!-- Developer & External MCP Connection -->
    <McpDeveloperTab {manifest} />
  {/if}
</main>
