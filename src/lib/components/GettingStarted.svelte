<script lang="ts">
  const steps = [
    "Open Draft to PDF. Paste a Markdown draft you wrote, or choose a .md file.",
    "Click Preview court paper. Check the boxed caption, signature, and certificate of service.",
    "Click Export Word and PDF. You file and serve yourself. This app never files.",
    "If you are unsure what to file, open What to file. Deadlines stay blank until you enter an order date.",
  ];
  let hidden = $state(true);

  $effect(() => {
    if (typeof localStorage === "undefined") return;
    hidden = localStorage.getItem("os-tour-done") === "1";
  });

  function dismiss() {
    hidden = true;
    if (typeof localStorage !== "undefined") localStorage.setItem("os-tour-done", "1");
  }
</script>

{#if !hidden}
  <section class="border border-indigo-500/40 bg-indigo-500/10 rounded-lg p-5 space-y-3">
    <div class="flex justify-between gap-3">
      <h2 class="font-semibold">Three clicks to a court-shaped PDF</h2>
      <button onclick={dismiss} class="text-xs border border-border px-2 py-1 rounded-md">Got it</button>
    </div>
    <ol class="list-decimal pl-5 text-sm space-y-1 text-muted-foreground">
      {#each steps as step}
        <li>{step}</li>
      {/each}
    </ol>
    <p class="text-xs text-muted-foreground">This app never files or serves. It is not legal advice.</p>
  </section>
{/if}
