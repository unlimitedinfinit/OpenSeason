<script lang="ts">
  import { goto } from "$app/navigation";
  import { ipc } from "$lib/ipc";
  import type { Filer, PortfolioCaseItem, PortfolioStatusReport } from "$lib/types/case";
  import NextUpCard from "$lib/components/NextUpCard.svelte";

  let {
    onCreated = () => {},
  }: {
    onCreated?: () => void;
  } = $props();

  let portfolio = $state<PortfolioStatusReport | null>(null);
  let showFilerModal = $state(false);
  let filerDraft = $state<Filer>({
    name: "",
    role: "Proceeding without an attorney",
    address_lines: [""],
    phone: "",
    email: "",
    signature_name: "",
  });
  let addressText = $state("");
  let busy = $state(false);
  let statusMsg = $state("");
  let errorMsg = $state("");

  $effect(() => {
    loadPortfolio();
  });

  export async function loadPortfolio() {
    try {
      const res = await ipc<PortfolioStatusReport>("portfolio_status_cmd");
      portfolio = res;
      if (res.global_filer) {
        filerDraft = { ...res.global_filer };
        addressText = (res.global_filer.address_lines || []).join("\n");
      }
    } catch (e) {
      errorMsg = String(e);
    }
  }

  const nextMatter = $derived.by((): PortfolioCaseItem | null => {
    if (!portfolio?.matters?.length) return null;
    const dated = portfolio.matters
      .filter((m) => m.urgent_deadline)
      .sort((a, b) => (a.days_until_due ?? 99999) - (b.days_until_due ?? 99999));
    if (dated[0]) return dated[0];
    return portfolio.matters.find((m) => m.urgent_status === "needs_trigger_date") || portfolio.matters[0];
  });

  function dueClass(m: PortfolioCaseItem): string {
    if (m.urgent_status === "needs_trigger_date" || !m.urgent_deadline) return "due-none";
    if ((m.days_until_due ?? 99) <= 7) return "due-red";
    return "due-amber";
  }

  async function saveGlobalFiler() {
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      filerDraft.address_lines = addressText
        .split("\n")
        .map((l) => l.trim())
        .filter(Boolean);
      if (!filerDraft.signature_name.trim()) {
        filerDraft.signature_name = filerDraft.name.trim();
      }
      await ipc<Filer>("save_global_filer_cmd", { filer: filerDraft });
      await loadPortfolio();
      showFilerModal = false;
      statusMsg = "Saved your filer profile. New cases can copy this name and address.";
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="space-y-4">
  <NextUpCard matter={nextMatter} />

  <section class="rounded-lg border border-border bg-card p-4 space-y-3">
    <div class="flex justify-between items-start gap-3 flex-wrap">
      <div>
        <h2 class="text-sm font-semibold">Your identity on papers</h2>
        <p class="text-xs text-muted-foreground">Name, address, phone, and email used on every new case. You can change them per case.</p>
      </div>
      <button
        type="button"
        class="text-xs border border-border px-3 py-1.5 rounded-md"
        aria-label="Edit global filer profile"
        onclick={() => (showFilerModal = !showFilerModal)}
      >
        {portfolio?.global_filer?.name ? `Filer: ${portfolio.global_filer.name}` : "Set filer profile"}
      </button>
    </div>

    {#if showFilerModal}
      <div class="border border-border rounded-md p-3 space-y-3 bg-background">
        <div class="grid gap-2 md:grid-cols-2">
          <label class="text-xs space-y-1">
            Full name
            <input class="w-full bg-background border border-border rounded-md px-2 py-1.5 text-sm" type="text" bind:value={filerDraft.name} placeholder="e.g. Jordan Example" />
          </label>
          <label class="text-xs space-y-1">
            Role
            <input class="w-full bg-background border border-border rounded-md px-2 py-1.5 text-sm" type="text" bind:value={filerDraft.role} placeholder="Proceeding without an attorney" />
          </label>
          <label class="text-xs space-y-1">
            Phone
            <input class="w-full bg-background border border-border rounded-md px-2 py-1.5 text-sm" type="text" bind:value={filerDraft.phone} placeholder="555-0100" />
          </label>
          <label class="text-xs space-y-1">
            Email
            <input class="w-full bg-background border border-border rounded-md px-2 py-1.5 text-sm" type="email" bind:value={filerDraft.email} placeholder="name@example.test" />
          </label>
          <label class="text-xs space-y-1 md:col-span-2">
            Mailing address (one line per row)
            <textarea class="w-full bg-background border border-border rounded-md px-2 py-1.5 text-sm" rows="2" bind:value={addressText} placeholder={"100 Sample Street\nExample City, NJ 00000"}></textarea>
          </label>
        </div>
        <div class="flex gap-2">
          <button type="button" class="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-sm" disabled={busy} onclick={saveGlobalFiler}>Save filer profile</button>
          <button type="button" class="border border-border px-3 py-1.5 rounded-md text-sm" onclick={() => (showFilerModal = false)}>Cancel</button>
        </div>
      </div>
    {/if}

    {#if portfolio && portfolio.matters.length > 0}
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <h3 class="text-sm font-semibold">Other matters ({portfolio.total_matters})</h3>
          <button type="button" class="text-xs border border-border px-2 py-1 rounded-md" onclick={loadPortfolio} aria-label="Refresh case list">Refresh</button>
        </div>
        <div class="space-y-1">
          {#each portfolio.matters as m}
            <a class="flex flex-wrap justify-between gap-2 border border-border rounded-md px-3 py-2 text-sm hover:border-primary/40" href={`/case/${m.case_id}`}>
              <span>
                <span class="text-[10px] uppercase tracking-wide border border-border px-1.5 py-0.5 rounded mr-1">{m.venue_code}</span>
                <strong>{m.title}</strong>
                {#if m.docket_number}<span class="text-xs font-mono text-muted-foreground"> #{m.docket_number}</span>{/if}
              </span>
              <span class="text-xs {dueClass(m)}">
                {#if m.urgent_status === "needs_trigger_date" || !m.urgent_deadline}
                  Enter the order date to compute this deadline
                {:else}
                  {m.urgent_branch_title}: {m.urgent_deadline}
                  {#if m.days_until_due != null} ({m.days_until_due}d){/if}
                  {#if m.unverified} · unverified{/if}
                {/if}
              </span>
            </a>
          {/each}
        </div>
      </div>
    {/if}

    {#if statusMsg}<p class="text-sm text-muted-foreground">{statusMsg}</p>{/if}
    {#if errorMsg}<p class="text-sm text-destructive">{errorMsg}</p>{/if}
  </section>
</div>

<style>
  .due-none { color: hsl(var(--muted-foreground)); }
  .due-amber { color: #fbbf24; }
  .due-red { color: #f87171; }
</style>
