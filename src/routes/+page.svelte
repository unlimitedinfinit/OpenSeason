<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import PdfToolsModal from "$lib/components/PdfToolsModal.svelte";
  import SignatureStudioModal from "$lib/components/SignatureStudioModal.svelte";
  import PortfolioRadarCard from "$lib/components/PortfolioRadarCard.svelte";
  import PromptDialog from "$lib/components/PromptDialog.svelte";
  import { ipc, runningInTauri } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type { WorkspaceCase, CloudSession, FolderImportReport, BootstrapResult } from "$lib/types/case";

  let cases = $state<WorkspaceCase[]>([]);
  let casesRoot = $state("");
  let loadError = $state("");
  let actionError = $state("");
  let status = $state("");
  let showCloudModal = $state(false);
  let showPdfToolsModal = $state(false);
  let showSignatureModal = $state(false);
  let cloudSession = $state<CloudSession | null>(null);
  let promptKind = $state<"folder" | "osb" | "import" | "bootstrap" | null>(null);
  let showLocalCases = $state(true);

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
      loadError = "";
    } catch (e) {
      loadError = String(e);
    }
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
      status = "Opened the fake Lucille Example sample. Nothing was filed.";
      await load();
      goto(`/case/${profile.id}`);
    } catch (e) {
      actionError = String(e);
    }
  }
</script>

