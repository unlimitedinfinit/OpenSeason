<script lang="ts">
  import { goto } from "$app/navigation";
  import { ipc } from "$lib/ipc";
  import ExternalConfirm from "$lib/components/ExternalConfirm.svelte";
  import type { HelpLink, HelpLinks } from "$lib/types/packs";

  let open = $state(false);
  let confirmOpen = $state(false);
  let confirmText = $state("This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?");
  let pending = $state<HelpLink | null>(null);
  let links = $state<HelpLink[]>([]);

  async function load() {
    try {
      const set = await ipc<HelpLinks>("help_links_cmd");
      links = set.links || [];
      confirmText = set.confirm || confirmText;
    } catch {
      links = [];
    }
  }

  function ask(link: HelpLink) {
    pending = link;
    confirmOpen = true;
    open = false;
  }
</script>

<div class="relative">
  <button
    class="text-sm border border-border px-3 py-1.5 rounded-md"
    onclick={() => {
      open = !open;
      if (open) load();
    }}
  >
    Help
  </button>
  {#if open}
    <div class="absolute right-0 mt-2 w-72 border border-border bg-card rounded-md shadow-lg z-40 p-2 space-y-1">
      <button onclick={() => { open = false; goto("/guide"); }} class="w-full text-left text-sm px-2 py-1.5 rounded hover:bg-accent/20">
        What do I need to file?
      </button>
      <button onclick={() => { open = false; goto("/packs"); }} class="w-full text-left text-sm px-2 py-1.5 rounded hover:bg-accent/20">
        Forms packs
      </button>
      <div class="border-t border-border my-1"></div>
      {#each links as link}
        <button onclick={() => ask(link)} class="w-full text-left text-sm px-2 py-1.5 rounded hover:bg-accent/20">
          {link.label}
        </button>
      {/each}
      <p class="text-[10px] text-muted-foreground px-2 py-1">A confirmation appears before any website opens. The app stays usable offline.</p>
    </div>
  {/if}
</div>

<ExternalConfirm bind:open={confirmOpen} bind:link={pending} confirm={confirmText} />
