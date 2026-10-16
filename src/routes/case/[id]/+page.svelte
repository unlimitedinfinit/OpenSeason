<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import PartyEditor from "$lib/components/PartyEditor.svelte";
  import FieldHelp from "$lib/components/FieldHelp.svelte";
  import PreFlightAuditCard from "$lib/components/PreFlightAuditCard.svelte";
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import MotionDocket from "$lib/components/MotionDocket.svelte";
  import ServiceCertCard from "$lib/components/ServiceCertCard.svelte";
  import PdfToolsModal from "$lib/components/PdfToolsModal.svelte";
  import SignatureStudioModal from "$lib/components/SignatureStudioModal.svelte";
  import IntakeWizardModal from "$lib/components/IntakeWizardModal.svelte";
  import ProceduralGraphCard from "$lib/components/ProceduralGraphCard.svelte";
  import CaseOverviewTab from "$lib/components/tabs/CaseOverviewTab.svelte";
  import CaseDocumentsTab from "$lib/components/tabs/CaseDocumentsTab.svelte";
  import CaseEvidenceTab from "$lib/components/tabs/CaseEvidenceTab.svelte";
  import CaseTimelineTab from "$lib/components/tabs/CaseTimelineTab.svelte";
  import CaseCheckTab from "$lib/components/tabs/CaseCheckTab.svelte";
  import CaseFilesTab from "$lib/components/tabs/CaseFilesTab.svelte";
  import CaseConfidentialTab from "$lib/components/tabs/CaseConfidentialTab.svelte";
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
    ExhibitPacketReport,
    FormInspection,
    FormFillResult,
    CaseFilingItem,
  } from "$lib/types/case";

  let caseId = $derived($page.params.id || "");
  let profile = $state<CaseProfile | null>(null);
  let parties = $state<Party[]>([]);
  let addressText = $state("");
  let tab = $state<"overview" | "docket" | "service" | "parties" | "documents" | "evidence" | "timeline" | "files" | "check" | "confidential">("overview");
  let showCloudModal = $state(false);
  let showPdfToolsModal = $state(false);
  let showSignatureModal = $state(false);
  let showIntakeWizard = $state(false);
  let activeFilingTitle = $state("");
  let exhibitPacketReport = $state<ExhibitPacketReport | null>(null);
  let batesPrefix = $state("");
  let batesStart = $state(1);
  let assemblingPacket = $state(false);
  let selectedFormPath = $state("");
  let formInspection = $state<FormInspection | null>(null);
  let formFillResult = $state<FormFillResult | null>(null);
  let fillingForm = $state(false);
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

  async function generateExhibitPacket() {
    assemblingPacket = true;
    error = "";
    try {
      exhibitPacketReport = await ipc<ExhibitPacketReport>("assemble_exhibit_packet_cmd", {
        caseId,
        prefix: batesPrefix.trim() || null,
        startNumber: batesStart || 1,
      });
      status = `Exhibit packet assembled!\nTable: ${exhibitPacketReport.table_of_exhibits_pdf}\nPacket: ${exhibitPacketReport.exhibit_packet_pdf}\nTotal pages: ${exhibitPacketReport.total_pages}`;
      artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => artifacts);
    } catch (e) {
      error = String(e);
    } finally {
      assemblingPacket = false;
    }
  }

  async function inspectSelectedForm(formPath: string) {
    selectedFormPath = formPath;
    error = "";
    try {
      formInspection = await ipc<FormInspection>("inspect_court_form_cmd", {
        formPath,
      });
      status = `Inspected form: ${formInspection.detected_form_number || formInspection.form_name} (${formInspection.fields.length} fields)`;
    } catch (e) {
      error = String(e);
    }
  }

  async function fillSelectedForm(formPath: string) {
    fillingForm = true;
    error = "";
    try {
      formFillResult = await ipc<FormFillResult>("fill_court_form_cmd", {
        caseId,
        formPath,
      });
      status = `Filled form generated!\nPDF: ${formFillResult.output_pdf_path}\nJSON: ${formFillResult.output_json_path}\nMapped fields: ${formFillResult.mapped_fields}`;
      artifacts = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => artifacts);
    } catch (e) {
      error = String(e);
    } finally {
      fillingForm = false;
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
      <div class="flex items-center gap-2">
        <button
          onclick={() => (showIntakeWizard = true)}
          class="text-xs px-3 py-1.5 rounded-md border border-amber-500/40 text-amber-300 hover:bg-amber-950/30 flex items-center gap-1.5 font-medium transition-colors"
        >
          <span>🧙 Guided Wizard</span>
        </button>
        <button
          onclick={() => (showPdfToolsModal = true)}
          class="text-xs px-3 py-1.5 rounded-md border border-blue-500/40 text-blue-300 hover:bg-blue-950/30 flex items-center gap-1.5 font-medium transition-colors"
        >
          <span>🛠️ PDF Tools</span>
        </button>
        <button
          onclick={() => (showSignatureModal = true)}
          class="text-xs px-3 py-1.5 rounded-md border border-emerald-500/40 text-emerald-300 hover:bg-emerald-950/30 flex items-center gap-1.5 font-medium transition-colors"
        >
          <span>✍️ Signature</span>
        </button>
        {#if profile.cloud_matter_id}
          <span class="text-xs bg-indigo-500/10 text-indigo-400 border border-indigo-500/30 px-2.5 py-1 rounded-md font-mono">
            ☁ Synced ({profile.cloud_matter_id.slice(0, 8)})
          </span>
        {/if}
        {#if profile.mode !== "confidential"}
          <button
            onclick={() => (showCloudModal = true)}
            class="text-xs px-3 py-1.5 rounded-md bg-primary text-primary-foreground hover:opacity-90 flex items-center gap-1.5 font-medium shadow-sm"
          >
            <span>☁ JustLegal Cloud</span>
          </button>
        {:else}
          <span class="text-xs text-red-400 border border-red-500/30 px-2 py-1 rounded bg-red-500/10">
            Airgapped (Confidential)
          </span>
        {/if}
        <span class="text-xs px-2 py-1 rounded border {profile.mode === 'confidential' ? 'border-red-500/40 text-red-300' : 'border-border'}">
          {profile.mode}
        </span>
      </div>
    </div>

    <ReviewNotice />

    <ProceduralGraphCard
      caseId={caseId}
      bind:profile
      onOpenWizard={() => (showIntakeWizard = true)}
      onProfileSaved={load}
    />

    <PreFlightAuditCard caseId={caseId} />

    {#if profile.sealed_at}
      <p class="text-sm border border-red-500/30 bg-red-500/10 rounded-md p-3">
        This case is sealed. The Documents folder is a pointer only. Exhibit edits go through the unlocked vault and require the vault password.
      </p>
    {/if}

    <nav class="flex flex-wrap gap-2 text-sm">
      {#each [
        ["overview", "Overview"],
        ["docket", "📑 Motions & Filings"],
        ["service", "📜 Service & Orders"],
        ["parties", "Parties"],
        ["documents", "Documents"],
        ["evidence", "Evidence"],
        ["timeline", "Timeline"],
        ["check", "Check"],
        ["files", "Drafts and exports"]
      ] as [id, label]}
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
      <CaseOverviewTab bind:profile bind:addressText />
    {:else if tab === "docket"}
      <section class="space-y-4">
        <MotionDocket
          caseId={caseId}
          onselectmotion={(m) => { activeFilingTitle = m.title; }}
          onexport={(paths) => { exportPaths = { docx: paths.docx, pdf: paths.pdf, notice: "Review every line before filing." }; status = `Exported ${paths.pdf}`; }}
        />
      </section>
    {:else if tab === "service"}
      <section class="space-y-4">
        <ServiceCertCard
          caseId={caseId}
          profile={profile}
          defaultMotionTitle={activeFilingTitle || profile.title}
        />
      </section>
    {:else if tab === "parties"}
      <section class="space-y-4 bg-card border rounded-lg p-6">
        <h2 class="font-semibold">Parties</h2>
        <PartyEditor bind:parties />
      </section>
    {:else if tab === "documents"}
      <CaseDocumentsTab
        bind:profile
        guided={guided}
        artifacts={artifacts}
        busy={busy}
        bind:mdDraft
        bind:selectedFormPath
        bind:formInspection
        bind:formFillResult
        fillingForm={fillingForm}
        onfillsection={fillSection}
        onexportkind={exportKind}
        onexportpacket={exportPacket}
        onimportmarkdown={importMarkdown}
        oninspectform={inspectSelectedForm}
        onfillform={fillSelectedForm}
      />
    {:else if tab === "evidence"}
      <CaseEvidenceTab
        profile={profile}
        artifacts={artifacts}
        bind:assemblingPacket
        bind:batesPrefix
        bind:batesStart
        bind:exhibitPacketReport
        onaddevidence={addEvidence}
        ongeneratepacket={generateExhibitPacket}
      />
    {:else if tab === "timeline"}
      <CaseTimelineTab
        bind:profile
        bind:eventTitle
        bind:eventDate
        bind:eventNotes
        onaddevent={addTimeline}
        onremoveevent={removeTimeline}
      />
    {:else if tab === "check"}
      <CaseCheckTab
        bind:deadlinePack
        bind:deadlineRule
        bind:triggerDate
        bind:gapTitle
        bind:gapHard
        exhibits={exhibits}
        checkReport={checkReport}
        onrunchecks={runChecks}
        onexportics={exportIcs}
        onexportpacket={exportPacket}
        onlinkfact={linkFact}
        ontogglesignoff={toggleSignoff}
        onaddgap={addGap}
      />
    {:else if tab === "files"}
      <CaseFilesTab artifacts={artifacts} />
    {:else}
      <CaseConfidentialTab
        sessionUnlocked={session.unlocked}
        bind:targetLookup
        targetHits={targetHits}
        onlookuptarget={lookupTarget}
        ondisclosure={disclosure}
        onexportosb={exportOsb}
      />
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

    <CloudSyncModal
      bind:open={showCloudModal}
      activeCaseId={caseId}
      activeCaseTitle={profile?.title}
      isConfidential={profile?.mode === "confidential"}
      onsynced={load}
    />

    <PdfToolsModal
      bind:open={showPdfToolsModal}
      caseId={caseId}
      onclose={() => (showPdfToolsModal = false)}
    />

    <SignatureStudioModal
      bind:open={showSignatureModal}
      caseId={caseId}
      onclose={() => (showSignatureModal = false)}
    />

    <IntakeWizardModal
      bind:open={showIntakeWizard}
      caseId={caseId}
      bind:profile
      onUpdated={load}
    />
  {/if}
</main>
