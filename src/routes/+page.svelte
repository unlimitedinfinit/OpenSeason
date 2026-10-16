<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import PdfToolsModal from "$lib/components/PdfToolsModal.svelte";
  import SignatureStudioModal from "$lib/components/SignatureStudioModal.svelte";
  import PromptDialog from "$lib/components/PromptDialog.svelte";
  import { ipc, runningInTauri } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type { WorkspaceCase, CloudSession, FolderImportReport, BootstrapResult, PortfolioStatusReport, PortfolioCaseItem } from "$lib/types/case";

  let cases = $state<WorkspaceCase[]>([]);
  let casesRoot = $state("");
  let portfolio = $state<PortfolioStatusReport | null>(null);
  let loadError = $state("");
  let actionError = $state("");
  let status = $state("");
  let showCloudModal = $state(false);
  let showPdfToolsModal = $state(false);
  let showSignatureModal = $state(false);
  let cloudSession = $state<CloudSession | null>(null);
  let promptKind = $state<"folder" | "osb" | "import" | "bootstrap" | null>(null);
  let heroQuery = $state("");

  const QUICK_GOALS = [
    { label: "⏱️ Need 30-Day Extension", goal: "Assess baseline 30-day extension safeguard and draft opposing counsel conferral email" },
    { label: "📁 Auto-Organize Court Folder", action: () => (promptKind = "bootstrap") },
    { label: "⚖️ Audit Court Order", goal: "Audit court order findings and test factual discrepancies against evidence" },
    { label: "📝 File Notice of Appeal", goal: "Prepare and file Notice of Appeal with jurisdictional mail audit" },
    { label: "🔍 Search Legal Forms", action: () => goto("/forms") },
  ];

  const RESOURCE_CATEGORIES = [
    { id: "housing", label: "Housing", desc: "Eviction defense, leases, repairs", icon: "🏠" },
    { id: "family", label: "Family", desc: "Custody, support, parenting", icon: "👨‍👩‍👧" },
    { id: "wills", label: "Wills & Estates", desc: "Probate, directives, POA", icon: "📜" },
    { id: "contracts", label: "Contracts", desc: "NDAs, contractor, bills of sale", icon: "💼" },
    { id: "court", label: "Court Forms", desc: "Fee waivers, summons, notices", icon: "⚖️" },
    { id: "all", label: "Full Library", desc: "Browse all official packs", icon: "📚" },
  ];

  onMount(() => {
    load();
    checkCloud();
  });

  async function checkCloud() {
    try {
      cloudSession = await ipc<CloudSession | null>("cloud_status_cmd");
    } catch {
      cloudSession = null;
    }
  }

  async function load() {
    try {
      casesRoot = await ipc<string>("get_cases_root");
      cases = await ipc<WorkspaceCase[]>("list_workspace_cases");
      portfolio = await ipc<PortfolioStatusReport>("portfolio_status_cmd").catch(() => null);
      loadError = "";
    } catch (e) {
      loadError = String(e);
    }
  }

  function handleHeroSubmit() {
    const q = heroQuery.trim();
    if (!q) return;
    goto(`/ai?goal=${encodeURIComponent(q)}`);
  }

  function startNew(kind: string = "reconsideration") {
    goto(`/case/new?kind=${encodeURIComponent(kind)}`);
  }

  async function handlePrompt(value: string) {
    const kind = promptKind;
    promptKind = null;
    actionError = "";
    try {
      if (kind === "folder") {
        const profile = await ipc<{ id: string }>("open_case_folder", { path: value || "." });
        goto(`/case/${profile.id}`);
      } else if (kind === "osb") {
        if (!session.unlocked) {
          actionError = "Unlock Confidential mode first. Import reads a vault bundle (.osb).";
          return;
        }
        const id = await ipc<string>("import_hunt_cmd", { osbPath: value || "sample.osb" });
        status = "Imported the confidential bundle.";
        await load();
        goto(`/case/${id}`);
      } else if (kind === "import") {
        const report = await ipc<FolderImportReport>("import_folder_cmd", {
          from: value,
          title: "Imported matter",
        });
        status = report.notice;
        await load();
        if (report.case_id) goto(`/case/${report.case_id}`);
      } else if (kind === "bootstrap") {
        const res = await ipc<BootstrapResult>("case_bootstrap_cmd", {
          sourceFolder: value || "sample-docket-folder",
          destinationParent: null,
          caseTitle: null,
        });
        status = res.summary;
        await load();
        goto(`/case/${res.case_id}`);
      }
    } catch (e) {
      actionError = String(e);
    }
  }

  async function trySample() {
    actionError = "";
    try {
      const profile = await ipc<{ id: string }>("seed_sample_cmd");
      status = "Opened sample case (Lucille Example). Nothing was filed.";
      await load();
      goto(`/case/${profile.id}`);
    } catch (e) {
      actionError = String(e);
    }
  }

  function getMatterUrgency(caseId: string): PortfolioCaseItem | undefined {
    return portfolio?.matters?.find((m) => m.case_id === caseId);
  }
