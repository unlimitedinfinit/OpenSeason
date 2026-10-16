<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import { ipc } from "$lib/ipc";
  import type { InstallReport, PackList } from "$lib/types/packs";

  let list = $state<PackList | null>(null);
  let error = $state("");
  let status = $state("");
  let filePath = $state("");

  onMount(load);

  async function load() {
    try {
      list = await ipc<PackList>("list_form_packs_cmd");
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function install(state: string, situation?: string) {
    error = "";
    status = "";
    try {
      const report = await ipc<InstallReport>("install_form_pack_cmd", {
        state,
        situation: situation || null,
        file: filePath || null,
      });
      status = `Installed ${report.pack_id} (${report.size_label}) to ${report.dest}. Offline afterward.`;
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function remove(state: string) {
    error = "";
    try {
      await ipc<string>("remove_form_pack_cmd", { state });
      status = `Removed ${state}.`;
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function verify(state?: string) {
    error = "";
    try {
      const reports = await ipc<Array<{ pack_id: string; ok: boolean }>>("verify_form_pack_cmd", {
        state: state || null,
      });
      status = reports.map((r) => `${r.pack_id}: ${r.ok ? "hashes match" : "hash failed"}`).join(" · ");
    } catch (e) {
      error = String(e);
    }
  }
</script>

<main class="container mx-auto p-8 space-y-6 max-w-4xl">
  <div class="flex justify-between gap-4">
    <div>
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Add-on packs</p>
      <h1 class="text-3xl font-bold">Forms packs</h1>
      <p class="text-muted-foreground mt-2">
        Official court PDFs are not stored in this repository. The catalog lists size and SHA-256 first. Install to this computer, then work offline. Never files or serves.
      </p>
    </div>
    <button onclick={() => goto("/")} class="text-sm border border-border px-3 py-1.5 rounded-md h-fit">Home</button>
  </div>

  <ReviewNotice />

  {#if list}
    <p class="text-xs font-mono text-muted-foreground">{list.packs_root}</p>
    <p class="text-sm text-muted-foreground">{list.notice}</p>
  {/if}

  <label class="block space-y-1">
    <span class="text-sm font-medium">Optional local zip or pack folder</span>
    <input bind:value={filePath} class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm font-mono" placeholder="/path/to/nj-appeal.zip" />
  </label>

  {#if error}
    <p class="text-sm text-destructive">{error}</p>
  {/if}
  {#if status}
    <p class="text-sm text-emerald-300">{status}</p>
  {/if}

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Bundled rule packs (teaching + deadlines)</h2>
    <div class="grid gap-3 md:grid-cols-2">
      {#each list?.rule_packs || [] as pack}
        <div class="border border-border rounded-lg p-3">
          <p class="font-medium">{pack.name}</p>
          <p class="text-xs text-muted-foreground">{pack.id} · {pack.jurisdiction} · {pack.forms} forms · {pack.situations} situations</p>
        </div>
      {/each}
    </div>
  </section>

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Available form packs</h2>
    {#if !list?.available.length}
      <p class="text-sm text-muted-foreground">No remote packs are bundled. Point the catalog at JustLegal's registry or a zip you host.</p>
    {/if}
    {#each list?.available || [] as pack}
      <article class="border border-border rounded-lg p-4 space-y-2">
        <div class="flex justify-between gap-3">
          <div>
            <h3 class="font-semibold">{pack.title}</h3>
            <p class="text-xs text-muted-foreground">{pack.state} · {pack.court}</p>
            <p class="text-xs font-mono">Download size: {pack.size_label}{pack.sha256 ? ` · sha256 ${pack.sha256.slice(0, 12)}…` : ""}</p>
            {#if pack.installable === false && pack.unavailable_reason}
              <p class="text-sm text-amber-200 mt-2">Unavailable: {pack.unavailable_reason}</p>
            {/if}
          </div>
          <div class="flex flex-col gap-2">
            {#if pack.installable === false}
              <span class="text-xs border border-amber-500/40 text-amber-200 px-3 py-1.5 rounded-md">Cannot install</span>
            {:else}
              <button onclick={() => install(pack.state, pack.situations[0])} class="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-sm">
                Install {pack.state}
              </button>
            {/if}
            {#if pack.installed}
              <button onclick={() => remove(pack.state)} class="border border-border px-3 py-1.5 rounded-md text-sm">Remove</button>
              <button onclick={() => verify(pack.state)} class="border border-border px-3 py-1.5 rounded-md text-sm">Verify hashes</button>
            {/if}
          </div>
        </div>
      </article>
    {/each}
  </section>

  <section class="space-y-3">
    <h2 class="text-sm font-semibold uppercase tracking-wide text-muted-foreground">Installed on this computer</h2>
    {#if !list?.installed.length}
      <p class="text-sm text-muted-foreground">Nothing installed yet. Teaching still works from the bundled rule packs.</p>
    {/if}
    {#each list?.installed || [] as pack}
      <div class="border border-border rounded-md p-3 text-sm">
        {pack.title} · {pack.size_label} · {pack.install_path}
      </div>
    {/each}
  </section>
</main>
