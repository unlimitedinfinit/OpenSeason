<script lang="ts">
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import DraftToPdfCard from "$lib/components/DraftToPdfCard.svelte";

  let markdown = $state("");
</script>

<main class="container mx-auto p-8 space-y-6 max-w-4xl">
  <div class="flex justify-between gap-4">
    <div>
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Draft to PDF</p>
      <h1 class="text-3xl font-bold">Your words, court-shaped paper</h1>
      <p class="text-muted-foreground mt-2 max-w-2xl">
        Import a Markdown draft, preview the boxed caption, then export Word and PDF. OpenSeason does not invent facts, citations, or deadlines. It never files or serves.
      </p>
    </div>
    <button onclick={() => goto("/")} class="text-sm border border-border px-3 py-1.5 rounded-md h-fit">Home</button>
  </div>
  <ReviewNotice />
  <DraftToPdfCard bind:markdown />
</main>