</script>

<main class="container mx-auto p-4 md:p-6 space-y-5 max-w-6xl">
  <!-- Top Bar: Apple-Style Compact Header -->
  <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
    <div class="flex items-center gap-2.5">
      <div class="w-8 h-8 rounded-xl bg-primary/20 border border-primary/30 flex items-center justify-center text-primary font-bold shadow-sm">
        ⚖️
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-xl md:text-2xl font-bold tracking-tight text-foreground">JustLegal Desktop</h1>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-primary/10 text-primary border border-primary/20">
            Local-First · Private
          </span>
        </div>
        <p class="text-xs text-muted-foreground">Your sovereign legal operating system and case workspace</p>
      </div>
    </div>

    <!-- Quick Utilities Ribbon -->
    <div class="flex flex-wrap items-center gap-1.5 shrink-0">
      <button onclick={() => (showPdfToolsModal = true)} class="text-xs border border-border/80 bg-card/80 hover:border-primary/50 text-foreground px-3 py-1.5 rounded-xl transition-all shadow-sm">
        PDF Studio
      </button>
      <button onclick={() => (showSignatureModal = true)} class="text-xs border border-border/80 bg-card/80 hover:border-primary/50 text-foreground px-3 py-1.5 rounded-xl transition-all shadow-sm">
        Signature Block
      </button>
      <button onclick={() => (showCloudModal = true)} class="text-xs border border-border/80 bg-card/80 hover:border-primary/50 text-foreground px-3 py-1.5 rounded-xl transition-all shadow-sm">
        {cloudSession ? `Account: ${cloudSession.email.split("@")[0]}` : "JustLegal.me"}
      </button>
      <button onclick={() => goto("/confidential")} class="text-xs border border-red-500/30 text-red-300 hover:bg-red-500/10 px-3 py-1.5 rounded-xl transition-all shadow-sm">
        {session.unlocked ? "Confidential Vault" : "Unlock Vault"}
      </button>
    </div>
  </div>

  {#if actionError}
    <div class="p-3 bg-destructive/10 border border-destructive/30 rounded-xl text-xs text-destructive flex items-center justify-between">
      <span>{actionError}</span>
      <button class="font-bold text-sm px-1 text-destructive" onclick={() => (actionError = "")}>&times;</button>
    </div>
  {/if}

  {#if status}
    <div class="p-3 bg-primary/10 border border-primary/30 rounded-xl text-xs text-primary flex items-center justify-between">
      <span>{status}</span>
      <button class="font-bold text-sm px-1 text-primary" onclick={() => (status = "")}>&times;</button>
    </div>
  {/if}

  <!-- Spotlight Hero: Central AI Command Bar -->
  <div class="rounded-3xl border border-primary/30 bg-gradient-to-b from-card/80 to-card/50 backdrop-blur-2xl p-5 md:p-6 space-y-3.5 shadow-xl">
    <div class="space-y-1">
      <div class="flex items-center justify-between">
        <label for="hero-assistant-input" class="text-sm font-semibold text-foreground flex items-center gap-2">
          <span>What are you trying to accomplish?</span>
          <span class="text-[10px] text-muted-foreground font-normal">(Explain in plain English; assistant handles the mechanics)</span>
        </label>
        <button
          onclick={() => goto("/ai")}
          class="text-xs text-primary hover:underline font-medium flex items-center gap-1"
        >
          Open Full AI Studio →
        </button>
      </div>

      <div class="flex gap-2">
        <input
          id="hero-assistant-input"
          type="text"
          bind:value={heroQuery}
          onkeydown={(e) => e.key === "Enter" && handleHeroSubmit()}
          placeholder="e.g. I need a 30-day extension, or I have a folder of court orders to organize, or I need eviction defense forms..."
          class="flex-1 px-4 py-3 text-sm rounded-2xl border border-border bg-background/90 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary shadow-inner"
        />
        <button
          onclick={handleHeroSubmit}
          disabled={!heroQuery.trim()}
          class="px-5 py-3 bg-primary hover:opacity-90 disabled:opacity-50 text-primary-foreground font-semibold text-sm rounded-2xl shadow-md transition-opacity flex items-center gap-1.5 shrink-0"
        >
          <span>Ask Assistant</span>
        </button>
      </div>
    </div>

    <!-- Quick Action Pills -->
    <div class="flex flex-wrap items-center gap-2 pt-1">
      <span class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mr-1">Quick Goals:</span>
      {#each QUICK_GOALS as qg}
        <button
          type="button"
          onclick={() => {
            if (qg.action) qg.action();
            else if (qg.goal) goto(`/ai?goal=${encodeURIComponent(qg.goal)}`);
          }}
          class="text-xs px-3 py-1.5 rounded-full border border-border/80 bg-secondary/70 hover:border-primary/60 hover:bg-card text-foreground transition-all shadow-sm"
        >
          {qg.label}
        </button>
      {/each}
    </div>
  </div>

  <!-- Main Split Hub: Dense, Proportional Viewport Layout -->
  <div class="grid gap-5 lg:grid-cols-12 items-start">
    <!-- Left Column (7 cols): Active Matters & Case Workspaces -->
    <section class="lg:col-span-7 rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-5 space-y-4 shadow-sm">
      <div class="flex items-center justify-between border-b border-border/40 pb-3">
        <div class="flex items-center gap-2.5">
          <span class="w-2.5 h-2.5 rounded-full bg-primary"></span>
          <h2 class="text-sm font-bold uppercase tracking-wider text-foreground">Your Matters</h2>
          <span class="text-xs font-mono px-2 py-0.5 rounded-full bg-secondary text-muted-foreground">
            {cases.length} {cases.length === 1 ? 'matter' : 'matters'}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <button onclick={() => (promptKind = "folder")} class="text-xs text-muted-foreground hover:text-foreground transition-colors">
            Browse folder
          </button>
          <span class="text-muted-foreground text-xs">·</span>
          <button onclick={trySample} class="text-xs text-muted-foreground hover:text-foreground transition-colors">
            Sample case
          </button>
          <button
            onclick={() => startNew("reconsideration")}
            class="text-xs font-semibold bg-primary/10 text-primary hover:bg-primary/20 border border-primary/30 px-3 py-1 rounded-xl transition-colors ml-1"
          >
            + New Case
          </button>
        </div>
      </div>

      {#if loadError}
        <p class="text-xs text-destructive">{loadError}</p>
      {:else if cases.length === 0}
        <div class="p-8 text-center border border-dashed border-border/80 rounded-2xl space-y-3 bg-background/30">
          <p class="text-sm text-foreground font-medium">No legal cases detected yet.</p>
          <p class="text-xs text-muted-foreground max-w-sm mx-auto">
            Start a guided case setup, or point Case Rescue at a folder of court papers to organize your docket automatically.
          </p>
          <div class="flex justify-center gap-2.5 pt-2">
            <button onclick={() => startNew("reconsideration")} class="text-xs font-semibold bg-primary text-primary-foreground px-4 py-2 rounded-xl shadow-sm hover:opacity-90 transition-opacity">
              Start Guided Intake
            </button>
            <button onclick={() => (promptKind = "bootstrap")} class="text-xs font-semibold border border-amber-500/40 text-amber-300 hover:bg-amber-500/10 px-4 py-2 rounded-xl transition-colors">
              Rescue Court Folder
            </button>
          </div>
        </div>
      {:else}
        <div class="space-y-2.5 max-h-[460px] overflow-y-auto pr-1">
          {#each cases as item}
            {@const urgency = getMatterUrgency(item.profile.id)}
            <button
              onclick={() => goto(`/case/${item.profile.id}`)}
              class="w-full text-left p-4 rounded-xl border border-border/70 bg-background/60 hover:border-primary/60 hover:bg-background/90 transition-all space-y-2 group shadow-sm"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="space-y-0.5 truncate">
                  <h4 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors truncate">
                    {item.profile.title || "Untitled Matter"}
                  </h4>
                  <p class="text-xs text-muted-foreground truncate font-mono">
                    {item.profile.court.name || "Court unassigned"} {item.profile.docket_number ? `· Docket #${item.profile.docket_number}` : ""}
                  </p>
                </div>
                <div class="flex items-center gap-1.5 shrink-0">
                  <span class="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full border border-border/60 bg-secondary/80 text-muted-foreground">
                    {item.profile.document_kind.replaceAll("_", " ")}
                  </span>
                  {#if item.sealed}
                    <span class="text-[10px] font-mono px-2 py-0.5 rounded-full border border-red-500/30 bg-red-500/10 text-red-300">
                      Sealed
                    </span>
                  {/if}
                </div>
              </div>

              {#if urgency && urgency.urgent_deadline}
                <div class="flex items-center justify-between text-xs pt-1 border-t border-border/30">
                  <span class="text-muted-foreground text-[11px] truncate">
                    Next: <strong class="text-foreground">{urgency.urgent_branch_title}</strong>
                  </span>
                  <span class="text-[11px] font-mono font-semibold px-2 py-0.5 rounded-full {urgency.days_until_due != null && urgency.days_until_due <= 7 ? 'bg-red-500/20 text-red-300 border border-red-500/30' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'}">
                    {urgency.days_until_due != null ? `${urgency.days_until_due}d remaining` : urgency.urgent_deadline}
                  </span>
                </div>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Right Column (5 cols): Case Starters & Compact Resource Hub -->
    <div class="lg:col-span-5 space-y-4">
      <!-- Fast Starters Ribbon -->
      <section class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-4 space-y-2.5 shadow-sm">
        <h3 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">Quick Setup</h3>
        <div class="grid gap-2">
          <button
            onclick={() => startNew("reconsideration")}
            class="text-left p-3 rounded-xl border border-border/60 bg-background/50 hover:border-primary/50 hover:bg-background transition-all flex items-center justify-between group"
          >
            <div>
              <h4 class="text-xs font-semibold text-foreground group-hover:text-primary transition-colors">Start a New Case</h4>
              <p class="text-[11px] text-muted-foreground">Guided intake wizard or custom matter</p>
            </div>
            <span class="text-xs text-muted-foreground group-hover:text-primary transition-colors">→</span>
          </button>

          <button
            onclick={() => (promptKind = "bootstrap")}
            class="text-left p-3 rounded-xl border border-amber-500/30 bg-amber-950/10 hover:border-amber-500/60 transition-all flex items-center justify-between group"
          >
            <div>
              <h4 class="text-xs font-semibold text-amber-200">Case Rescue</h4>
              <p class="text-[11px] text-amber-300/70">Drop a messy folder to auto-organize dockets</p>
            </div>
            <span class="text-xs text-amber-300">→</span>
          </button>

          <button
            onclick={() => (showCloudModal = true)}
            class="text-left p-3 rounded-xl border border-border/60 bg-background/50 hover:border-primary/50 hover:bg-background transition-all flex items-center justify-between group"
          >
            <div>
              <h4 class="text-xs font-semibold text-foreground group-hover:text-primary transition-colors">Continue from JustLegal.me</h4>
              <p class="text-[11px] text-muted-foreground">Sync cloud workspace & verified forms</p>
            </div>
            <span class="text-xs text-muted-foreground group-hover:text-primary transition-colors">→</span>
          </button>
        </div>
      </section>

      <!-- Compact Legal Resource & Forms Navigator -->
      <section class="rounded-2xl border border-border/70 bg-card/60 backdrop-blur-xl p-4 space-y-3 shadow-sm">
        <div class="flex items-center justify-between">
          <h3 class="text-xs font-bold uppercase tracking-wider text-muted-foreground">Forms & Resources</h3>
          <button onclick={() => goto("/forms")} class="text-[11px] text-primary hover:underline font-medium">
            Browse All (30+) →
          </button>
        </div>
        <div class="grid grid-cols-2 gap-2">
          {#each RESOURCE_CATEGORIES as cat}
            <button
              onclick={() => goto(cat.id === "all" ? "/forms" : `/forms?category=${cat.id}`)}
              class="text-left p-2.5 rounded-xl border border-border/50 bg-background/50 hover:border-primary/50 hover:bg-background transition-all group space-y-0.5"
            >
              <div class="text-sm">{cat.icon}</div>
              <h4 class="text-xs font-semibold text-foreground group-hover:text-primary transition-colors truncate">{cat.label}</h4>
              <p class="text-[10px] text-muted-foreground line-clamp-1">{cat.desc}</p>
            </button>
          {/each}
        </div>
      </section>
    </div>
  </div>

  <!-- Bottom: Discreet Review & Legal Notice -->
  <footer class="pt-2 text-center">
    <p class="text-[11px] text-muted-foreground">
      OpenSeason / JustLegal OS formats user-authored material into court-compliant files. It does not provide legal advice and never files or serves documents without human review.
    </p>
  </footer>

  <!-- Modals -->
  <CloudSyncModal bind:open={showCloudModal} onclose={() => (showCloudModal = false)} onsynced={() => { load(); checkCloud(); }} />
  <PdfToolsModal bind:open={showPdfToolsModal} onclose={() => (showPdfToolsModal = false)} />
  <SignatureStudioModal bind:open={showSignatureModal} onclose={() => (showSignatureModal = false)} />

  <PromptDialog
    open={promptKind !== null}
    title={promptKind === "osb"
      ? "Path to .osb file"
      : promptKind === "import"
        ? "Folder of drafts, notes, and orders"
        : promptKind === "bootstrap"
          ? "Case Rescue: Folder of court papers & downloads"
          : "Path to a case folder"}
    body={promptKind === "bootstrap"
      ? "Select or paste the path to your folder of court PDFs, orders, and filings. Files are copied into a clean case workspace organized into orders, filings, evidence, and drafts without altering your originals."
      : promptKind === "import"
        ? "Files are copied. Originals stay put. Proposed names and dates are not written until you confirm them."
        : runningInTauri()
          ? "Paste the full path."
          : "Browser preview: press OK to use the sample path."}
    placeholder={promptKind === "osb" ? "/path/to/bundle.osb" : "/path/to/folder"}
    onsubmit={handlePrompt}
    oncancel={() => (promptKind = null)}
  />
</main>
