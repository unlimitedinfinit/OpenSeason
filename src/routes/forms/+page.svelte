<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import { ipc } from "$lib/ipc";
  import type { FormSearchResult, FetchFormResult, AttachFormResult, CaseRecommendations } from "$lib/types/packs";
  import type { WorkspaceCase } from "$lib/types/case";

  let forms = $state<FormSearchResult[]>([]);
  let cases = $state<WorkspaceCase[]>([]);
  let selectedCaseId = $state<string>("");
  let searchQuery = $state<string>("");
  let selectedJurisdiction = $state<string>("ALL");
  let selectedCategory = $state<string>("ALL");
  let selectedSource = $state<string>("ALL");
  let loading = $state<boolean>(false);
  let statusMessage = $state<string>("");
  let errorMessage = $state<string>("");
  let activeTab = $state<"catalog" | "recommended">("catalog");
  let caseRecommendations = $state<CaseRecommendations | null>(null);

  const categories = [
    { id: "ALL", label: "All Tasks" },
    { id: "housing", label: "Housing & Eviction" },
    { id: "family", label: "Family & Custody" },
    { id: "wills", label: "Wills & Estates" },
    { id: "contracts", label: "Contracts & NDAs" },
    { id: "court", label: "Court Filings & Motions" },
  ];

  const jurisdictions = [
    { id: "ALL", label: "All Jurisdictions" },
    { id: "CO", label: "Colorado" },
    { id: "NJ", label: "New Jersey" },
    { id: "CA", label: "California" },
    { id: "FED", label: "Federal / US Courts" },
  ];

  onMount(async () => {
    // Check URL parameters for initial category or query
    const urlParams = $page.url.searchParams;
    const cat = urlParams.get("category");
    if (cat) selectedCategory = cat;
    const q = urlParams.get("q");
    if (q) searchQuery = q;

    await loadCases();
    await search();
  });

  async function loadCases() {
    try {
      cases = await ipc<WorkspaceCase[]>("list_workspace_cases");
      if (cases.length > 0 && !selectedCaseId) {
        selectedCaseId = cases[0].profile.id;
        await loadRecommendations(selectedCaseId);
      }
    } catch {
      // Ignored if cases cannot be loaded
    }
  }

  async function loadRecommendations(caseId: string) {
    if (!caseId) return;
    try {
      caseRecommendations = await ipc<CaseRecommendations>("recommend_forms_cmd", { caseId });
    } catch {
      caseRecommendations = null;
    }
  }

  async function search() {
    loading = true;
    errorMessage = "";
    try {
      const stateFilter = selectedJurisdiction === "ALL" ? null : selectedJurisdiction;
      const taskFilter = selectedCategory === "ALL" ? null : selectedCategory;
      forms = await ipc<FormSearchResult[]>("search_forms_cmd", {
        query: searchQuery || null,
        state: stateFilter,
        task: taskFilter,
      });
    } catch (e) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }

  async function fetchForm(formId: string) {
    errorMessage = "";
    statusMessage = "";
    try {
      const res = await ipc<FetchFormResult>("fetch_form_cmd", { formId, allowNetwork: true });
      statusMessage = `Cached ${formId} (${res.size_label}). SHA-256 verified.`;
      await search();
      if (selectedCaseId) await loadRecommendations(selectedCaseId);
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function attachForm(formId: string, asTemplate: boolean) {
    if (!selectedCaseId) {
      errorMessage = "Select a case first to attach this document.";
      return;
    }
    errorMessage = "";
    statusMessage = "";
    try {
      const res = await ipc<AttachFormResult>("attach_form_cmd", {
        caseId: selectedCaseId,
        formId,
        asTemplate,
      });
      statusMessage = res.notice;
    } catch (e) {
      errorMessage = String(e);
    }
  }

  function matchesSource(item: FormSearchResult): boolean {
    if (selectedSource === "ALL") return true;
    if (selectedSource === "official" && item.source_type === "official_pdf") return true;
    if (selectedSource === "template" && (item.source_type === "editable_template" || item.source_type === "guide")) return true;
    return false;
  }
</script>

<main class="container mx-auto p-6 md:p-8 space-y-6 max-w-5xl">
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
    <div>
      <div class="flex items-center gap-2 mb-1">
        <span class="text-xs uppercase tracking-widest text-muted-foreground font-semibold">Legal Library</span>
        <span class="text-[10px] bg-primary/20 text-primary px-2 py-0.5 rounded-full font-mono">Offline-First</span>
      </div>
      <h1 class="text-3xl font-bold tracking-tight">Forms & Templates Library</h1>
      <p class="text-muted-foreground text-sm mt-1">
        Official court filings, everyday agreements, and editable document templates. Download once, work offline forever.
      </p>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <button onclick={() => goto("/packs")} class="text-xs border border-border bg-card px-3 py-2 rounded-md hover:border-primary/50 transition-colors">
        Manage Packs Archive
      </button>
      <button onclick={() => goto("/")} class="text-xs bg-secondary text-secondary-foreground px-3 py-2 rounded-md hover:opacity-90 transition-opacity">
        Back to Workspace
      </button>
    </div>
  </div>

  <ReviewNotice />

  {#if cases.length > 0}
    <div class="bg-card border border-border rounded-lg p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div class="space-y-0.5">
        <span class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Active Case Context</span>
        <p class="text-sm font-semibold">{cases.find((c) => c.profile.id === selectedCaseId)?.profile.title || "Select a case"}</p>
      </div>
      <div class="flex items-center gap-3">
        <select
          bind:value={selectedCaseId}
          onchange={() => loadRecommendations(selectedCaseId)}
          class="bg-background border border-border rounded-md px-3 py-1.5 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
        >
          {#each cases as c}
            <option value={c.profile.id}>{c.profile.title} ({c.profile.court.name || "General"})</option>
          {/each}
        </select>
        <button
          onclick={() => (activeTab = activeTab === "recommended" ? "catalog" : "recommended")}
          class="text-xs px-3 py-1.5 rounded-md border {activeTab === 'recommended' ? 'bg-primary text-primary-foreground border-primary' : 'bg-background border-border text-foreground hover:border-primary/40'}"
        >
          Recommended for Case
        </button>
      </div>
    </div>
  {/if}

  <!-- Search & Filter Controls -->
  <div class="space-y-3 bg-card border border-border rounded-lg p-4">
    <div class="flex flex-col md:flex-row gap-3">
      <div class="relative flex-1">
        <input
          type="text"
          bind:value={searchQuery}
          oninput={search}
          placeholder="Search by keywords: 'lease', 'custody', 'fee waiver', 'AO 240', 'reconsideration'..."
          class="w-full bg-background border border-border rounded-md px-3.5 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50"
        />
      </div>
      <div class="flex gap-2 shrink-0">
        <select
          bind:value={selectedJurisdiction}
          onchange={search}
          class="bg-background border border-border rounded-md px-3 py-2 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
        >
          {#each jurisdictions as j}
            <option value={j.id}>{j.label}</option>
          {/each}
        </select>
        <select
          bind:value={selectedSource}
          class="bg-background border border-border rounded-md px-3 py-2 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-primary"
        >
          <option value="ALL">All Sources</option>
          <option value="official">Official Court Forms</option>
          <option value="template">Everyday Templates</option>
        </select>
      </div>
    </div>

    <!-- Category Pills -->
    <div class="flex flex-wrap gap-1.5 pt-1">
      {#each categories as cat}
        <button
          onclick={() => { selectedCategory = cat.id; search(); }}
          class="text-xs px-3 py-1 rounded-full border transition-colors {selectedCategory === cat.id ? 'bg-primary text-primary-foreground border-primary font-medium' : 'bg-background border-border text-muted-foreground hover:text-foreground'}"
        >
          {cat.label}
        </button>
      {/each}
    </div>
  </div>

  {#if errorMessage}
    <div class="p-3 bg-destructive/10 border border-destructive/30 rounded-md text-xs text-destructive">
      {errorMessage}
    </div>
  {/if}

  {#if statusMessage}
    <div class="p-3 bg-primary/10 border border-primary/30 rounded-md text-xs text-primary">
      {statusMessage}
    </div>
  {/if}

  <!-- Recommendations View -->
  {#if activeTab === "recommended"}
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Recommended for Current Matter</h2>
        <span class="text-xs text-muted-foreground">{caseRecommendations?.recommendations.length || 0} tailored suggestions</span>
      </div>

      {#if !caseRecommendations?.recommendations || caseRecommendations.recommendations.length === 0}
        <div class="p-8 text-center border border-dashed border-border rounded-lg text-sm text-muted-foreground">
          No automated recommendations found for this matter type and court. You can browse the complete catalog below.
        </div>
      {:else}
        <div class="grid gap-3 md:grid-cols-2">
          {#each caseRecommendations.recommendations as rec}
            <article class="bg-card border border-border hover:border-primary/40 rounded-lg p-4 space-y-3 transition-colors">
              <div class="flex items-start justify-between gap-2">
                <div>
                  <span class="text-[10px] font-semibold uppercase px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20">
                    Recommended: {rec.relevance_reason}
                  </span>
                  <h3 class="text-base font-semibold text-foreground mt-2">{rec.form.title}</h3>
                  <p class="text-xs text-muted-foreground">{rec.form.court} · {rec.form.state}</p>
                </div>
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground shrink-0">
                  {rec.form.form_number || rec.form.id}
                </span>
              </div>

              {#if rec.form.teach.what_it_is}
                <p class="text-xs text-muted-foreground bg-background/50 p-2.5 rounded border border-border/50">
                  {rec.form.teach.what_it_is}
                </p>
              {/if}

              <div class="flex items-center justify-between pt-2 border-t border-border/50 text-xs">
                <span class="text-muted-foreground">{rec.form.size_label}</span>
                <div class="flex gap-2">
                  {#if !rec.form.is_offline}
                    <button onclick={() => fetchForm(rec.form.id)} class="text-xs bg-primary text-primary-foreground px-3 py-1 rounded hover:opacity-90">
                      Download
                    </button>
                  {:else}
                    <button onclick={() => attachForm(rec.form.id, true)} class="text-xs border border-primary text-primary px-3 py-1 rounded hover:bg-primary/10">
                      Attach as Draft
                    </button>
                    <button onclick={() => attachForm(rec.form.id, false)} class="text-xs bg-secondary text-secondary-foreground px-3 py-1 rounded hover:opacity-90">
                      Add to Filings
                    </button>
                  {/if}
                </div>
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </section>
  {/if}

  <!-- Catalog Grid -->
  {#if activeTab === "catalog"}
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Catalog Results</h2>
        <span class="text-xs text-muted-foreground">{forms.filter(matchesSource).length} documents available</span>
      </div>

      {#if loading}
        <div class="p-8 text-center text-sm text-muted-foreground">Searching catalog...</div>
      {:else if forms.filter(matchesSource).length === 0}
        <div class="p-8 text-center border border-dashed border-border rounded-lg text-sm text-muted-foreground">
          No forms or templates match your search criteria. Try a different task or jurisdiction.
        </div>
      {:else}
        <div class="grid gap-3 md:grid-cols-2">
          {#each forms.filter(matchesSource) as item}
            <article class="bg-card border border-border hover:border-primary/40 rounded-lg p-4 space-y-3 transition-colors flex flex-col justify-between">
              <div class="space-y-2">
                <div class="flex items-start justify-between gap-2">
                  <div class="space-y-1">
                    <div class="flex items-center gap-1.5">
                      <span class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded border {item.source_type === 'official_pdf' ? 'bg-blue-500/10 text-blue-300 border-blue-500/20' : 'bg-teal-500/10 text-teal-300 border-teal-500/20'}">
                        {item.source_type === 'official_pdf' ? 'Official Court Form' : 'Document Template'}
                      </span>
                      {#if item.is_offline}
                        <span class="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-300 border border-emerald-500/20">
                          Offline Ready
                        </span>
                      {/if}
                    </div>
                    <h3 class="text-sm font-semibold text-foreground leading-snug">{item.title}</h3>
                    <p class="text-xs text-muted-foreground">{item.court} · {item.state}</p>
                  </div>
                  <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground shrink-0">
                    {item.form_number || item.id}
                  </span>
                </div>

                {#if item.teach.what_it_is}
                  <p class="text-xs text-muted-foreground line-clamp-2">
                    {item.teach.what_it_is}
                  </p>
                {/if}
              </div>

              <div class="flex items-center justify-between pt-3 border-t border-border/50 text-xs">
                <span class="text-[11px] font-mono text-muted-foreground">{item.size_label}</span>
                <div class="flex gap-2">
                  {#if !item.is_offline}
                    <button onclick={() => fetchForm(item.id)} class="text-xs bg-primary text-primary-foreground px-2.5 py-1 rounded hover:opacity-90">
                      Download Offline
                    </button>
                  {:else if selectedCaseId}
                    <button onclick={() => attachForm(item.id, true)} class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50">
                      Draft
                    </button>
                    <button onclick={() => attachForm(item.id, false)} class="text-xs bg-secondary text-secondary-foreground px-2.5 py-1 rounded hover:opacity-90">
                      Attach
                    </button>
                  {:else}
                    <span class="text-xs text-muted-foreground">Cached</span>
                  {/if}
                </div>
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</main>
