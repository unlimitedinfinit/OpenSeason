<script lang="ts">
  let { text }: { text: string } = $props();
  let open = $state(false);
</script>

<span class="relative inline-block align-middle">
  <button
    type="button"
    class="text-[11px] px-1.5 py-0.5 rounded border border-border text-muted-foreground"
    aria-label="Help"
    onclick={() => (open = !open)}
  >
    ?
  </button>
  {#if open}
    <span class="absolute z-20 left-0 mt-1 w-64 text-xs bg-card border border-border rounded-md p-2 shadow">
      {text}
    </span>
  {/if}
</span>
