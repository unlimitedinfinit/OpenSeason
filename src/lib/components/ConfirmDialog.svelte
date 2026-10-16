<script lang="ts">
  let {
    open = $bindable(false),
    title = "Please confirm",
    body = "",
    confirmLabel = "Continue",
    cancelLabel = "Cancel",
    danger = false,
    onconfirm = () => {},
    oncancel = () => {},
  }: {
    open: boolean;
    title?: string;
    body?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
    onconfirm?: () => void;
    oncancel?: () => void;
  } = $props();

  function cancel() {
    open = false;
    oncancel();
  }

  function confirm() {
    open = false;
    onconfirm();
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
    <div class="w-full max-w-md rounded-lg border border-border bg-card p-5 space-y-4 shadow-xl">
      <h2 id="confirm-title" class="text-lg font-semibold">{title}</h2>
      {#if body}
        <p class="text-sm text-muted-foreground whitespace-pre-wrap">{body}</p>
      {/if}
      <div class="flex justify-end gap-2">
        <button type="button" class="px-3 py-1.5 rounded-md border border-border text-sm" onclick={cancel}>{cancelLabel}</button>
        <button
          type="button"
          class="px-3 py-1.5 rounded-md text-sm {danger ? 'bg-red-700 text-white' : 'bg-primary text-primary-foreground'}"
          onclick={confirm}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
