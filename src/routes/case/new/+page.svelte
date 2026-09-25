<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import type { CaseProfile } from "$lib/types/case";

  let title = $state("");
  let kind = $derived($page.url.searchParams.get("kind") || "notice_of_appeal");
  let error = $state("");
  let busy = $state(false);

  async function create() {
    if (!title.trim()) {
      error = "Give the case a title, such as Example v. Sample County Clerk.";
      return;
    }
    busy = true;
    error = "";
    try {
      const created = await invoke<CaseProfile>("create_case", {
        title: title.trim(),
        documentKind: kind,
      });
      goto(`/case/${created.id}`);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<main class="container mx-auto max-w-2xl p-8 space-y-6">
  <button onclick={() => goto("/")} class="text-sm text-muted-foreground hover:text-foreground">Back to home</button>
  <h1 class="text-2xl font-bold">New case</h1>
  <p class="text-sm text-muted-foreground">
    This creates a standard folder under Documents/JustLegal/Cases. Caption fields you fill here will be reused on every document in the case.
  </p>
  <ReviewNotice />

  <label class="block space-y-2">
    <span class="text-sm font-medium">Case title</span>
    <input
      bind:value={title}
      class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm"
      placeholder="Example v. Sample County Clerk"
      onkeydown={(e) => e.key === "Enter" && create()}
    />
  </label>
  <p class="text-xs text-muted-foreground">Document: {kind.replaceAll("_", " ")}</p>
  {#if error}
    <p class="text-sm text-destructive">{error}</p>
  {/if}
  <button
    onclick={create}
    disabled={busy}
    class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm font-medium disabled:opacity-50"
  >
    {busy ? "Creating..." : "Create case folder"}
  </button>
</main>
