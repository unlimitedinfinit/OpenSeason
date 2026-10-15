<script lang="ts">
  import { goto } from "$app/navigation";

  interface Props {
    sessionUnlocked: boolean;
    targetLookup: string;
    targetHits: { recipient_name?: string; award_amount?: number; description?: string }[];
    onlookuptarget: () => void;
    ondisclosure: () => void;
    onexportosb: () => void;
  }

  let {
    sessionUnlocked,
    targetLookup = $bindable(),
    targetHits,
    onlookuptarget,
    ondisclosure,
    onexportosb,
  }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Confidential tools</h2>
  <p class="text-sm text-muted-foreground">These used to live on a separate hunt screen. They now belong to this Confidential case. The optional USAspending lookup sends only a target name to a public API. It does not upload the case folder.</p>
  {#if !sessionUnlocked}
    <button onclick={() => goto("/confidential")} class="border border-red-500/40 text-red-200 px-4 py-2 rounded-md text-sm">Unlock vault</button>
  {:else}
    <div class="flex gap-2">
      <input bind:value={targetLookup} class="flex-1 h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Agency or contractor name" />
      <button onclick={onlookuptarget} class="border border-border px-3 rounded-md text-sm">Look up target</button>
    </div>
    {#if targetHits.length}
      <ul class="text-sm space-y-1">
        {#each targetHits as hit}
          <li>{hit.recipient_name || "Unknown"} · {hit.award_amount ?? "n/a"} · {hit.description || ""}</li>
        {/each}
      </ul>
    {/if}
    <div class="flex flex-wrap gap-2">
      <button onclick={ondisclosure} class="border border-red-500/40 text-red-200 px-4 py-2 rounded-md text-sm">Disclosure statement</button>
      <button onclick={onexportosb} class="border border-border px-4 py-2 rounded-md text-sm">Export .osb</button>
    </div>
  {/if}
</section>
