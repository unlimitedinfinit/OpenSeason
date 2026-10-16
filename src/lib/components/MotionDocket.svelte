<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseFilingItem } from "$lib/types/case";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";

  interface Props {
    caseId: string;
    onselectmotion?: (filing: CaseFilingItem) => void;
    onexport?: (paths: { docx: string; pdf: string }) => void;
  }

  let { caseId, onselectmotion, onexport }: Props = $props();

  let filings = $state<CaseFilingItem[]>([]);
  let selectedId = $state<string | null>(null);
  let activeFiling = $state<CaseFilingItem>({
    id: "",
    title: "",
    kind: "motion",
    rule: "",
    status: "draft",
    user_text: "",
    relief_sought: "",
    created_at: "",
    updated_at: "",
  });

  let filterKind = $state<string>("all");
  let loading = $state(false);
  let saving = $state(false);
  let exporting = $state(false);
  let statusMessage = $state("");
  let errorMessage = $state("");
  let lastExport = $state<{ docx: string; pdf: string } | null>(null);
  let pendingDeleteId = $state<string | null>(null);

  const FILING_KINDS = [
    { id: "motion", label: "Standard Motion", emoji: "⚖️", color: "teal" },
    { id: "reconsideration", label: "Reconsideration", emoji: "🔄", color: "teal" },
    { id: "stay", label: "Stay Pending Appeal", emoji: "⏸️", color: "amber" },
    { id: "enlarge", label: "Enlarge Time / Extension", emoji: "🟢", color: "emerald" },
    { id: "response", label: "Response / Opposition", emoji: "🟠", color: "orange" },
    { id: "brief", label: "Merits / Supp. Brief", emoji: "🔴", color: "rose" },
    { id: "custom", label: "Custom Pleading", emoji: "⚪", color: "zinc" },
  ];

  onMount(loadFilings);

  export async function loadFilings() {
    loading = true;
    errorMessage = "";
    try {
      const items = await ipc<CaseFilingItem[]>("list_case_filings_cmd", { caseId });
      filings = items || [];
      if (filings.length > 0 && !selectedId) {
        selectFiling(filings[0]);
      }
    } catch (e) {
      errorMessage = String(e);
    } finally {
      loading = false;
    }
  }

  function selectFiling(filing: CaseFilingItem) {
    selectedId = filing.id;
    activeFiling = { ...filing };
    statusMessage = "";
    errorMessage = "";
    lastExport = null;
    if (onselectmotion) onselectmotion(filing);
  }

  function startNewFiling(kind = "motion") {
    const kindObj = FILING_KINDS.find((k) => k.id === kind) || FILING_KINDS[0];
    selectedId = "new";
    activeFiling = {
      id: "",
      title: `Motion for ${kindObj.label}`,
      kind,
      rule: kind === "reconsideration" ? "R. 2:11-6" : kind === "stay" ? "R. 2:9-5" : kind === "enlarge" ? "R. 2:4-4" : "",
      status: "draft",
      user_text: "",
      relief_sought: "",
      created_at: "",
      updated_at: "",
    };
    statusMessage = "Started a new filing draft. Enter your text and save.";
    errorMessage = "";
    lastExport = null;
  }

  async function saveFiling() {
    if (!activeFiling.title.trim()) {
      errorMessage = "Please enter a title for this filing.";
      return;
    }
    saving = true;
    errorMessage = "";
    statusMessage = "";
    try {
      const saved = await ipc<CaseFilingItem>("save_case_filing_cmd", {
        caseId,
        filing: activeFiling,
      });
      statusMessage = `Saved "${saved.title}" successfully.`;
      activeFiling = { ...saved };
      selectedId = saved.id;
      await loadFilings();
      if (onselectmotion) onselectmotion(saved);
    } catch (e) {
      errorMessage = String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteFiling(id: string) {
    pendingDeleteId = id;
  }

  async function confirmDelete() {
    const id = pendingDeleteId;
    pendingDeleteId = null;
    if (!id) return;
    try {
      await ipc<boolean>("delete_case_filing_cmd", { caseId, filingId: id });
      statusMessage = "Filing removed from docket.";
      selectedId = null;
      await loadFilings();
      if (filings.length > 0) {
        selectFiling(filings[0]);
      } else {
        startNewFiling();
      }
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function exportFiling() {
    if (!activeFiling.id || activeFiling.id === "new") {
      await saveFiling();
    }
    if (!activeFiling.id) return;
    exporting = true;
    errorMessage = "";
    statusMessage = "";
    try {
      const result = await ipc<{ docx: string; pdf: string }>("export_case_filing_cmd", {
        caseId,
        filingId: activeFiling.id,
      });
      lastExport = result;
      statusMessage = `Export complete! Court-formatted PDF and Word files generated.`;
      activeFiling.status = "exported";
      await saveFiling();
      if (onexport) onexport(result);
    } catch (e) {
      errorMessage = String(e);
    } finally {
      exporting = false;
    }
  }

  let filteredFilings = $derived(
    filterKind === "all" ? filings : filings.filter((f) => f.kind === filterKind)
  );

  function getKindInfo(kind: string) {
    return FILING_KINDS.find((k) => k.id === kind) || { id: kind, label: kind, emoji: "📄", color: "zinc" };
  }

  function wordCount(str: string) {
    return str.trim() ? str.trim().split(/\s+/).length : 0;
  }
</script>

<div class="space-y-6">
  <!-- Top Banner & Action Header -->
  <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 shadow-sm relative overflow-hidden">
    <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="h-12 w-12 rounded-xl bg-primary/10 border border-primary/25 flex items-center justify-center text-2xl shadow-inner text-primary">
          📑
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h2 class="text-lg font-bold text-foreground tracking-tight">Case Motions & Pleadings Docket</h2>
            <span class="text-xs px-2.5 py-0.5 rounded-full bg-secondary text-foreground border border-border font-medium">
              {filings.length} {filings.length === 1 ? 'filing' : 'filings'}
            </span>
          </div>
          <p class="text-xs text-muted-foreground mt-0.5">
            Manage multiple independent motions, stays, extensions, and responses for this case. Never overwrite existing drafts.
          </p>
        </div>
      </div>

      <!-- Quick New Filing Button -->
      <div class="flex items-center gap-2">
        <button
          onclick={() => startNewFiling("reconsideration")}
          class="px-3 py-1.5 rounded-xl bg-secondary hover:bg-card text-foreground border border-border text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm"
        >
          <span>⚖️</span> + Reconsideration
        </button>
        <button
          onclick={() => startNewFiling("stay")}
          class="px-3 py-1.5 rounded-xl bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border border-amber-500/30 text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm"
        >
          <span>⏱️</span> + Stay
        </button>
        <button
          onclick={() => startNewFiling("motion")}
          class="px-3.5 py-1.5 rounded-xl bg-primary hover:opacity-90 text-primary-foreground text-xs font-bold flex items-center gap-1.5 transition-all shadow-md active:scale-95"
        >
          <span>➕</span> New Motion
        </button>
      </div>
    </div>
  </div>

  <!-- Main 2-Column Grid: List on Left, Active Editor on Right -->
  <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
    <!-- Left Column: Filings List -->
    <div class="lg:col-span-4 space-y-3">
      <!-- Filter Bar -->
      <div class="flex items-center justify-between gap-2 pb-1">
        <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Docket Entries</span>
        <select
          bind:value={filterKind}
          class="bg-background border border-border text-foreground text-xs rounded-lg px-2 py-1 focus:ring-1 focus:ring-primary"
        >
          <option value="all">All Kinds ({filings.length})</option>
          {#each FILING_KINDS as k}
            <option value={k.id}>{k.emoji} {k.label}</option>
          {/each}
        </select>
      </div>

      {#if loading}
        <div class="rounded-xl border border-border/60 bg-card/40 p-8 text-center text-muted-foreground text-xs animate-pulse">
          Loading docket entries...
        </div>
      {:else if filteredFilings.length === 0}
        <div class="rounded-xl border border-dashed border-border bg-card/20 p-8 text-center space-y-3">
          <div class="text-3xl">📭</div>
          <p class="text-xs text-muted-foreground">No motions or pleadings matching this filter.</p>
          <button
            onclick={() => startNewFiling("motion")}
            class="text-xs px-3 py-1.5 rounded-lg bg-primary hover:opacity-90 text-primary-foreground font-medium"
          >
            Create First Motion
          </button>
        </div>
      {:else}
        <div class="space-y-2 max-h-[600px] overflow-y-auto pr-1">
          {#each filteredFilings as f (f.id)}
            {@const info = getKindInfo(f.kind)}
            {@const isSelected = selectedId === f.id}
            <!-- Filing Card -->
            <button
              type="button"
              onclick={() => selectFiling(f)}
              class="w-full text-left rounded-xl border p-3.5 transition-all {isSelected
                ? 'border-primary bg-primary/10 shadow-md ring-1 ring-primary/40'
                : 'border-border/60 bg-card/60 hover:bg-card/90 hover:border-border'}"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="flex items-center gap-1.5">
                  <span class="text-sm">{info.emoji}</span>
                  <span class="text-xs font-semibold text-foreground line-clamp-1">{f.title || 'Untitled Motion'}</span>
                </div>
                <!-- Status Pill -->
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full {
                  f.status === 'exported' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' :
                  f.status === 'ready' ? 'bg-primary/20 text-primary border border-primary/30' :
                  'bg-secondary text-muted-foreground border border-border'
                }">
                  {f.status}
                </span>
              </div>

              {#if f.rule}
                <div class="mt-1 text-[11px] text-muted-foreground font-mono">
                  Rule: {f.rule}
                </div>
              {/if}

              {#if f.relief_sought}
                <p class="mt-1 text-[11px] text-muted-foreground line-clamp-1 italic">
                  "{f.relief_sought}"
                </p>
              {/if}

              <div class="mt-2 flex items-center justify-between text-[10px] text-muted-foreground border-t border-border/40 pt-1.5">
                <span>{wordCount(f.user_text)} words</span>
                <span>{f.updated_at ? new Date(f.updated_at).toLocaleDateString() : 'Draft'}</span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Right Column: Drafting & Export Studio -->
    <div class="lg:col-span-8 space-y-4">
      <div class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-4 shadow-sm">
        <!-- Editor Header -->
        <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-border/50 pb-4">
          <div class="flex items-center gap-2">
            <span class="text-xl">{getKindInfo(activeFiling.kind).emoji}</span>
            <div>
              <h3 class="font-bold text-sm text-foreground">
                {selectedId === "new" ? "New Filing / Motion Draft" : `Editing: ${activeFiling.title}`}
              </h3>
              <p class="text-xs text-muted-foreground">OpenSeason formats your words into court-ready Word & PDF files.</p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            {#if activeFiling.id && activeFiling.id !== "new"}
              <button
                onclick={() => deleteFiling(activeFiling.id)}
                class="px-2.5 py-1.5 rounded-lg border border-destructive/30 bg-destructive/10 hover:bg-destructive/20 text-destructive text-xs font-medium transition-colors"
                title="Remove this filing"
              >
                🗑️ Delete
              </button>
            {/if}
            <button
              onclick={saveFiling}
              disabled={saving}
              class="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white text-xs font-semibold flex items-center gap-1.5 shadow transition-all"
            >
              <span>💾</span> {saving ? 'Saving...' : 'Save Draft'}
            </button>
            <button
              onclick={exportFiling}
              disabled={exporting || saving}
              class="px-3.5 py-1.5 rounded-lg bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground text-xs font-bold flex items-center gap-1.5 shadow-md transition-all active:scale-95"
            >
              <span>🚀</span> {exporting ? 'Compiling...' : 'Export Word + PDF'}
            </button>
          </div>
        </div>


        <!-- Feedback Alerts -->
        {#if statusMessage}
          <div class="rounded-xl border border-emerald-500/30 bg-emerald-950/30 p-3 text-xs text-emerald-300 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span>✅</span>
              <span>{statusMessage}</span>
            </div>
            {#if lastExport}
              <div class="flex items-center gap-2">
                <span class="text-[10px] text-zinc-400">Saved to exports/</span>
              </div>
            {/if}
          </div>
        {/if}

        {#if errorMessage}
          <div class="rounded-xl border border-rose-500/30 bg-rose-950/30 p-3 text-xs text-rose-300 flex items-center gap-2">
            <span>⚠️</span>
            <span>{errorMessage}</span>
          </div>
        {/if}

        <!-- Metadata Form Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-12 gap-3">
          <!-- Filing Title -->
          <div class="sm:col-span-6 space-y-1">
            <label class="text-xs font-semibold text-zinc-300 flex items-center gap-1">
              <span>Filing Title</span>
              <span class="text-rose-400">*</span>
            </label>
            <input
              type="text"
              bind:value={activeFiling.title}
              placeholder="e.g. Motion for Reconsideration of January 16 Order"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <!-- Filing Kind -->
          <div class="sm:col-span-3 space-y-1">
            <label class="text-xs font-semibold text-zinc-300">Filing Kind</label>
            <select
              bind:value={activeFiling.kind}
              class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 focus:border-primary focus:outline-none"
            >
              {#each FILING_KINDS as k}
                <option value={k.id}>{k.emoji} {k.label}</option>
              {/each}
            </select>
          </div>

          <!-- Status -->
          <div class="sm:col-span-3 space-y-1">
            <label class="text-xs font-semibold text-zinc-300">Status</label>
            <select
              bind:value={activeFiling.status}
              class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 focus:border-primary focus:outline-none"
            >
              <option value="draft">📝 Draft</option>
              <option value="ready">⚡ Ready</option>
              <option value="exported">📤 Exported</option>
            </select>
          </div>

          <!-- Rule / Authority -->
          <div class="sm:col-span-4 space-y-1">
            <label class="text-xs font-semibold text-zinc-300">Governing Rule / Statute</label>
            <input
              type="text"
              bind:value={activeFiling.rule}
              placeholder="e.g. R. 2:11-6, C.A.R. 27"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-primary focus:outline-none"
            />
          </div>

          <!-- Specific Relief Sought -->
          <div class="sm:col-span-8 space-y-1">
            <label class="text-xs font-semibold text-zinc-300">Specific Relief Sought</label>
            <input
              type="text"
              bind:value={activeFiling.relief_sought}
              placeholder="e.g. Vacating the dismissal order or 30-day extension of time"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-primary focus:outline-none"
            />
          </div>
        </div>

        <!-- Draft Textarea -->
        <div class="space-y-1.5 pt-1">
          <div class="flex items-center justify-between">
            <label class="text-xs font-semibold text-zinc-200 flex items-center gap-1.5">
              <span>Motion Body & Legal Argument</span>
              <span class="text-[10px] text-zinc-500 font-normal">(User's own words only; no hallucinated citations)</span>
            </label>
            <div class="text-[11px] text-zinc-400 font-mono">
              {wordCount(activeFiling.user_text)} words • {activeFiling.user_text.length} chars
            </div>
          </div>
          <textarea
            bind:value={activeFiling.user_text}
            rows="12"
            placeholder="Type or paste your motion argument, procedural history, and reasons supporting the relief requested here... OpenSeason compiles your own text into court margins, line spacing, captions, and signature blocks."
            class="w-full rounded-xl border border-border bg-background/80 p-3.5 text-xs text-foreground placeholder:text-muted-foreground font-mono leading-relaxed focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
          ></textarea>
        </div>

        <!-- Export Paths Badge when exported -->
        {#if lastExport}
          <div class="rounded-xl border border-primary/30 bg-primary/5 p-4 space-y-2">
            <div class="text-xs font-semibold text-primary flex items-center gap-1.5">
              <span>📄</span> Generated Court Files:
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs font-mono">
              <div class="rounded-lg bg-background p-2.5 border border-border text-foreground break-all">
                <span class="text-primary font-bold">PDF:</span> {lastExport.pdf}
              </div>
              <div class="rounded-lg bg-background p-2.5 border border-border text-foreground break-all">
                <span class="text-muted-foreground font-bold">Word:</span> {lastExport.docx}
              </div>
            </div>
          </div>
        {/if}

        <!-- Safety Notice Footer -->
        <div class="text-[11px] text-zinc-500 border-t border-zinc-800/80 pt-3 flex items-center gap-2">
          <span>⚖️</span>
          <span>OpenSeason is a formatting and assembly tool. It never files with the court or serves opposing counsel. Review all output carefully before filing.</span>
        </div>
      </div>
    </div>
  </div>
</div>

<ConfirmDialog
  open={pendingDeleteId !== null}
  title="Remove this filing?"
  body="This removes the filing from the case docket. It does not file or un-file anything with a court."
  confirmLabel="Remove"
  danger={true}
  onconfirm={confirmDelete}
  oncancel={() => (pendingDeleteId = null)}
/>
