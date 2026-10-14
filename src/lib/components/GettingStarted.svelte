<script lang="ts">
  const steps = [
    "Choose what you need in plain language (look again, more time, pause, appeal).",
    "Fill your name, the other side, the court, and the case number from papers you already have.",
    "Answer the short writing prompts. Leave a box blank rather than guessing.",
    "Add the papers you have. The first file is Exhibit A. Link each fact to a letter.",
    "Run Check. Fix the red list. Then export Word and PDF. You file and serve yourself.",
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
      <h2 class="font-semibold">Five steps to a court-shaped motion</h2>
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
