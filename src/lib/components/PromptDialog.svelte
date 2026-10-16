<script lang="ts">
  let {
    open = $bindable(false),
    title = "Enter a value",
    body = "",
    placeholder = "",
    initial = "",
    confirmLabel = "OK",
    cancelLabel = "Cancel",
    onsubmit = (_value: string) => {},
    oncancel = () => {},
  }: {
    open: boolean;
    title?: string;
    body?: string;
    placeholder?: string;
    initial?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    onsubmit?: (value: string) => void;
    oncancel?: () => void;
  } = $props();

  let value = $state(initial);

  $effect(() => {
    if (open) value = initial;
  });

  function cancel() {
    open = false;
    oncancel();
  }

  function submit() {
    open = false;
    onsubmit(value);
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60" role="dialog" aria-modal="true" aria-labelledby="prompt-title">
    <form class="w-full max-w-md rounded-lg border border-border bg-card p-5 space-y-4 shadow-xl" onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <h2 id="prompt-title" class="text-lg font-semibold">{title}</h2>
      {#if body}
        <p class="text-sm text-muted-foreground">{body}</p>
      {/if}
      <label class="block space-y-1">
        <span class="sr-only">{title}</span>
        <input
          class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm font-mono"
          bind:value
          {placeholder}
          aria-label={title}
        />
      </label>
      <div class="flex justify-end gap-2">
        <button type="button" class="px-3 py-1.5 rounded-md border border-border text-sm" onclick={cancel}>{cancelLabel}</button>
        <button type="submit" class="px-3 py-1.5 rounded-md bg-primary text-primary-foreground text-sm">{confirmLabel}</button>
      </div>
    </form>
  </div>
{/if}