<main class="container mx-auto p-6 md:p-8 space-y-8 max-w-5xl">
  <!-- Top App Header -->
  <div class="flex flex-col sm:flex-row justify-between items-start gap-4">
    <div class="space-y-1">
      <div class="flex items-center gap-2">
        <h1 class="text-2xl md:text-3xl font-bold tracking-tight text-foreground">JustLegal Desktop</h1>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-primary/20 text-primary border border-primary/30">Local-First</span>
      </div>
      <p class="text-sm text-muted-foreground">
        Your legal workspace · Private and local-first
      </p>
    </div>
    <div class="flex flex-wrap items-center gap-2 shrink-0">
      <button onclick={() => (showPdfToolsModal = true)} class="text-xs border border-border bg-card hover:border-primary/50 text-foreground px-3 py-1.5 rounded-md transition-colors" aria-label="PDF tools">
        PDF tools
      </button>
      <button onclick={() => (showSignatureModal = true)} class="text-xs border border-border bg-card hover:border-primary/50 text-foreground px-3 py-1.5 rounded-md transition-colors" aria-label="Signature stamp">
        Signature
      </button>
      <button onclick={() => (showCloudModal = true)} class="text-xs border border-border bg-card hover:border-primary/50 text-foreground px-3 py-1.5 rounded-md transition-colors" aria-label="JustLegal Cloud">
        {cloudSession ? `Account: ${cloudSession.email.split("@")[0]}` : "Continue from JustLegal.me"}
      </button>
      <button onclick={() => goto("/confidential")} class="text-xs border border-red-500/40 text-red-300 hover:bg-red-500/10 px-3 py-1.5 rounded-md transition-colors">
        {session.unlocked ? "Confidential tools" : "Unlock Confidential"}
      </button>
    </div>
  </div>

  {#if actionError}<p class="text-sm text-destructive bg-destructive/10 p-3 rounded-md border border-destructive/20">{actionError}</p>{/if}
  {#if status}<p class="text-sm text-primary bg-primary/10 p-3 rounded-md border border-primary/20">{status}</p>{/if}

  <!-- Section 1: Your Workspace -->
  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Your workspace</h2>
    <div class="grid gap-3 sm:grid-cols-3">
      <!-- Start a new case -->
      <button
        onclick={() => startNew("reconsideration")}
        class="group text-left p-5 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex items-start gap-4"
      >
        <div class="p-2.5 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
          </svg>
        </div>
        <div>
          <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors">Start a new case</h3>
          <p class="text-xs text-muted-foreground mt-0.5">Guided setup or custom matter</p>
        </div>
      </button>

      <!-- Case Rescue: Already have a case? Let's get organized -->
      <button
        onclick={() => (promptKind = "bootstrap")}
        class="group text-left p-5 rounded-lg border border-amber-500/30 bg-amber-950/10 hover:border-amber-500/60 hover:bg-amber-950/20 transition-all flex items-start gap-4"
      >
        <div class="p-2.5 rounded-md bg-amber-500/20 text-amber-300 group-hover:text-amber-200 transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
          </svg>
        </div>
        <div>
          <h3 class="font-semibold text-amber-200 group-hover:text-amber-100 transition-colors">Case Rescue</h3>
          <p class="text-xs text-amber-300/70 mt-0.5">Already have a folder? Auto-organize orders & filings</p>
        </div>
      </button>

      <!-- Continue from JustLegal.me -->
      <button
        onclick={() => (showCloudModal = true)}
        class="group text-left p-5 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex items-start gap-4"
      >
        <div class="p-2.5 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
          </svg>
        </div>
        <div>
          <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors">Continue from JustLegal.me</h3>
          <p class="text-xs text-muted-foreground mt-0.5">Connect account & choose case to sync</p>
        </div>
      </button>
    </div>

    <!-- Open an existing local case banner -->
    <div class="rounded-lg border border-border bg-card overflow-hidden">
      <button
        onclick={() => (showLocalCases = !showLocalCases)}
        class="w-full text-left p-4 hover:bg-muted/40 transition-colors flex items-center justify-between gap-4"
      >
        <div class="flex items-center gap-3.5">
          <div class="p-2 rounded-md bg-secondary text-foreground">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div>
            <h3 class="text-sm font-semibold text-foreground">Open an existing local case</h3>
            <p class="text-xs text-muted-foreground">Recent matters and case folders on this computer</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground font-mono">{cases.length} {cases.length === 1 ? 'case' : 'cases'}</span>
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-muted-foreground transition-transform {showLocalCases ? 'rotate-90' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </div>
      </button>

      {#if showLocalCases}
        <div class="p-4 pt-0 border-t border-border/50 space-y-3">
          <div class="flex flex-wrap items-center justify-between gap-2 pt-3">
            <span class="text-xs font-mono text-muted-foreground truncate max-w-sm">{casesRoot || "Default cases folder"}</span>
            <div class="flex gap-2">
              <button onclick={() => (promptKind = "folder")} class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50">
                Browse folder
              </button>
              <button onclick={() => (promptKind = "import")} class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50">
                Import papers
              </button>
              <button onclick={trySample} class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50">
                Sample case
              </button>
            </div>
          </div>

          {#if loadError}
            <p class="text-xs text-destructive">Could not load cases: {loadError}</p>
          {:else if cases.length === 0}
            <p class="text-xs text-muted-foreground p-4 text-center border border-dashed border-border rounded-md">
              No local cases yet. Click "Start a new case" above to begin.
            </p>
          {:else}
            <div class="grid gap-2.5 sm:grid-cols-2 pt-1">
              {#each cases as item}
                <button
                  onclick={() => goto(`/case/${item.profile.id}`)}
                  class="text-left p-3.5 rounded-md border border-border bg-background/60 hover:border-primary/50 transition-colors space-y-1.5 group"
                >
                  <div class="flex justify-between items-start gap-2">
                    <h4 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors truncate">
                      {item.profile.title || "Untitled case"}
                    </h4>
                    <span class="text-[9px] uppercase tracking-wide px-1.5 py-0.5 rounded border {item.profile.mode === 'confidential' ? 'border-red-500/40 text-red-300' : 'border-border text-muted-foreground'} shrink-0">
                      {item.profile.mode}{item.sealed ? " · sealed" : ""}
                    </span>
                  </div>
                  <p class="text-xs text-muted-foreground truncate">
                    {item.profile.court.name || "Court not filled"} {item.profile.docket_number ? `· ${item.profile.docket_number}` : ""}
                  </p>
                  <p class="text-[10px] text-muted-foreground">
                    {item.profile.document_kind.replaceAll("_", " ") || "case workspace"}
                  </p>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </section>

  <!-- Section 2: Or find a form, document, or resource -->
  <section class="space-y-3">
    <div>
      <h2 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">Or find a form, document, or resource</h2>
      <p class="text-xs text-muted-foreground mt-0.5">No active lawsuit or case setup required.</p>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
      <!-- Housing -->
      <button
        onclick={() => goto("/forms?category=housing")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Housing</h3>
          <p class="text-xs text-muted-foreground">Leases, eviction defenses, repair demands</p>
        </div>
      </button>

      <!-- Family -->
      <button
        onclick={() => goto("/forms?category=family")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Family</h3>
          <p class="text-xs text-muted-foreground">Custody, support, parenting plans</p>
        </div>
      </button>

      <!-- Wills & Estates -->
      <button
        onclick={() => goto("/forms?category=wills")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Wills & estates</h3>
          <p class="text-xs text-muted-foreground">Wills, powers of attorney, healthcare directives</p>
        </div>
      </button>

      <!-- Contracts -->
      <button
        onclick={() => goto("/forms?category=contracts")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Contracts</h3>
          <p class="text-xs text-muted-foreground">NDAs, contractor agreements, bills of sale</p>
        </div>
      </button>

      <!-- Court forms -->
      <button
        onclick={() => goto("/forms?category=court")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6l3 1m0 0l-3 9a5.002 5.002 0 006.001 0M6 7l3 9M6 7l6-2m6 2l3-1m-3 1l-3 9a5.002 5.002 0 006.001 0M18 7l3 9m-3-9l-6-2m0-2v2m0 16V5m0 16H9m3 0h3" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Court forms</h3>
          <p class="text-xs text-muted-foreground">Fee waivers, summons, notices of appeal</p>
        </div>
      </button>

      <!-- Full library -->
      <button
        onclick={() => goto("/forms")}
        class="group p-4 rounded-lg border border-border bg-card hover:border-primary/60 hover:bg-card/80 transition-all flex flex-col items-start gap-2.5 text-left"
      >
        <div class="p-2 rounded-md bg-secondary text-foreground group-hover:text-primary transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
          </svg>
        </div>
        <div>
          <h3 class="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Full library</h3>
          <p class="text-xs text-muted-foreground">Search and browse all official forms & templates</p>
        </div>
      </button>
    </div>
  </section>

  <!-- Section 3: Need help getting started? (Assistant Launcher) -->
  <section>
    <div class="p-5 rounded-lg border border-border bg-card hover:border-primary/50 transition-colors flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div class="flex items-start gap-3.5">
        <div class="p-2.5 rounded-md bg-secondary text-foreground shrink-0 mt-0.5 sm:mt-0">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        </div>
        <div>
          <h3 class="font-semibold text-foreground">Need help getting started?</h3>
          <p class="text-xs text-muted-foreground mt-0.5">
            Work with your connected assistant to organize a case, find resources, and use JustLegal tools.
          </p>
        </div>
      </div>
      <div class="flex gap-2 shrink-0 w-full sm:w-auto">
        <button onclick={() => goto("/guide")} class="flex-1 sm:flex-none text-xs border border-border px-3.5 py-2 rounded-md hover:border-primary/50 text-foreground transition-colors">
          Intake Guide
        </button>
        <button onclick={() => goto("/ai")} class="flex-1 sm:flex-none text-xs bg-primary text-primary-foreground px-3.5 py-2 rounded-md hover:opacity-90 transition-opacity">
          Open AI CLI & Tools
        </button>
      </div>
    </div>
  </section>

  <!-- Active Litigation Portfolio Radar -->
  <PortfolioRadarCard onCreated={load} />

  <!-- Legal Disclaimer & Attorney Notice -->
  <ReviewNotice />

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
