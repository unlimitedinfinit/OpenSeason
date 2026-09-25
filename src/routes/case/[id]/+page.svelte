<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import type { CaseProfile, SyncRefusal, ValidationReport } from "$lib/types/case";

  let caseId = $derived($page.params.id);
  let profile = $state<CaseProfile | null>(null);
  let error = $state("");
  let status = $state("");
  let report = $state<ValidationReport | null>(null);
  let syncResult = $state<SyncRefusal | null>(null);
  let busy = $state(false);

  onMount(load);

  async function load() {
    try {
      profile = await invoke<CaseProfile>("get_case", { caseId });
    } catch (e) {
      error = String(e);
    }
  }

  function partyLine(list: { name: string; role: string }[]): string {
    return list.map((p) => (p.role ? `${p.name} (${p.role})` : p.name)).join("\n");
  }

  function parseParties(text: string, fallbackRole: string): { name: string; role: string }[] {
    return text
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const match = line.match(/^(.*)\((.+)\)\s*$/);
        if (match) {
          return { name: match[1].trim(), role: match[2].trim() };
        }
        return { name: line, role: fallbackRole };
      });
  }

  let appellantsText = $state("");
  let appelleesText = $state("");
  let addressText = $state("");
  let hydratedId = $state("");

  $effect(() => {
    if (profile && hydratedId !== profile.id) {
      appellantsText = partyLine(profile.caption.appellants.length ? profile.caption.appellants : profile.caption.plaintiffs);
      appelleesText = partyLine(profile.caption.appellees.length ? profile.caption.appellees : profile.caption.defendants);
      addressText = profile.filer.address_lines.join("\n");
      hydratedId = profile.id;
    }
  });

  function applyLists() {
    if (!profile) return;
    const appellants = parseParties(appellantsText, "Appellant");
    const appellees = parseParties(appelleesText, "Appellee");
    profile.caption.appellants = appellants;
    profile.caption.plaintiffs = appellants;
    profile.caption.appellees = appellees;
    profile.caption.defendants = appellees;
    profile.filer.address_lines = addressText.split("\n").map((l) => l.trim()).filter(Boolean);
  }

  async function save() {
    if (!profile) return;
    applyLists();
    busy = true;
    error = "";
    try {
      profile = await invoke<CaseProfile>("save_case", { caseId, profile });
      status = "Saved to case.json";
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function validate() {
    await save();
    try {
      report = await invoke<ValidationReport>("validate_case_cmd", { caseId });
      status = report.ok ? "Ready to export." : "Fix the issues below before export.";
    } catch (e) {
      error = String(e);
    }
  }

  async function exportDoc() {
    await save();
    busy = true;
    error = "";
    try {
      const result = await invoke<{ docx: string; pdf: string }>("export_notice_of_appeal_cmd", { caseId });
      status = `Wrote Word and PDF:\n${result.docx}\n${result.pdf}`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function trySync(action: string) {
    try {
      syncResult = await invoke<SyncRefusal>("sync_case_cmd", { caseId, action });
    } catch (e) {
      error = String(e);
    }
  }

  async function seal() {
    if (!confirm("Convert this case to Confidential mode? This cannot be reversed. Sync and cloud backup will be refused forever. Unlock Confidential mode first.")) {
      return;
    }
    busy = true;
    try {
      await invoke("convert_case_to_confidential", { caseId });
      status = "Case sealed. Open it from Confidential (Open Season).";
      goto("/confidential");
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<main class="container mx-auto max-w-4xl p-8 space-y-6">
  <button onclick={() => goto("/")} class="text-sm text-muted-foreground hover:text-foreground">Back to home</button>

  {#if !profile}
    <p class="text-sm text-muted-foreground">{error || "Loading case..."}</p>
  {:else}
    <div class="flex justify-between items-start gap-4">
      <div>
        <p class="text-xs uppercase tracking-widest text-muted-foreground">Standard case</p>
        <h1 class="text-2xl font-bold">{profile.title || "Untitled case"}</h1>
        <p class="text-xs font-mono text-muted-foreground mt-1">{profile.id}</p>
      </div>
      <span class="text-xs px-2 py-1 rounded border border-border">mode: {profile.mode}</span>
    </div>

    <ReviewNotice />

    <section class="space-y-4 bg-card border rounded-lg p-6">
      <h2 class="font-semibold">Case profile (fills every document)</h2>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Title</span>
        <input bind:value={profile.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Court name</span>
        <input bind:value={profile.court.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="United States Court of Appeals for the Sample Circuit" />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Docket number</span>
        <input bind:value={profile.docket_number} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="24-01000" />
      </label>
      <div class="grid md:grid-cols-2 gap-4">
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Appellants or plaintiffs (one per line)</span>
          <textarea bind:value={appellantsText} class="w-full h-24 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Appellees or defendants (one per line)</span>
          <textarea bind:value={appelleesText} class="w-full h-24 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
        </label>
      </div>
      <div class="grid md:grid-cols-2 gap-4">
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Filer name</span>
          <input bind:value={profile.filer.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Signature name</span>
          <input bind:value={profile.filer.signature_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
      </div>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Filer role</span>
        <input bind:value={profile.filer.role} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Appellant, proceeding without an attorney" />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Address (one line per row)</span>
        <textarea bind:value={addressText} class="w-full h-20 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
      </label>
      <div class="grid md:grid-cols-2 gap-4">
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Phone</span>
          <input bind:value={profile.filer.phone} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Email</span>
          <input bind:value={profile.filer.email} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
      </div>
    </section>

    <section class="space-y-4 bg-card border rounded-lg p-6">
      <h2 class="font-semibold">Notice of Appeal (your words)</h2>
      <p class="text-xs text-muted-foreground">
        OpenSeason will not invent grounds, facts, or citations. Paste only text you wrote or researched.
      </p>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Trial court</span>
        <input bind:value={profile.notice_of_appeal.trial_court_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
      <div class="grid md:grid-cols-2 gap-4">
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Trial court docket</span>
          <input bind:value={profile.notice_of_appeal.trial_court_docket} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Judgment or order date</span>
          <input type="date" bind:value={profile.notice_of_appeal.judgment_date} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
      </div>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">What order or judgment is being appealed?</span>
        <input bind:value={profile.notice_of_appeal.judgment_description} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Order granting the motion to dismiss" />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Your notice text</span>
        <textarea
          bind:value={profile.notice_of_appeal.user_text}
          class="w-full h-40 rounded-md border border-input bg-background px-3 py-2 text-sm"
          placeholder="Paste the notice language you drafted. Do not leave [PLACEHOLDERS] or TODO markers."
        ></textarea>
      </label>
    </section>

    {#if report && !report.ok}
      <ul class="text-sm text-destructive list-disc pl-5 space-y-1">
        {#each report.issues as issue}
          <li>{issue.field}: {issue.message}</li>
        {/each}
      </ul>
    {/if}

    {#if status}
      <pre class="text-xs whitespace-pre-wrap bg-muted/20 border rounded p-3">{status}</pre>
    {/if}
    {#if error}
      <p class="text-sm text-destructive">{error}</p>
    {/if}

    <div class="flex flex-wrap gap-2">
      <button onclick={save} disabled={busy} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Save profile</button>
      <button onclick={validate} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Validate</button>
      <button onclick={exportDoc} disabled={busy} class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm">Export Word and PDF</button>
      <button onclick={() => trySync("backup")} class="border border-border px-4 py-2 rounded-md text-sm">Account backup (stub)</button>
      <button onclick={seal} disabled={busy} class="border border-red-500/40 text-red-300 px-4 py-2 rounded-md text-sm">Convert to Confidential</button>
    </div>

    {#if syncResult}
      <p class="text-xs text-muted-foreground">{syncResult.kind}: {syncResult.reason}</p>
    {/if}

    <p class="text-xs text-muted-foreground">
      Guests: keep a local copy of this folder. Creating a justlegal.me account so you can publish finished documents is planned, not built.
    </p>
  {/if}
</main>
