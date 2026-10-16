<script lang="ts">
  import "../app.css";
  import HelpMenu from "$lib/components/HelpMenu.svelte";
  import { goto } from "$app/navigation";
  let { children } = $props();
</script>

<div class="dark min-h-screen bg-background text-foreground font-sans antialiased">
  <header class="border-b border-border px-6 py-3 flex items-center justify-between gap-4">
    <nav class="flex flex-wrap items-center gap-5 text-sm" aria-label="Main">
      <button onclick={() => goto("/")} class="flex items-center gap-2 font-semibold text-foreground tracking-tight hover:opacity-90" aria-label="Home">
        <span class="w-2.5 h-2.5 rounded-full bg-primary inline-block"></span>
        <span>JustLegal Desktop</span>
        <span class="text-[10px] font-normal uppercase tracking-wider px-1.5 py-0.5 rounded bg-muted text-muted-foreground border border-border">OS</span>
      </button>
      <button onclick={() => goto("/")} class="text-muted-foreground hover:text-foreground transition-colors">Workspace</button>
      <button onclick={() => goto("/forms")} class="text-muted-foreground hover:text-foreground transition-colors">Forms & Templates</button>
      <button onclick={() => goto("/draft")} class="text-muted-foreground hover:text-foreground transition-colors">Draft to PDF</button>
      <button onclick={() => goto("/guide")} class="text-muted-foreground hover:text-foreground transition-colors">What to file</button>
      <button onclick={() => goto("/ai")} class="text-muted-foreground hover:text-foreground transition-colors">For your AI</button>
    </nav>
    <HelpMenu />
  </header>
  {@render children()}
</div>
