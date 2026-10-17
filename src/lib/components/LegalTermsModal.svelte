<script lang="ts">
  import { onMount } from "svelte";

  const STORAGE_KEY = "justlegal_legal_terms_accepted_v1";

  let isOpen = $state(false);
  let doNotShowAgain = $state(true);

  onMount(() => {
    const accepted = localStorage.getItem(STORAGE_KEY);
    if (!accepted) {
      isOpen = true;
    }
  });

  export function openModal() {
    isOpen = true;
  }

  function handleAccept() {
    if (doNotShowAgain) {
      localStorage.setItem(STORAGE_KEY, "true");
    }
    isOpen = false;
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-md">
    <div
      class="relative w-full max-w-xl rounded-3xl border border-primary/30 bg-card p-6 md:p-8 shadow-2xl space-y-6 text-foreground animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Header -->
      <div class="flex items-start gap-4">
        <div class="p-3 rounded-2xl bg-primary/10 text-primary border border-primary/20 text-2xl shrink-0">
          ⚖️
        </div>
        <div class="space-y-1">
          <h2 class="text-xl font-bold tracking-tight text-foreground">Important Legal Notice & Terms of Use</h2>
          <p class="text-xs font-mono text-muted-foreground uppercase tracking-wider">
            Just Liberty 501(c)(3) · OpenSeason Legal Operating System
          </p>
        </div>
      </div>

      <!-- Content -->
      <div class="space-y-3 text-xs leading-relaxed text-muted-foreground border-y border-border/60 py-4 max-h-72 overflow-y-auto pr-1">
        <div class="p-3 rounded-xl bg-secondary/50 border border-border/80 text-foreground font-medium">
          OpenSeason and JustLegal Desktop provide legal information, formatting aids, and administrative case organization tools.
          <strong class="text-primary font-semibold">This software is NOT legal advice and does NOT practice law.</strong>
        </div>

        <p>
          1. <strong class="text-foreground">Formatting and Assembly Only:</strong> This software formats your own words, evidence, and factual observations into court-shaped pleadings, orders, and disclosure statements. It does not invent facts, fabricate citations, or formulate strategic legal theories.
        </p>

        <p>
          2. <strong class="text-foreground">Never Files or Serves:</strong> OpenSeason will never file papers with a court or serve opposing parties on your behalf. You (or your attorney) remain solely responsible for reviewing, signing, filing, and serving every document in compliance with all applicable court rules.
        </p>

        <p>
          3. <strong class="text-foreground">Attorney Consultation Recommended:</strong> Before filing any document or motion in court, we strongly advise consulting with a licensed attorney or contacting your local bar association's lawyer referral service for an unbundled document review.
        </p>

        <p>
          4. <strong class="text-foreground">Strict Privacy & Airgap:</strong> Standard cases live in your personal documents folder. Confidential and Qui Tam cases are encrypted locally with zero cloud telemetry and will never be uploaded to remote AI servers.
        </p>
      </div>

      <!-- Controls -->
      <div class="space-y-4 pt-1">
        <label class="flex items-center gap-2.5 text-xs text-foreground cursor-pointer select-none">
          <input
            type="checkbox"
            bind:checked={doNotShowAgain}
            class="h-4 w-4 rounded border-border text-primary focus:ring-primary/40"
          />
          <span>Do not show this acknowledgment again on startup</span>
        </label>

        <div class="flex justify-end gap-3">
          <button
            type="button"
            onclick={handleAccept}
            class="px-6 py-2.5 bg-primary hover:opacity-90 text-primary-foreground font-semibold text-xs rounded-xl shadow-md transition-opacity"
          >
            I Understand & Accept
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
