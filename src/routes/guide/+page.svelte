<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import ExternalConfirm from "$lib/components/ExternalConfirm.svelte";
  import { ipc } from "$lib/ipc";
  import type { GuideQuestions, GuideResult, HelpLink } from "$lib/types/packs";

  let questions = $state<GuideQuestions | null>(null);
  let result = $state<GuideResult | null>(null);
  let error = $state("");
  let jurisdiction = $state("NJ");
  let court = $state("");
  let situation = $state("reconsideration");
  let triggerDate = $state("");
  let confirmOpen = $state(false);
  let pending = $state<HelpLink | null>(null);
  let confirmText = $state("This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?");

  onMount(load);

  async function load() {
    try {
      questions = await ipc<GuideQuestions>("guide_questions_cmd");
      confirmText = questions.confirm_before_open || confirmText;
      const firstState = questions.questions.find((q: { id: string }) => q.id === "state")?.choices[0];
      if (firstState) jurisdiction = firstState.id;
    } catch (e) {
      error = String(e);
    }
  }

  async function find() {
    error = "";
    try {
      result = await ipc<GuideResult>("guide_find_cmd", {
        state: jurisdiction,
        court: court || null,
        situation,
        triggerDate: triggerDate || null,
      });
      confirmText = result.confirm_before_open || confirmText;
    } catch (e) {
      error = String(e);
    }
  }

  function ask(link: HelpLink) {
    pending = link;
    confirmOpen = true;
  }

  const states = $derived(questions?.questions.find((q: { id: string }) => q.id === "state")?.choices || []);
  const courts = $derived(questions?.questions.find((q: { id: string }) => q.id === "court")?.choices || []);
  const situations = $derived(questions?.questions.find((q: { id: string }) => q.id === "situation")?.choices || []);
</script>

<main class="container mx-auto p-8 space-y-6 max-w-3xl">
  <div class="flex justify-between gap-4">
    <div>
      <p class="text-xs uppercase tracking-widest text-muted-foreground">Finder</p>
      <h1 class="text-3xl font-bold">What do I need to file?</h1>
      <p class="text-muted-foreground mt-2">
        Answer in plain language. The app only recommends forms already listed in a pack. It is legal information, not legal advice. It never files or serves.
      </p>
    </div>
    <button onclick={() => goto("/")} class="text-sm border border-border px-3 py-1.5 rounded-md h-fit">Home</button>
  </div>

  <ReviewNotice />

  {#if questions}
    <p class="text-xs text-amber-100 border border-amber-500/30 bg-amber-500/10 rounded-md p-3">{questions.disclaimer}</p>
  {/if}

  <section class="space-y-4 border border-border rounded-lg p-4">
    <label class="block space-y-1">
      <span class="text-sm font-medium">State</span>
      <select bind:value={jurisdiction} class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm">
        {#each states as choice}
          <option value={choice.id}>{choice.label}</option>
        {/each}
      </select>
    </label>
    <label class="block space-y-1">
      <span class="text-sm font-medium">Court (optional)</span>
      <select bind:value={court} class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm">
        <option value="">I will copy it from my papers</option>
        {#each courts as choice}
          <option value={choice.id}>{choice.label}</option>
        {/each}
      </select>
    </label>
    <label class="block space-y-1">
      <span class="text-sm font-medium">What happened?</span>
      <select bind:value={situation} class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm">
        {#each situations as choice}
          <option value={choice.id}>{choice.label}</option>
        {/each}
      </select>
    </label>
    <label class="block space-y-1">
      <span class="text-sm font-medium">Date that started the clock (optional)</span>
      <input bind:value={triggerDate} type="date" class="w-full bg-background border border-border rounded-md px-3 py-2 text-sm" />
      <span class="text-xs text-muted-foreground">Used only when the pack already names a deadline. The finder will not invent a day count.</span>
    </label>
    <button onclick={find} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm">Show what the pack lists</button>
    {#if error}
      <p class="text-sm text-destructive">{error}</p>
    {/if}
  </section>

  {#if result}
    <section class="space-y-4">
      <h2 class="text-xl font-semibold">{result.stuck ? "Nothing in the pack matches" : result.situation_label}</h2>
      <p class="text-sm text-muted-foreground">{result.why}</p>
      {#if result.citation}
        <p class="text-xs font-mono">Source: {result.citation} · {result.citation_status}</p>
      {/if}
      {#if result.deadline}
        <p class="text-sm">Pack math due date: <strong>{result.deadline.due}</strong> ({result.deadline.rule}). Confirm it yourself.</p>
      {/if}
      {#each result.unverified as note}
        <p class="text-sm text-amber-200">Needs verification: {note}</p>
      {/each}

      {#each result.forms as form}
        <article class="border border-border rounded-lg p-4 space-y-2">
          <h3 class="font-semibold">{form.title}</h3>
          <p class="text-xs text-muted-foreground">{form.form_number || "No form number in the pack"} · {form.citation_status}</p>
          {#if form.teach.what_it_is}<p class="text-sm"><strong>What it is.</strong> {form.teach.what_it_is}</p>{/if}
          {#if form.teach.when_used}<p class="text-sm"><strong>When it is used.</strong> {form.teach.when_used}</p>{/if}
          {#if form.teach.common_mistakes.length}
            <div>
              <p class="text-sm font-medium">Common mistakes</p>
              <ul class="list-disc pl-5 text-sm text-muted-foreground">
                {#each form.teach.common_mistakes as item}<li>{item}</li>{/each}
              </ul>
            </div>
          {/if}
          {#if form.teach.learn_more}<p class="text-sm text-muted-foreground">{form.teach.learn_more}</p>{/if}
        </article>
      {/each}

      {#each result.steps as step}
        <article class="border border-border rounded-lg p-4 space-y-2">
          <h3 class="font-semibold">{step.title}</h3>
          <p class="text-sm">{step.why}</p>
          {#if step.teach.what_it_is}<p class="text-sm text-muted-foreground">{step.teach.what_it_is}</p>{/if}
          {#if step.citation}<p class="text-xs font-mono">{step.citation} · {step.citation_status}</p>{/if}
        </article>
      {/each}

      {#if result.stuck || result.unverified.length}
        <div class="border border-indigo-500/40 bg-indigo-500/10 rounded-lg p-4 space-y-2">
          <h3 class="font-semibold">Need help finding a lawyer or a form?</h3>
          <p class="text-sm text-muted-foreground">These JustLegal pages need a browser and internet. A confirmation appears first. The rest of the app stays offline.</p>
          <div class="flex flex-wrap gap-2">
            {#each result.help_links as link}
              <button onclick={() => ask(link)} class="border border-border px-3 py-1.5 rounded-md text-sm">{link.label}</button>
            {/each}
          </div>
        </div>
      {:else}
        <div class="flex flex-wrap gap-2">
          <button onclick={() => goto("/packs")} class="border border-border px-3 py-1.5 rounded-md text-sm">Install form files</button>
          {#if result.help_links[0]}
            <button onclick={() => { const link = result?.help_links[0]; if (link) ask(link); }} class="border border-border px-3 py-1.5 rounded-md text-sm">Get legal help</button>
          {/if}
        </div>
      {/if}
    </section>
  {/if}
</main>

<ExternalConfirm bind:open={confirmOpen} bind:link={pending} confirm={confirmText} />
