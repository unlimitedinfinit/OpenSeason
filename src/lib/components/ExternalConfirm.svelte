<script lang="ts">
  import type { HelpLink } from "$lib/types/packs";

  let {
    open = $bindable(false),
    link = $bindable<HelpLink | null>(null),
    confirm = "This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?",
  }: {
    open?: boolean;
    link?: HelpLink | null;
    confirm?: string;
  } = $props();

  async function proceed() {
    const url = link?.url;
    open = false;
    if (!url) return;
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  function cancel() {
    open = false;
    link = null;
  }
</script>

{#if open && link}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <button class="absolute inset-0 bg-background/80" onclick={cancel} aria-label="Dismiss overlay"></button>
    <div class="relative w-full max-w-lg border border-amber-500/40 bg-card rounded-lg p-5 space-y-3 shadow-xl" role="dialog" aria-modal="true">
      <p class="text-xs uppercase tracking-wide text-amber-200">Open a website</p>
      <h2 class="text-lg font-semibold">{link.label}</h2>
      <p class="text-sm text-muted-foreground">{confirm}</p>
      <p class="text-xs font-mono break-all text-muted-foreground">{link.url}</p>
      <p class="text-xs text-muted-foreground">{link.description}</p>
      <div class="flex justify-end gap-2 pt-2">
        <button onclick={cancel} class="border border-border px-3 py-1.5 rounded-md text-sm">Cancel</button>
        <button onclick={proceed} class="bg-primary text-primary-foreground px-3 py-1.5 rounded-md text-sm">Continue</button>
      </div>
    </div>
  </div>
{/if}
