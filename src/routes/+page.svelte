<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import GlassOSPanel from "$lib/components/GlassOSPanel.svelte";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import type { CaseProfile } from "$lib/types/case";

  let cases = $state<CaseProfile[]>([]);
  let casesRoot = $state<string>("");
  let loadError = $state("");

  onMount(async () => {
    try {
      casesRoot = await invoke<string>("get_cases_root");
      cases = await invoke<CaseProfile[]>("list_cases");
    } catch (e) {
      loadError = String(e);
    }
  });

  function startAppeal() {
    goto("/case/new?kind=notice_of_appeal");
  }
</script>

<main class="container mx-auto p-8 space-y-8 max-w-6xl">
  <div class="flex justify-between items-start gap-6 border-b pb-6">
    <div class="space-y-2">
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Local desktop builder</p>
      <h1 class="text-3xl font-bold tracking-tight">OpenSeason</h1>
      <p class="text-muted-foreground max-w-2xl">
        A free, privacy-first document formatter for people representing themselves.
        Fill a case profile once. The builder puts caption, court, docket, parties, and your signature block into the document.
        You bring the research. The software only formats it.
      </p>
    </div>
  </div>

  <ReviewNotice />

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">What do you want to build?</h2>
    <div class="grid gap-6 md:grid-cols-2">
      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button
          onclick={startAppeal}
          class="w-full h-full p-6 text-left space-y-2 hover:bg-accent/10 transition-colors"
        >
          <p class="text-xs font-mono text-indigo-400">Ready</p>
          <h3 class="text-xl font-semibold">Notice of Appeal</h3>
          <p class="text-sm text-muted-foreground">
            Create a standard case folder, fill the profile, paste your own notice text, and export Word and PDF.
          </p>
        </button>
      </GlassOSPanel>

      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <div class="p-6 space-y-2 opacity-80">
          <p class="text-xs font-mono text-muted-foreground">Later release</p>
          <h3 class="text-xl font-semibold">Complaint</h3>
          <p class="text-sm text-muted-foreground">
            Intake interview and complaint assembly are planned. You can still start a case profile today from Notice of Appeal.
          </p>
        </div>
      </GlassOSPanel>

      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <div class="p-6 space-y-2 opacity-80">
          <p class="text-xs font-mono text-muted-foreground">Later release</p>
          <h3 class="text-xl font-semibold">Motion</h3>
          <p class="text-sm text-muted-foreground">
            Motion captions will reuse the same fill-once profile. The motion builder is not in this release.
          </p>
        </div>
      </GlassOSPanel>

      <GlassOSPanel borderRadius={8} scale={12} depth={15} enableSpecular={true}>
        <button
          onclick={() => goto("/confidential")}
          class="w-full h-full p-6 text-left space-y-2 hover:bg-red-500/10 transition-colors"
        >
          <p class="text-xs font-mono text-red-400">Encrypted vault</p>
          <h3 class="text-xl font-semibold">Confidential (Open Season)</h3>
          <p class="text-sm text-muted-foreground">
            For sealed whistleblower work. Shows the Legal Airlock, unlocks the vault, and never sends the case through sync, cloud backup, or publish.
          </p>
        </button>
      </GlassOSPanel>
    </div>
  </section>

  <section class="space-y-3">
    <div class="flex justify-between items-end">
      <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Standard cases on this computer</h2>
      {#if casesRoot}
        <p class="text-[11px] font-mono text-muted-foreground max-w-md text-right">{casesRoot}</p>
      {/if}
    </div>
    {#if loadError}
      <p class="text-sm text-muted-foreground">
        Case list is available when the desktop backend is running. You can still use the command-line tool described in AGENTS.md.
      </p>
    {:else if cases.length === 0}
      <p class="text-sm text-muted-foreground">No standard cases yet. Start a Notice of Appeal to create a portable folder.</p>
    {:else}
      <div class="grid gap-4 md:grid-cols-2">
        {#each cases as item}
          <button
            onclick={() => goto(`/case/${item.id}`)}
            class="text-left p-4 rounded-lg border border-border bg-card hover:border-indigo-500/40 transition-colors"
          >
            <h3 class="font-semibold">{item.title}</h3>
            <p class="text-xs text-muted-foreground mt-1">
              {item.court.name || "Court not filled"} · {item.docket_number || "No docket"}
            </p>
          </button>
        {/each}
      </div>
    {/if}
  </section>
</main>
