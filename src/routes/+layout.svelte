<script lang="ts">
  import "../app.css";
  import HelpMenu from "$lib/components/HelpMenu.svelte";
  import { goto } from "$app/navigation";
  let { children } = $props();
</script>

<div class="dark min-h-screen bg-background text-foreground font-sans antialiased">
  <header class="border-b border-border px-6 py-3 flex items-center justify-between gap-4">
    <nav class="flex flex-wrap items-center gap-4 text-sm" aria-label="Main">
      <button onclick={() => goto("/")} class="font-semibold" aria-label="Home">OpenSeason</button>
      <button onclick={() => goto("/draft")} class="text-muted-foreground hover:text-foreground">Draft to PDF</button>
      <button onclick={() => goto("/guide")} class="text-muted-foreground hover:text-foreground">What to file</button>
      <button onclick={() => goto("/packs")} class="text-muted-foreground hover:text-foreground">Forms packs</button>
      <button onclick={() => goto("/ai")} class="text-muted-foreground hover:text-foreground">For your AI</button>
    </nav>
    <HelpMenu />
  </header>
  {@render children()}
</div>
