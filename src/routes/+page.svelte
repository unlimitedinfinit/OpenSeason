<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import GettingStarted from "$lib/components/GettingStarted.svelte";
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import PdfToolsModal from "$lib/components/PdfToolsModal.svelte";
  import SignatureStudioModal from "$lib/components/SignatureStudioModal.svelte";
  import PortfolioRadarCard from "$lib/components/PortfolioRadarCard.svelte";
  import PromptDialog from "$lib/components/PromptDialog.svelte";
  import { ipc, runningInTauri } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type { WorkspaceCase, CloudSession, FolderImportReport } from "$lib/types/case";

  let cases = $state<WorkspaceCase[]>([]);
  let casesRoot = $state("");
  let loadError = $state("");
  let actionError = $state("");
  let status = $state("");
  let showCloudModal = $state(false);
  let showPdfToolsModal = $state(false);
  let showSignatureModal = $state(false);
  let cloudSession = $state<CloudSession | null>(null);
  let promptKind = $state<"folder" | "osb" | "import" | null>(null);

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

  function startNew(kind: string) {
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

<main class="container mx-auto p-8 space-y-8 max-w-5xl">
  <div class="flex justify-between items-start gap-6">
    <div class="space-y-2">
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Offline document builder</p>
      <h1 class="text-3xl font-bold tracking-tight">OpenSeason</h1>
      <p class="text-muted-foreground max-w-2xl">
        You write the words. The app formats the paper. It never files or serves. Review every line before you sign.
      </p>
    </div>
    <div class="flex flex-wrap justify-end gap-2 shrink-0">
      <button onclick={() => (showPdfToolsModal = true)} class="text-xs border border-border px-3 py-1.5 rounded-md" aria-label="PDF tools">PDF tools</button>
      <button onclick={() => (showSignatureModal = true)} class="text-xs border border-border px-3 py-1.5 rounded-md" aria-label="Signature stamp">Signature</button>
      <button onclick={() => (showCloudModal = true)} class="text-xs border border-border px-3 py-1.5 rounded-md" aria-label="JustLegal Cloud (not implemented for this build)">
        {cloudSession ? `Account: ${cloudSession.email.split("@")[0]}` : "Account (optional)"}
      </button>
      <button onclick={() => goto("/confidential")} class="text-xs border border-red-500/40 text-red-200 px-3 py-1.5 rounded-md">
        {session.unlocked ? "Confidential tools" : "Unlock Confidential"}
      </button>
    </div>
  </div>

  <ReviewNotice />
  <GettingStarted />
  <PortfolioRadarCard onCreated={load} />

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Start</h2>
    <div class="flex flex-wrap gap-2">
      <button onclick={() => goto("/draft")} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm" aria-label="Turn a Markdown draft into a PDF">Turn a draft into a PDF</button>
      <button onclick={() => startNew("reconsideration")} class="border border-border px-4 py-2 rounded-md text-sm">Start a motion</button>
      <button onclick={trySample} class="border border-border px-4 py-2 rounded-md text-sm">Try a sample (Lucille Example)</button>
      <button onclick={() => (promptKind = "folder")} class="border border-border px-4 py-2 rounded-md text-sm">Open a case folder</button>
      <button onclick={() => (promptKind = "import")} class="border border-border px-4 py-2 rounded-md text-sm">Import a folder of papers</button>
      <button onclick={() => (promptKind = "osb")} class="border border-border px-4 py-2 rounded-md text-sm">Import .osb</button>
    </div>
    {#if actionError}<p class="text-sm text-destructive">{actionError}</p>{/if}
    {#if status}<p class="text-sm text-muted-foreground">{status}</p>{/if}
  </section>

  <section class="space-y-3">
    <div class="flex justify-between items-end">
      <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Cases on this computer</h2>
      {#if casesRoot}<p class="text-[11px] font-mono text-muted-foreground max-w-md text-right">{casesRoot}</p>{/if}
    </div>
    {#if loadError}
      <p class="text-sm text-destructive">Could not load cases: {loadError}</p>
    {:else if cases.length === 0}
      <p class="text-sm text-muted-foreground border border-dashed border-border rounded-md p-6">
        No cases yet. Start a motion, open a folder that already has case.json, or import drafts and orders.
      </p>
    {:else}
      <div class="grid gap-4 md:grid-cols-2">
        {#each cases as item}
          <button
            onclick={() => goto(`/case/${item.profile.id}`)}
            class="text-left p-4 rounded-lg border border-border bg-card hover:border-indigo-500/40 transition-colors space-y-2"
          >
            <div class="flex justify-between gap-2">
              <h3 class="font-semibold">{item.profile.title || "Untitled case"}</h3>
              <span class="text-[10px] uppercase tracking-wide px-2 py-0.5 rounded border {item.profile.mode === 'confidential' ? 'border-red-500/40 text-red-300' : 'border-border text-muted-foreground'}">
                {item.profile.mode}{item.sealed ? " · sealed" : ""}
              </span>
            </div>
            <p class="text-xs text-muted-foreground">
              {item.profile.court.name || "Court not filled"} · {item.profile.docket_number || "No docket"}
            </p>
            <p class="text-[11px] text-muted-foreground">
              {item.profile.document_kind.replaceAll("_", " ") || "no document yet"}
              {#if item.has_vault} · vault{/if}
            </p>
            <p class="text-[10px] text-muted-foreground">
              {item.profile.mode === "confidential" || item.sealed
                ? "Airgapped. Sync is refused."
                : "Local only. Account backup is not implemented in this build."}
            </p>
          </button>
        {/each}
      </div>
    {/if}
  </section>

  <CloudSyncModal bind:open={showCloudModal} onclose={() => (showCloudModal = false)} onsynced={() => { load(); checkCloud(); }} />
  <PdfToolsModal bind:open={showPdfToolsModal} onclose={() => (showPdfToolsModal = false)} />
  <SignatureStudioModal bind:open={showSignatureModal} onclose={() => (showSignatureModal = false)} />

  <PromptDialog
    open={promptKind !== null}
    title={promptKind === "osb" ? "Path to .osb file" : promptKind === "import" ? "Folder of drafts, notes, and orders" : "Path to a case folder"}
    body={promptKind === "import"
      ? "Files are copied. Originals stay put. Proposed names and dates are not written until you confirm them."
      : runningInTauri()
        ? "Paste the full path."
        : "Browser preview: press OK to use the sample path."}
    placeholder={promptKind === "osb" ? "/path/to/bundle.osb" : "/path/to/folder"}
    onsubmit={handlePrompt}
    oncancel={() => (promptKind = null)}
  />
</main>
