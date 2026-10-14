<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import PartyEditor from "$lib/components/PartyEditor.svelte";
  import FieldHelp from "$lib/components/FieldHelp.svelte";
  import { ipc } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import type {
    CaseArtifact,
    CaseProfile,
    ExportResult,
    Party,
    SyncRefusal,
    TimelineEvent,
    ValidationReport,
    GuidedDraft,
  } from "$lib/types/case";

  let caseId = $derived($page.params.id || "");
  let profile = $state<CaseProfile | null>(null);
  let parties = $state<Party[]>([]);
  let addressText = $state("");
  let tab = $state<"overview" | "parties" | "documents" | "evidence" | "timeline" | "files" | "check" | "confidential">("overview");
  let checkReport = $state<{
    ok?: boolean;
    never_files?: boolean;
    issues?: { field: string; message: string }[];
    fact?: { rows: { id: number; assertion: string; record_cite: string; exhibit?: string; verified: boolean }[] };
    authority?: { rows: { id: number; cite: string; status: string }[] };
    deadline?: { due: string; math: string; rule: string };
    signoff?: { items: { id: string; label: string; checked: boolean }[]; ok?: boolean };
  } | null>(null);
  let guided = $state<{ kind: string; sections: { id: string; heading: string; hint: string; user_text: string }[] } | null>(null);
  let exhibits = $state<{ letter: string; filename: string; description: string }[]>([]);
  let gapTitle = $state("");
  let gapHard = $state("");
  let deadlinePack = $state("nj-app-div");
  let deadlineRule = $state("reconsideration-appellate");
  let triggerDate = $state("");
  let mdDraft = $state("");
  let error = $state("");
  let status = $state("");
  let report = $state<ValidationReport | null>(null);
  let syncResult = $state<SyncRefusal | null>(null);
  let artifacts = $state<CaseArtifact[]>([]);
  let exportPaths = $state<ExportResult | null>(null);
  let busy = $state(false);
  let sealPassword = $state("");
  let eventTitle = $state("");
  let eventDate = $state("");
  let eventNotes = $state("");
  let targetLookup = $state("");
  let targetHits = $state<{ recipient_name?: string; award_amount?: number; description?: string }[]>([]);
  let hydrated = $state("");

  onMount(load);

  async function load() {
    error = "";
    try {
      profile = await ipc<CaseProfile>("get_case", { caseId });
      hydrate(profile);
      artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => []);
      guided = await ipc<GuidedDraft>("guided_sections_cmd", { caseId }).catch(() => null);
      const idx = await ipc<{ items?: { letter: string; filename: string; description: string }[] }>("exhibit_index_cmd", { caseId }).catch(() => ({ items: [] }));
      exhibits = idx.items || [];
    } catch (e) {
      error = String(e);
    }
  }

  function hydrate(next: CaseProfile) {
    if (hydrated === next.id + next.updated_at) return;
    const left = next.caption.appellants.length ? next.caption.appellants : next.caption.plaintiffs;
    const right = next.caption.appellees.length ? next.caption.appellees : next.caption.defendants;
    parties = [...left, ...right].map((p) => ({
      name: p.name,
      role: p.role,
      address: p.address || "",
      email: p.email || "",
      phone: p.phone || "",
      counsel: p.counsel || "",
    }));
    addressText = next.filer.address_lines.join("\n");
    if (!next.complaint) next.complaint = { title: "", user_text: "" };
    if (!next.motion) next.motion = { title: "", user_text: "" };
    if (!next.brief) next.brief = { title: "", user_text: "" };
    if (!next.appellate_motion) next.appellate_motion = { title: "", user_text: "" };
    if (!next.mandamus) next.mandamus = { title: "", user_text: "" };
    if (!next.timeline) next.timeline = [];
    hydrated = next.id + next.updated_at;
  }

  function apply() {
    if (!profile) return;
    const left = parties.filter((p) => ["Plaintiff", "Appellant", "Petitioner"].includes(p.role));
    const right = parties.filter((p) => ["Defendant", "Appellee", "Respondent"].includes(p.role));
    const other = parties.filter((p) => !left.includes(p) && !right.includes(p));
    profile.caption.plaintiffs = left.length ? left : other;
    profile.caption.appellants = left;
    profile.caption.defendants = right;
    profile.caption.appellees = right;
    profile.filer.address_lines = addressText.split("\n").map((l) => l.trim()).filter(Boolean);
  }

  async function save() {
    if (!profile) return;
    apply();
    busy = true;
    error = "";
    try {
      profile = await ipc<CaseProfile>("save_case", { caseId, profile });
      hydrate(profile);
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
      report = await ipc<ValidationReport>("validate_case_cmd", { caseId });
      status = report.ok ? "Ready to export the selected document." : "Fix the issues below before export.";
    } catch (e) {
      error = String(e);
    }
  }

  async function exportKind(kind: string) {
    await save();
    busy = true;
    error = "";
    try {
      exportPaths = await ipc<ExportResult>("export_pleading_cmd", { caseId, kind });
      status = `Wrote Word and PDF:\n${exportPaths.docx}\n${exportPaths.pdf}`;
      artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => artifacts);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function trySync(action: string) {
    try {
      syncResult = await ipc<SyncRefusal>("sync_case_cmd", { caseId, action });
    } catch (e) {
      error = String(e);
    }
  }

  async function seal() {
    if (!sealPassword) {
      error = "Re-enter the Confidential vault password to seal. A wrong password is refused so files are not encrypted under an unknown key.";
      return;
    }
    if (!session.unlocked) {
      error = "Unlock Confidential mode first.";
      return;
    }
    if (!confirm("Convert this case to a sealed Confidential vault? This cannot be reversed by editing case.json. Sync will be refused.")) {
      return;
    }
    busy = true;
    try {
      await ipc("convert_case_to_confidential", { caseId, password: sealPassword });
      sealPassword = "";
      status = "Case sealed. The vault copy is encrypted. This build has no in-app reader for sealed files.";
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function addTimeline() {
    if (!profile || !eventTitle || !eventDate) {
      error = "Timeline events need a title and a date.";
      return;
    }
    const item: TimelineEvent = {
      id: crypto.randomUUID(),
      title: eventTitle,
      date: eventDate,
      notes: eventNotes,
    };
    profile.timeline = [...(profile.timeline || []), item];
    eventTitle = "";
    eventDate = "";
    eventNotes = "";
  }

  function removeTimeline(id: string) {
    if (!profile) return;
    profile.timeline = (profile.timeline || []).filter((e) => e.id !== id);
  }

  async function addEvidence() {
    const input = document.createElement("input");
    input.type = "file";
    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file || !profile) return;
      const desc = prompt("Short description for this exhibit:", file.name);
      if (desc === null) return;
      try {
        const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
        if (profile.mode === "confidential") {
          const password = prompt("If this case is sealed, re-enter the vault password. Leave empty only for an unsealed Confidential case.");
          const result = await ipc<string>("add_hunt_evidence_bytes", {
            huntId: caseId,
            filename: file.name,
            fileBytes: bytes,
            description: desc,
            password: password && password.length ? password : null,
          });
          status =
            result === "already_present"
              ? `This exhibit is already in the vault (same file contents). The existing sealed copy was not changed: ${file.name}`
              : `Encrypted exhibit stored in the vault: ${file.name}`;
        } else {
          await ipc("add_case_evidence", {
            caseId,
            filename: file.name,
            fileBytes: bytes,
            description: desc,
          });
          status = `Copied exhibit into the case evidence folder: ${file.name}`;
        }
        artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => artifacts);
      } catch (err) {
        error = String(err);
      }
    };
    input.click();
  }

  async function lookupTarget() {
    if (!targetLookup.trim()) return;
    try {
      targetHits = await ipc("verify_target_cmd", { name: targetLookup.trim() });
    } catch (e) {
      error = String(e);
    }
  }

  async function disclosure() {
    if (!profile) return;
    const password = prompt("If this case is sealed, re-enter the vault password. Leave empty only for an unsealed Confidential case.");
    try {
      const path = await ipc<string>("save_disclosure_cmd", {
        huntId: caseId,
        target: profile.title,
        count: artifacts.filter((a) => a.kind === "evidence").length,
        value: 0,
        password: password && password.length ? password : null,
      });
      status = `Disclosure statement saved:\n${path}`;
    } catch (e) {
      error = String(e);
    }
  }

  async function importMarkdown() {
    if (!mdDraft.trim()) {
      error = "Paste a Markdown draft first.";
      return;
    }
    try {
      await ipc("import_markdown_cmd", { caseId, markdown: mdDraft });
      status = "Imported Markdown into drafts/draft.md. The checker will not invent cites from this text.";
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function fillSection(id: string, text: string) {
    try {
      guided = await ipc<GuidedDraft>("guided_fill_cmd", { caseId, id, text });
      status = "Saved that section. The checker still will not invent cites.";
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function linkFact(factId: number, letter: string) {
    try {
      await ipc("link_fact_cmd", { caseId, factId, letter });
      await runChecks();
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleSignoff(id: string, checked: boolean) {
    try {
      await ipc("set_signoff_cmd", { caseId, id, checked });
      await runChecks();
    } catch (e) {
      error = String(e);
    }
  }

  async function exportPacket() {
    try {
      const report = await ipc<{ items?: { label: string; docx: string }[] }>("export_packet_cmd", { caseId });
      status = `Packet written. ${(report.items || []).map((i) => i.label).join(", ")}. You still file and serve.`;
    } catch (e) {
      error = String(e);
    }
  }

  async function exportIcs() {
    try {
      const path = await ipc<string>("export_ics_cmd", {
        caseId,
        pack: deadlinePack,
        rule: deadlineRule,
        triggerDate,
      });
      status = `Calendar file: ${path}`;
    } catch (e) {
      error = String(e);
    }
  }

  async function addGap() {
    if (!gapTitle.trim()) return;
    try {
      await ipc("gap_add_cmd", {
        caseId,
        title: gapTitle,
        court: profile?.court.name || "",
        rule: deadlineRule,
        whatHappened: gapTitle,
        whatWasHard: gapHard,
      });
      const dest = await ipc<string>("gap_export_cmd", { caseId });
      status = `Gap note saved. ${dest}`;
      gapTitle = "";
      gapHard = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function runChecks() {
    try {
      checkReport = await ipc("check_case_cmd", {
        caseId,
        pack: deadlinePack,
        rule: deadlineRule,
        triggerDate: triggerDate || null,
      });
      status = checkReport?.ok
        ? "Check gates passed. The app still did not file or serve."
        : "Check found issues. See the list. The app never invents cites.";
    } catch (e) {
      error = String(e);
    }
  }

  async function exportOsb() {
    const path = prompt("Optional target path for the .osb file. Leave empty to save in Downloads.");
    try {
      const out = await ipc<string>("export_hunt_cmd", {
        huntId: caseId,
        targetPath: path && path.length ? path : "DOWNLOADS",
      });
      status = `Exported confidential bundle:\n${out}`;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<main class="container mx-auto max-w-5xl p-8 space-y-6">
  <button onclick={() => goto("/")} class="text-sm text-muted-foreground hover:text-foreground">Back to home</button>

  {#if !profile}
    <p class="text-sm {error ? 'text-destructive' : 'text-muted-foreground'}">{error || "Loading case..."}</p>
  {:else}
    <div class="flex justify-between items-start gap-4">
      <div>
        <p class="text-xs uppercase tracking-widest text-muted-foreground">
          {profile.mode === "confidential" ? "Confidential case" : "Standard case"}
          {#if profile.sealed_at} · sealed{/if}
        </p>
        <h1 class="text-2xl font-bold">{profile.title || "Untitled case"}</h1>
        <p class="text-xs font-mono text-muted-foreground mt-1">{profile.id}</p>
      </div>
      <span class="text-xs px-2 py-1 rounded border {profile.mode === 'confidential' ? 'border-red-500/40 text-red-300' : 'border-border'}">
        {profile.mode}
      </span>
    </div>

    <ReviewNotice />

    {#if profile.sealed_at}
      <p class="text-sm border border-red-500/30 bg-red-500/10 rounded-md p-3">
        This case is sealed. The Documents folder is a pointer only. Exhibit edits go through the unlocked vault and require the vault password.
      </p>
    {/if}

    <nav class="flex flex-wrap gap-2 text-sm">
      {#each [["overview", "Overview"], ["parties", "Parties"], ["documents", "Documents"], ["evidence", "Evidence"], ["timeline", "Timeline"], ["check", "Check"], ["files", "Drafts and exports"]] as [id, label]}
        <button
          onclick={() => (tab = id as typeof tab)}
          class="px-3 py-1.5 rounded-md border {tab === id ? 'bg-primary text-primary-foreground border-transparent' : 'border-border'}"
        >
          {label}
        </button>
      {/each}
      {#if profile.mode === "confidential"}
        <button
          onclick={() => (tab = "confidential")}
          class="px-3 py-1.5 rounded-md border {tab === 'confidential' ? 'bg-red-700 text-white border-transparent' : 'border-red-500/40 text-red-200'}"
        >
          Confidential tools
        </button>
      {/if}
    </nav>

    {#if tab === "overview"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Case profile</h2>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Title</span>
          <input bind:value={profile.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Court name <FieldHelp text="Copy the court name from a paper you already have. Do not guess a court." /></span>
          <input bind:value={profile.court.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Docket number <FieldHelp text="Copy the case number exactly. If you do not have it, leave this blank." /></span>
          <input bind:value={profile.docket_number} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <div class="grid md:grid-cols-2 gap-3">
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
          <input bind:value={profile.filer.role} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Address (one line per row)</span>
          <textarea bind:value={addressText} class="w-full h-20 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
        </label>
        <div class="grid md:grid-cols-2 gap-3">
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
    {:else if tab === "parties"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Parties</h2>
        <PartyEditor bind:parties />
      </section>
    {:else if tab === "documents"}
      {#if guided}
        <section class="space-y-4 bg-card border rounded-lg p-6">
          <h2 class="font-semibold">Write this paper (guided)</h2>
          <p class="text-sm text-muted-foreground">Answer each prompt in your own words. Leave a box blank rather than guessing. The app will not invent facts or citations.</p>
          {#each guided.sections as section}
            <label class="block space-y-1">
              <span class="text-sm font-medium">{section.heading} <FieldHelp text={section.hint} /></span>
              <p class="text-xs text-muted-foreground">{section.hint}</p>
              <textarea
                class="w-full h-24 rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={section.user_text}
                onblur={(e) => fillSection(section.id, (e.currentTarget as HTMLTextAreaElement).value)}
              ></textarea>
            </label>
          {/each}
          <div class="flex flex-wrap gap-2">
            <button onclick={() => exportKind(profile?.document_kind || "motion")} disabled={busy} class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm">Export this paper (Word and PDF)</button>
            <button onclick={exportPacket} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export filing packet</button>
          </div>
        </section>
      {/if}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Notice of Appeal</h2>
        <p class="text-xs text-muted-foreground">OpenSeason will not invent grounds, facts, or citations. Paste only text you wrote or researched.</p>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Trial court</span>
          <input bind:value={profile.notice_of_appeal.trial_court_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <div class="grid md:grid-cols-2 gap-3">
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
          <input bind:value={profile.notice_of_appeal.judgment_description} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Your notice text</span>
          <textarea bind:value={profile.notice_of_appeal.user_text} class="w-full h-36 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
        </label>
        <button onclick={() => exportKind("notice_of_appeal")} disabled={busy} class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm">Export Notice of Appeal (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Complaint</h2>
          <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
        </div>
        <p class="text-xs text-muted-foreground">Caption, parties, and the signature block come from the profile. You write the claim. This is not a full complaint interview.</p>
        <input bind:value={profile.complaint.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Complaint title" />
        <textarea bind:value={profile.complaint.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your statement of the claim"></textarea>
        <button onclick={() => exportKind("complaint")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export complaint (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Motion</h2>
          <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
        </div>
        <p class="text-xs text-muted-foreground">Same fill-once profile. You write the request. This is not a motion library.</p>
        <input bind:value={profile.motion.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Motion title" />
        <textarea bind:value={profile.motion.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your motion text"></textarea>
        <button onclick={() => exportKind("motion")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export motion (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Appellate brief</h2>
          <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic scaffold</span>
        </div>
        <input bind:value={profile.brief.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Brief title" />
        <textarea bind:value={profile.brief.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your brief text. The scaffold will not invent argument."></textarea>
        <button onclick={() => exportKind("appellate_brief")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export brief (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Motion in the appellate court</h2>
          <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
        </div>
        <input bind:value={profile.appellate_motion.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Appellate motion title" />
        <textarea bind:value={profile.appellate_motion.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your R. 2:8-1 motion text"></textarea>
        <button onclick={() => exportKind("appellate_motion")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export appellate motion (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Mandamus (Colorado)</h2>
          <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic C.A.R. 21</span>
        </div>
        <input bind:value={profile.mandamus.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Petition title" />
        <textarea bind:value={profile.mandamus.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your petition text"></textarea>
        <button onclick={() => exportKind("mandamus")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export mandamus (Word and PDF)</button>
      </section>

      <section class="space-y-4 bg-card border rounded-lg p-6">
        <div class="flex justify-between">
          <h2 class="font-semibold">Court Markdown</h2>
          <span class="text-[10px] uppercase tracking-wide text-muted-foreground">Same engine as the CLI</span>
        </div>
        <p class="text-xs text-muted-foreground">Paste an md2court subset draft (YAML front matter, ::: signature, ::: certificate). Import writes drafts/draft.md. Export uses the same renderer as `openseason export court-md`.</p>
        <textarea bind:value={mdDraft} class="w-full h-36 rounded-md border border-input bg-background px-3 py-2 text-sm font-mono" placeholder={"---\ncourt: ...\nplaintiff: ...\ndefendant: ...\ncase_no: ...\n---\n"}></textarea>
        <div class="flex flex-wrap gap-2">
          <button onclick={importMarkdown} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Import Markdown</button>
          <button onclick={() => exportKind("court_md")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export court Markdown (Word and PDF)</button>
        </div>
      </section>
    {:else if tab === "evidence"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Evidence</h2>
        <p class="text-sm text-muted-foreground">
          {#if profile.mode === "confidential"}
            Confidential exhibits are encrypted in the vault. A sealed case asks for the vault password again before a new file is stored.
          {:else}
            Standard exhibits are copied into this case folder. They are not encrypted.
          {/if}
        </p>
        <button onclick={addEvidence} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Add evidence</button>
        {#if artifacts.filter((a) => a.kind === "evidence").length === 0}
          <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No exhibits yet. Add a file to start a record.</p>
        {:else}
          <ul class="text-sm space-y-2">
            {#each artifacts.filter((a) => a.kind === "evidence") as item}
              <li class="border rounded-md p-3">
                <p class="font-medium">{item.name}</p>
                <p class="text-xs font-mono text-muted-foreground">{item.path}</p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {:else if tab === "timeline"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Timeline</h2>
        <div class="grid md:grid-cols-3 gap-3">
          <input bind:value={eventTitle} class="h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Event title" />
          <input type="date" bind:value={eventDate} class="h-10 rounded-md border border-input bg-background px-3 text-sm" />
          <input bind:value={eventNotes} class="h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Notes" />
        </div>
        <button onclick={addTimeline} class="border border-border px-4 py-2 rounded-md text-sm">Add event</button>
        {#if !profile.timeline || profile.timeline.length === 0}
          <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No events yet. Dates you add here stay with the case profile.</p>
        {:else}
          <ul class="text-sm space-y-2">
            {#each profile.timeline as event}
              <li class="border rounded-md p-3 flex justify-between gap-3">
                <div>
                  <p class="font-medium">{event.date} · {event.title}</p>
                  <p class="text-muted-foreground">{event.notes}</p>
                </div>
                <button onclick={() => removeTimeline(event.id)} class="text-xs text-red-400">Remove</button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {:else if tab === "check"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Check and fix</h2>
        <p class="text-sm text-muted-foreground">Issues sit next to the draft. The app never invents a cite and never files or serves. <FieldHelp text="Run checks after you write. Link each fact to Ex. A, Ex. B. You check every sign-off box." /></p>
        <div class="grid md:grid-cols-3 gap-3">
          <label class="block space-y-1">
            <span class="text-xs text-muted-foreground">Rule pack</span>
            <select bind:value={deadlinePack} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm">
              <option value="nj-app-div">New Jersey Appellate Division</option>
              <option value="co-mandamus">Colorado mandamus (C.A.R. 21)</option>
              <option value="generic-us-appellate">Generic US appellate</option>
            </select>
          </label>
          <label class="block space-y-1">
            <span class="text-xs text-muted-foreground">Deadline rule</span>
            <select bind:value={deadlineRule} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm">
              <option value="reconsideration-appellate">R. 2:11-6 (10 days from entry)</option>
              <option value="reconsideration-trial">R. 4:49-2 (20 days from service)</option>
              <option value="motion-appellate">R. 2:8-1 (appellate motion)</option>
              <option value="toll-time-to-appeal">R. 2:4-3(e) (tolling note)</option>
              <option value="car-21-petition">C.A.R. 21 (no automatic day count)</option>
            </select>
          </label>
          <label class="block space-y-1">
            <span class="text-xs text-muted-foreground">Trigger date (YYYY-MM-DD) <FieldHelp text="The date the order was entered or served. Use YYYY-MM-DD. Example: 2026-01-09." /></span>
            <input bind:value={triggerDate} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="2026-01-09" />
          </label>
        </div>
        <div class="flex flex-wrap gap-2">
          <button onclick={runChecks} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Run checks</button>
          <button onclick={exportIcs} class="border border-border px-4 py-2 rounded-md text-sm">Save deadline on calendar (.ics)</button>
          <button onclick={exportPacket} class="border border-border px-4 py-2 rounded-md text-sm">Export filing packet</button>
        </div>
        {#if exhibits.length}
          <p class="text-xs text-muted-foreground">Exhibits: {exhibits.map((e) => `Ex. ${e.letter} ${e.filename}`).join(" · ")}</p>
        {/if}
        {#if checkReport}
          <p class="text-sm">{checkReport.never_files ? "Never files. " : ""}{checkReport.ok ? "Gates passed." : "Issues remain. Fix them below. Do not invent a cite."}</p>
          {#if checkReport.deadline}
            <p class="text-xs font-mono">{checkReport.deadline.rule}: {checkReport.deadline.math}</p>
          {/if}
          {#if checkReport.fact?.rows?.length}
            <ul class="text-sm space-y-3">
              {#each checkReport.fact.rows as row}
                <li class="border rounded-md p-3 space-y-2">
                  <p>{row.id}. {row.assertion}</p>
                  <p class="text-xs text-muted-foreground">{row.record_cite || "No record cite yet."}</p>
                  <div class="flex flex-wrap gap-2">
                    {#each exhibits as ex}
                      <button onclick={() => linkFact(row.id, ex.letter)} class="text-xs border border-border px-2 py-1 rounded-md">Link Ex. {ex.letter}</button>
                    {/each}
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
          {#if checkReport.authority?.rows?.length}
            <ul class="text-sm space-y-1">
              {#each checkReport.authority.rows as row}
                <li>{row.id}. {row.status} {row.cite}</li>
              {/each}
            </ul>
          {/if}
          {#if checkReport.signoff?.items?.length}
            <ul class="text-sm space-y-1">
              {#each checkReport.signoff.items as item}
                <li>
                  <label class="flex gap-2 items-start">
                    <input type="checkbox" checked={item.checked} onchange={(e) => toggleSignoff(item.id, (e.currentTarget as HTMLInputElement).checked)} />
                    <span>{item.label}</span>
                  </label>
                </li>
              {/each}
            </ul>
          {/if}
          {#if checkReport.issues?.length}
            <ul class="text-sm text-destructive list-disc pl-5">
              {#each checkReport.issues as issue}
                <li>{issue.field}: {issue.message}</li>
              {/each}
            </ul>
          {/if}
        {:else}
          <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No check run yet. Write the guided sections or import Markdown, add an exhibit, then run checks.</p>
        {/if}
        <div class="border-t pt-4 space-y-2">
          <h3 class="text-sm font-semibold">Justice-gap note</h3>
          <p class="text-xs text-muted-foreground">What was hard? Fake names only. This is for later content, not the court paper.</p>
          <input bind:value={gapTitle} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Short title" />
          <textarea bind:value={gapHard} class="w-full h-16 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="What was hard"></textarea>
          <button onclick={addGap} class="border border-border px-3 py-1.5 rounded-md text-sm">Save gap note</button>
        </div>
      </section>
    {:else if tab === "files"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Drafts and exports</h2>
        {#if artifacts.length === 0}
          <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">Nothing exported yet. Use the Documents tab to create Word and PDF files.</p>
        {:else}
          <ul class="text-sm space-y-2">
            {#each artifacts as item}
              <li class="border rounded-md p-3">
                <p class="text-[10px] uppercase text-muted-foreground">{item.kind}</p>
                <p class="font-medium">{item.name}</p>
                <p class="text-xs font-mono text-muted-foreground">{item.path}</p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {:else}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Confidential tools</h2>
        <p class="text-sm text-muted-foreground">These used to live on a separate hunt screen. They now belong to this Confidential case. The optional USAspending lookup sends only a target name to a public API. It does not upload the case folder.</p>
        {#if !session.unlocked}
          <button onclick={() => goto("/confidential")} class="border border-red-500/40 text-red-200 px-4 py-2 rounded-md text-sm">Unlock vault</button>
        {:else}
          <div class="flex gap-2">
            <input bind:value={targetLookup} class="flex-1 h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Agency or contractor name" />
            <button onclick={lookupTarget} class="border border-border px-3 rounded-md text-sm">Look up target</button>
          </div>
          {#if targetHits.length}
            <ul class="text-sm space-y-1">
              {#each targetHits as hit}
                <li>{hit.recipient_name || "Unknown"} · {hit.award_amount ?? "n/a"} · {hit.description || ""}</li>
              {/each}
            </ul>
          {/if}
          <div class="flex flex-wrap gap-2">
            <button onclick={disclosure} class="border border-red-500/40 text-red-200 px-4 py-2 rounded-md text-sm">Disclosure statement</button>
            <button onclick={exportOsb} class="border border-border px-4 py-2 rounded-md text-sm">Export .osb</button>
          </div>
        {/if}
      </section>
    {/if}

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
    {#if exportPaths}
      <div class="text-sm border border-emerald-500/30 bg-emerald-500/10 rounded-md p-3 space-y-1">
        <p class="font-medium">Export finished</p>
        <p class="font-mono text-xs">{exportPaths.docx}</p>
        <p class="font-mono text-xs">{exportPaths.pdf}</p>
      </div>
    {/if}

    <div class="flex flex-wrap gap-2 items-end">
      <button onclick={save} disabled={busy || !!profile.sealed_at} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Save profile</button>
      <button onclick={validate} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Validate</button>
      <button onclick={() => trySync("backup")} class="border border-border px-4 py-2 rounded-md text-sm">Account backup (stub)</button>
      {#if profile.mode === "standard"}
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Vault password to convert</span>
          <input type="password" bind:value={sealPassword} class="h-10 rounded-md border border-input bg-background px-3 text-sm" autocomplete="off" />
        </label>
        <button onclick={seal} disabled={busy} class="border border-red-500/40 text-red-300 px-4 py-2 rounded-md text-sm">Convert to Confidential</button>
      {/if}
    </div>
    {#if syncResult}
      <p class="text-xs text-muted-foreground">{syncResult.kind}: {syncResult.reason}</p>
    {/if}
  {/if}
</main>
