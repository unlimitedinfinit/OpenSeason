<script lang="ts">
  interface Props {
    isOpen?: boolean;
    filePath?: string;
    documentTitle?: string;
    onClose?: () => void;
  }

  let { isOpen = $bindable(false), filePath = "", documentTitle = "Court Document", onClose }: Props = $props();

  function handleClose() {
    isOpen = false;
    if (onClose) onClose();
  }

  async function openHelpUrl(url: string) {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-md">
    <div
      class="relative w-full max-w-2xl rounded-3xl border border-primary/30 bg-card p-6 md:p-8 shadow-2xl space-y-6 text-foreground animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Header -->
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="p-2.5 rounded-2xl bg-primary/10 text-primary border border-primary/20 text-xl shrink-0">
            ✅
          </div>
          <div>
            <h3 class="text-lg font-bold text-foreground">Export Complete: Next Steps</h3>
            <p class="text-xs text-muted-foreground font-mono truncate max-w-md">
              {documentTitle} {filePath ? `· ${filePath}` : ""}
            </p>
          </div>
        </div>
        <button
          onclick={handleClose}
          class="text-muted-foreground hover:text-foreground p-1 text-sm rounded-lg"
          aria-label="Close"
        >
          ✕
        </button>
      </div>

      <!-- Checklist Section -->
      <div class="space-y-4 text-xs">
        <div class="p-4 rounded-2xl bg-secondary/40 border border-border/70 space-y-2.5">
          <h4 class="font-semibold text-foreground uppercase tracking-wider text-[11px] flex items-center gap-1.5">
            <span>📋 Pre-Filing Quality & Procedure Checklist</span>
          </h4>
          <ul class="space-y-2 text-muted-foreground">
            <li class="flex items-start gap-2">
              <span class="text-primary font-bold">1.</span>
              <span><strong>Line-by-Line Review:</strong> Verify every party name, docket number, hearing date, and factual statement in your export before filing.</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-primary font-bold">2.</span>
              <span><strong>Court Filing Deadlines:</strong> Confirm with the clerk's office whether your motion must be filed electronically or by hand, and verify weekend/holiday roll.</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-primary font-bold">3.</span>
              <span><strong>Certificate of Service:</strong> Ensure a copy is served on all opposing counsel via approved service methods (electronic, mail, personal).</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-primary font-bold">4.</span>
              <span><strong>Retain Signed Master:</strong> Always save an exact copy of the signed and filed document with the clerk's stamped date.</span>
            </li>
          </ul>
        </div>

        <!-- Professional Legal Review Hub -->
        <div class="p-4 rounded-2xl bg-primary/5 border border-primary/20 space-y-3">
          <div class="flex items-center justify-between">
            <h4 class="font-semibold text-foreground text-xs flex items-center gap-1.5">
              <span>⚖️ Need Professional Legal Review?</span>
            </h4>
            <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-primary/10 text-primary">Unbundled Options</span>
          </div>
          <p class="text-muted-foreground leading-relaxed text-[11px]">
            You do not necessarily have to hire an attorney for your entire case. Many licensed attorneys offer
            <strong>unbundled (limited-scope) legal services</strong>—meaning you can pay for a 1-hour consultation to review this specific draft, verify local rules, and advise on strategy.
          </p>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1">
            <button
              type="button"
              onclick={() => openHelpUrl("https://justlegal.me/find-legal-help")}
              class="px-3 py-2 text-left rounded-xl border border-border/80 bg-background/80 hover:border-primary/50 text-foreground transition-colors flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-xs">Find Legal Help Router ↗</div>
                <div class="text-[10px] text-muted-foreground">Free aid & affordable consultation finder</div>
              </div>
            </button>

            <button
              type="button"
              onclick={() => openHelpUrl("https://justlegal.me/resources/guides")}
              class="px-3 py-2 text-left rounded-xl border border-border/80 bg-background/80 hover:border-primary/50 text-foreground transition-colors flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-xs">Court Procedural Guides ↗</div>
                <div class="text-[10px] text-muted-foreground">Step-by-step filing & service walkthroughs</div>
              </div>
            </button>
          </div>
        </div>
      </div>

      <!-- Action Footer -->
      <div class="flex justify-end gap-3 pt-2">
        <button
          type="button"
          onclick={handleClose}
          class="px-5 py-2.5 bg-primary text-primary-foreground font-semibold text-xs rounded-xl hover:opacity-90 transition-opacity"
        >
          Done / Continue
        </button>
      </div>
    </div>
  </div>
{/if}
