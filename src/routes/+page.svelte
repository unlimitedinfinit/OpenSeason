<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import GlassOSPanel from "$lib/components/GlassOSPanel.svelte";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import GettingStarted from "$lib/components/GettingStarted.svelte";
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import { ipc, runningInTauri } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type { WorkspaceCase, CloudSession } from "$lib/types/case";

  let cases = $state<WorkspaceCase[]>([]);
  let casesRoot = $state("");
  let loadError = $state("");
  let actionError = $state("");
  let status = $state("");
  let showCloudModal = $state(false);
  let cloudSession = $state<CloudSession | null>(null);

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

  async function openFolder() {
    actionError = "";
    const path = prompt(
      runningInTauri()
        ? "Paste the full path to a case folder (the folder that contains case.json)."
        : "Browser preview: press OK to load the sample case folder.",
    );
    if (path === null) return;
    try {
      const profile = await ipc<{ id: string }>("open_case_folder", { path: path || "." });
      goto(`/case/${profile.id}`);
    } catch (e) {
      actionError = String(e);
    }
  }

  async function importOsb() {
    actionError = "";
    if (!session.unlocked) {
      actionError = "Unlock Confidential mode first. Import reads a vault bundle (.osb).";
      return;
    }
    const path = prompt(
      runningInTauri()
        ? "Paste the full path to an .osb file."
        : "Browser preview: press OK to import a sample confidential case.",
    );
    if (path === null) return;
    try {
      const id = await ipc<string>("import_hunt_cmd", { osbPath: path || "sample.osb" });
      status = "Imported the confidential bundle.";
      await load();
      goto(`/case/${id}`);
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

<main class="container mx-auto p-8 space-y-8 max-w-6xl">
  <div class="flex justify-between items-start gap-6 border-b pb-6">
    <div class="space-y-2">
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Offline Case Builder</p>
      <h1 class="text-3xl font-bold tracking-tight">Document Builder</h1>
      <p class="text-muted-foreground max-w-2xl">
        Draft, check, and produce court-shaped motions on this computer. You write the words. The app formats the paper. It never files or serves.
        Confidential (encrypted False Claims vault) is one case type, not the whole product.
      </p>
    </div>
    <div class="text-right space-y-2 shrink-0">
      <div class="flex items-center justify-end gap-2">
        <button
          onclick={() => showCloudModal = true}
          class="text-xs border {cloudSession ? 'border-emerald-500/40 text-emerald-300 hover:bg-emerald-950/30' : 'border-indigo-500/40 text-indigo-300 hover:bg-indigo-950/30'} px-3 py-1.5 rounded-md flex items-center gap-1.5 transition-colors"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 00-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          <span>{cloudSession ? `Cloud: ${cloudSession.email.split('@')[0]}` : "JustLegal Cloud"}</span>
        </button>
        <button onclick={() => goto("/confidential")} class="text-xs border border-red-500/40 text-red-200 px-3 py-1.5 rounded-md">
          {session.unlocked ? "Confidential tools" : "Unlock Confidential"}
        </button>
      </div>
      {#if session.unlocked}
        <p class="text-[11px] text-emerald-400">Confidential vault unlocked</p>
      {:else}
        <p class="text-[11px] text-muted-foreground">Vault locked</p>
      {/if}
    </div>
  </div>

  <ReviewNotice />
  <GettingStarted />

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">What do you need?</h2>
    <div class="grid gap-2 md:grid-cols-2">
      <button onclick={() => goto("/guide")} class="text-left border border-indigo-500/40 rounded-md p-3 hover:border-indigo-400">
        <p class="font-medium">What do I need to file?</p>
        <p class="text-xs text-muted-foreground">Answer a few plain-language questions. The pack lists the forms and explains each step. Not legal advice.</p>
      </button>
      <button onclick={() => goto("/packs")} class="text-left border border-border rounded-md p-3 hover:border-indigo-500/40">
        <p class="font-medium">Install a forms pack</p>
        <p class="text-xs text-muted-foreground">Download official form files to this computer. Sizes and SHA-256 shown first. Then work offline.</p>
      </button>
      <button onclick={() => startNew("reconsideration")} class="text-left border border-border rounded-md p-3 hover:border-indigo-500/40">
        <p class="font-medium">I lost and want the court to look again</p>
        <p class="text-xs text-muted-foreground">Motion for reconsideration. Guided prompts. NJ deadline table included.</p>
      </button>
      <button onclick={() => startNew("enlargement")} class="text-left border border-border rounded-md p-3 hover:border-indigo-500/40">
        <p class="font-medium">I need more time</p>
        <p class="text-xs text-muted-foreground">Motion for enlargement of time. You write why.</p>
      </button>
      <button onclick={() => startNew("stay")} class="text-left border border-border rounded-md p-3 hover:border-indigo-500/40">
        <p class="font-medium">I need the order paused</p>
        <p class="text-xs text-muted-foreground">Motion for a stay. You write the terms.</p>
      </button>
      <button onclick={() => startNew("notice_of_appeal")} class="text-left border border-border rounded-md p-3 hover:border-indigo-500/40">
        <p class="font-medium">I am appealing</p>
        <p class="text-xs text-muted-foreground">Notice of Appeal is ready. Brief and appellate motion are scaffolds.</p>
      </button>
    </div>
  </section>

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Start</h2>
    <div class="flex flex-wrap gap-2">
      <button onclick={() => startNew("reconsideration")} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm">Start a motion</button>
      <button onclick={trySample} class="border border-indigo-500/40 text-indigo-200 px-4 py-2 rounded-md text-sm">Try a sample (Lucille Example)</button>
      <button onclick={openFolder} class="border border-border px-4 py-2 rounded-md text-sm">Open a case folder</button>
      <button onclick={importOsb} class="border border-border px-4 py-2 rounded-md text-sm">Import .osb</button>
      <button onclick={() => startNew("notice_of_appeal")} class="border border-border px-4 py-2 rounded-md text-sm">Notice of Appeal</button>
      <button onclick={() => startNew("appellate_brief")} class="border border-border px-4 py-2 rounded-md text-sm">Appellate brief (basic)</button>
      <button onclick={() => startNew("amend_findings")} class="border border-border px-4 py-2 rounded-md text-sm">Amend findings</button>
      <button onclick={() => startNew("mandamus")} class="border border-border px-4 py-2 rounded-md text-sm">Mandamus (CO, basic)</button>
      <button onclick={() => startNew("complaint")} class="border border-border px-4 py-2 rounded-md text-sm">Complaint (basic)</button>
    </div>
    {#if actionError}
      <p class="text-sm text-destructive">{actionError}</p>
    {/if}
    {#if status}
      <p class="text-sm text-emerald-300">{status}</p>
    {/if}
  </section>

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">What do you want to build?</h2>
    <div class="grid gap-6 md:grid-cols-2">
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("notice_of_appeal")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-indigo-400">Ready</p>
          <h3 class="text-xl font-semibold">Notice of Appeal</h3>
          <p class="text-sm text-muted-foreground">
            Guided case setup, then export Word and PDF from the case profile and your own notice text.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("reconsideration")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic template · NJ pack</p>
          <h3 class="text-xl font-semibold">Motions</h3>
          <p class="text-sm text-muted-foreground">
            Reconsideration, enlargement of time, stay, or amend findings. The NJ Appellate Division pack computes deadlines. You write the grounds.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("appellate_brief")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic scaffold</p>
          <h3 class="text-xl font-semibold">Appeals</h3>
          <p class="text-sm text-muted-foreground">
            Notice of Appeal is ready. Appellate brief and motion-in-the-appellate-court are scaffolds. Check stage runs on the same draft.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("mandamus")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic template · CO</p>
          <h3 class="text-xl font-semibold">Mandamus</h3>
          <p class="text-sm text-muted-foreground">
            C.A.R. 21 original-proceeding scaffold. The Mandamus-Colorado repo was not readable here, so this is a public-rule pack, not the original wizard dump.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("complaint")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic template</p>
          <h3 class="text-xl font-semibold">Complaint</h3>
          <p class="text-sm text-muted-foreground">
            Caption, parties, and signature from the profile, plus your own claim text. Not a full interview.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => goto("/confidential")} class="w-full h-full p-6 text-left space-y-2 hover:bg-red-500/10 transition-colors">
          <p class="text-xs font-mono text-red-400">Encrypted vault</p>
          <h3 class="text-xl font-semibold">Confidential</h3>
          <p class="text-sm text-muted-foreground">
            Legal Airlock and vault password. Existing hunts appear here as Confidential cases. USAspending lookup and the disclosure statement are tools inside those cases.
          </p>
        </button>
      </GlassOSPanel>
    </div>
  </section>

  <section class="space-y-3">
    <div class="flex justify-between items-end">
      <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Cases on this computer</h2>
      {#if casesRoot}
        <p class="text-[11px] font-mono text-muted-foreground max-w-md text-right">{casesRoot}</p>
      {/if}
    </div>
    {#if loadError}
      <p class="text-sm text-destructive">Could not load cases: {loadError}</p>
    {:else if cases.length === 0}
      <p class="text-sm text-muted-foreground border border-dashed border-border rounded-md p-6">
        No cases yet. Create a case, open a folder that already has case.json, or unlock Confidential and import an .osb bundle.
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
            <div class="flex items-center gap-2 pt-2 border-t border-border/40">
              {#if item.profile.mode === 'confidential' || item.sealed}
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 flex items-center gap-1">
                  🔒 Airgapped (Strictly Local)
                </span>
              {:else if item.profile.cloud_matter_id}
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-indigo-950/60 text-indigo-300 border border-indigo-500/30 flex items-center gap-1">
                  ☁ Cloud Synced: {item.profile.cloud_matter_id.slice(0, 10)}...
                </span>
              {:else}
                <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-800/40 text-zinc-400 border border-zinc-700/40">
                  Local Matter (Ready to Sync)
                </span>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </section>

  <CloudSyncModal
    bind:open={showCloudModal}
    onclose={() => showCloudModal = false}
    onsynced={() => { load(); checkCloud(); }}
  />
</main>
