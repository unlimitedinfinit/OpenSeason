<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import GlassOSPanel from "$lib/components/GlassOSPanel.svelte";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import { ipc, runningInTauri } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type { WorkspaceCase } from "$lib/types/case";

  let cases = $state<WorkspaceCase[]>([]);
  let casesRoot = $state("");
  let loadError = $state("");
  let actionError = $state("");
  let status = $state("");

  onMount(load);

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
</script>

<main class="container mx-auto p-8 space-y-8 max-w-6xl">
  <div class="flex justify-between items-start gap-6 border-b pb-6">
    <div class="space-y-2">
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Local desktop builder</p>
      <h1 class="text-3xl font-bold tracking-tight">OpenSeason</h1>
      <p class="text-muted-foreground max-w-2xl">
        One case list. Standard cases live in a folder you can copy. Confidential cases use the Legal Airlock and a local vault.
        Fill the profile once. The builder puts caption, court, docket, parties, and your signature block into the document.
        You bring the research. The software only formats it.
      </p>
    </div>
    <div class="text-right space-y-2 shrink-0">
      {#if session.unlocked}
        <p class="text-xs text-emerald-400">Confidential vault unlocked</p>
      {:else}
        <p class="text-xs text-muted-foreground">Vault locked</p>
      {/if}
      <button onclick={() => goto("/confidential")} class="text-sm border border-red-500/40 text-red-200 px-3 py-1.5 rounded-md">
        {session.unlocked ? "Confidential tools" : "Unlock Confidential"}
      </button>
    </div>
  </div>

  <ReviewNotice />

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Start</h2>
    <div class="flex flex-wrap gap-2">
      <button onclick={() => startNew("notice_of_appeal")} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm">Create a case</button>
      <button onclick={openFolder} class="border border-border px-4 py-2 rounded-md text-sm">Open a case folder</button>
      <button onclick={importOsb} class="border border-border px-4 py-2 rounded-md text-sm">Import .osb</button>
      <button onclick={() => startNew("notice_of_appeal")} class="border border-indigo-500/40 text-indigo-200 px-4 py-2 rounded-md text-sm">Notice of Appeal</button>
      <button onclick={() => startNew("complaint")} class="border border-border px-4 py-2 rounded-md text-sm">Complaint (basic)</button>
      <button onclick={() => startNew("motion")} class="border border-border px-4 py-2 rounded-md text-sm">Motion (basic)</button>
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
        <button onclick={() => startNew("complaint")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic template</p>
          <h3 class="text-xl font-semibold">Complaint</h3>
          <p class="text-sm text-muted-foreground">
            A working basic complaint: caption, parties, and signature from the profile, plus your own claim text. Not a full interview.
          </p>
        </button>
      </GlassOSPanel>
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button onclick={() => startNew("motion")} class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors">
          <p class="text-xs font-mono text-amber-300">Basic template</p>
          <h3 class="text-xl font-semibold">Motion</h3>
          <p class="text-sm text-muted-foreground">
            A working basic motion: same fill-once profile, plus the request you write. Not a motion library.
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
          </button>
        {/each}
      </div>
    {/if}
  </section>
</main>
