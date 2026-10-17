<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";

  interface Props {
    caseId: string;
    isOpen?: boolean;
    onClose?: () => void;
  }

  let { caseId, isOpen = $bindable(false), onClose }: Props = $props();

  let activeStep = $state<1 | 2 | 3 | 4>(1);
  let loading = $state(false);
  let error = $state("");
  let successMsg = $state("");

  // Step 1: Entity Verification
  let entityQuery = $state("");
  let entityState = $state("");
  let entityReport = $state<any>(null);

  // Step 2: Records Pack
  let recordsPack = $state<any>(null);

  // Step 3: Observation Log
  let obsFilePath = $state("");
  let obsNotes = $state("");
  let observerName = $state("Relator / Citizen Investigator");
  let observationsList = $state<any[]>([]);

  // Step 4: Rule 9(b) Audit & Export
  let rule9bAudit = $state<any>(null);
  let exportedPacket = $state<any>(null);
  let eoCategory = $state("Provider Enrollments & Ghost Clinics");

  onMount(async () => {
    if (caseId && isOpen) {
      await loadInitialData();
    }
  });

  async function loadInitialData() {
    loading = true;
    error = "";
    try {
      rule9bAudit = await ipc("quitam_audit_9b_cmd", { caseId }).catch(() => null);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleVerifyEntity() {
    if (!entityQuery.trim()) return;
    loading = true;
    error = "";
    try {
      entityReport = await ipc("quitam_verify_entity_cmd", {
        query: entityQuery.trim(),
        state: entityState.trim() ? entityState.trim() : null,
      });
      successMsg = "Entity verified across CMS NPPES and USAspending!";
      setTimeout(() => (successMsg = ""), 4000);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleGenerateRecordsPack() {
    loading = true;
    error = "";
    try {
      recordsPack = await ipc("quitam_records_pack_cmd", {
        caseId,
        state: entityState.trim() ? entityState.trim() : null,
      });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleIngestObservation() {
    if (!obsFilePath.trim()) {
      error = "Please provide an observational photo or document path.";
      return;
    }
    loading = true;
    error = "";
    try {
      const entry = await ipc("quitam_ingest_observation_cmd", {
        caseId,
        filePath: obsFilePath.trim(),
        notes: obsNotes.trim(),
        observer: observerName.trim() ? observerName.trim() : null,
      });
      observationsList = [entry, ...observationsList];
      obsFilePath = "";
      obsNotes = "";
      successMsg = "Observation cryptographically hashed and logged to provenance.json!";
      setTimeout(() => (successMsg = ""), 4000);
      // Re-run audit
      rule9bAudit = await ipc("quitam_audit_9b_cmd", { caseId }).catch(() => null);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleExportDisclosure() {
    loading = true;
    error = "";
    try {
      exportedPacket = await ipc("quitam_export_disclosure_cmd", {
        caseId,
        category: eoCategory,
        attestation: null,
      });
      successMsg = "DOJ 60-Day Relator Disclosure Packet compiled under seal!";
      setTimeout(() => (successMsg = ""), 5000);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    isOpen = false;
    if (onClose) onClose();
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-background/80 backdrop-blur-md">
    <div
      class="relative w-full max-w-4xl max-h-[90vh] flex flex-col rounded-3xl border border-amber-500/30 bg-card p-6 md:p-8 shadow-2xl text-foreground animate-in fade-in zoom-in-95 duration-200 overflow-hidden"
    >
      <!-- Header -->
      <div class="flex items-start justify-between gap-4 border-b border-border/60 pb-4 shrink-0">
        <div class="flex items-center gap-3">
          <div class="p-3 rounded-2xl bg-amber-500/10 text-amber-400 border border-amber-500/20 text-2xl shrink-0">
            👻
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-lg font-bold text-foreground">Ghost Facility & Shell Entity Verification Wizard</h2>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase font-semibold">
                EO 14395 Mandate
              </span>
            </div>
            <p class="text-xs text-muted-foreground mt-0.5">
              Automates preliminary federal OIG triage: NPI verification, licensing FOIA requests, and Rule 9(b) disclosure packets.
            </p>
          </div>
        </div>
        <button
          onclick={handleClose}
          class="text-muted-foreground hover:text-foreground p-1 text-sm rounded-lg"
          aria-label="Close"
        >
          ✕
        </button>
      </div>

      <!-- Step Navigation Tabs -->
      <div class="grid grid-cols-4 gap-2 pt-4 pb-2 border-b border-border/40 shrink-0">
        <button
          onclick={() => (activeStep = 1)}
          class="px-3 py-2 rounded-xl text-xs font-semibold transition-all text-left flex items-center gap-2 {activeStep === 1 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40 shadow-sm' : 'text-muted-foreground hover:bg-secondary/40'}"
        >
          <span class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold {activeStep === 1 ? 'bg-amber-400 text-black' : 'bg-secondary text-muted-foreground'}">1</span>
          <span class="truncate">Identity Lock</span>
        </button>

        <button
          onclick={() => {
            activeStep = 2;
            if (!recordsPack) handleGenerateRecordsPack();
          }}
          class="px-3 py-2 rounded-xl text-xs font-semibold transition-all text-left flex items-center gap-2 {activeStep === 2 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40 shadow-sm' : 'text-muted-foreground hover:bg-secondary/40'}"
        >
          <span class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold {activeStep === 2 ? 'bg-amber-400 text-black' : 'bg-secondary text-muted-foreground'}">2</span>
          <span class="truncate">Licensing FOIA</span>
        </button>

        <button
          onclick={() => (activeStep = 3)}
          class="px-3 py-2 rounded-xl text-xs font-semibold transition-all text-left flex items-center gap-2 {activeStep === 3 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40 shadow-sm' : 'text-muted-foreground hover:bg-secondary/40'}"
        >
          <span class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold {activeStep === 3 ? 'bg-amber-400 text-black' : 'bg-secondary text-muted-foreground'}">3</span>
          <span class="truncate">Observation Log</span>
        </button>

        <button
          onclick={() => {
            activeStep = 4;
            loadInitialData();
          }}
          class="px-3 py-2 rounded-xl text-xs font-semibold transition-all text-left flex items-center gap-2 {activeStep === 4 ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40 shadow-sm' : 'text-muted-foreground hover:bg-secondary/40'}"
        >
          <span class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold {activeStep === 4 ? 'bg-amber-400 text-black' : 'bg-secondary text-muted-foreground'}">4</span>
          <span class="truncate">Rule 9(b) & Dossier</span>
        </button>
      </div>

      <!-- Notifications -->
      {#if successMsg}
        <div class="p-3 my-2 rounded-xl bg-green-500/10 border border-green-500/30 text-green-400 text-xs flex items-center gap-2">
          <span>✓</span>
          <span>{successMsg}</span>
        </div>
      {/if}
      {#if error}
        <div class="p-3 my-2 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400 text-xs flex items-center gap-2">
          <span>⚠️</span>
          <span>{error}</span>
        </div>
      {/if}

      <!-- Step Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto py-4 space-y-4 pr-1 text-xs">
        <!-- STEP 1: IDENTITY LOCK -->
        {#if activeStep === 1}
          <div class="space-y-4">
            <div class="p-4 rounded-2xl bg-secondary/30 border border-border/70 space-y-3">
              <h4 class="font-bold text-sm text-foreground">Step 1: Cross-Check CMS NPI & USAspending Obligations</h4>
              <p class="text-muted-foreground">
                Enter the suspect clinic name, address, or 10-digit NPI. The engine queries the live CMS NPPES v2.1 registry to reveal registered taxonomy (physical clinic vs dispatch) and USAspending federal award obligations.
              </p>
              <div class="flex flex-col sm:flex-row gap-2 pt-1">
                <input
                  type="text"
                  bind:value={entityQuery}
                  placeholder="Target clinic name, officer name, or 10-digit NPI"
                  class="flex-1 px-3.5 py-2.5 rounded-xl border border-border bg-background text-foreground text-xs placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
                <input
                  type="text"
                  bind:value={entityState}
                  placeholder="State (e.g. MN, NJ)"
                  class="w-24 px-3.5 py-2.5 rounded-xl border border-border bg-background text-foreground text-xs uppercase placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
                />
                <button
                  type="button"
                  onclick={handleVerifyEntity}
                  disabled={loading || !entityQuery.trim()}
                  class="px-5 py-2.5 bg-primary text-primary-foreground font-semibold rounded-xl hover:opacity-90 disabled:opacity-50 transition-opacity"
                >
                  {loading ? "Verifying..." : "Run Cross-Check"}
                </button>
              </div>
            </div>

            {#if entityReport}
              <div class="p-4 rounded-2xl bg-background/60 border border-border space-y-3">
                <div class="flex items-center justify-between border-b border-border/40 pb-2">
                  <div>
                    <h5 class="font-bold text-sm text-foreground">{entityReport.target_query}</h5>
                    <p class="text-[11px] text-muted-foreground font-mono">
                      NPI: {entityReport.npi_matched || "Not Found"} · Verified at {entityReport.verified_at}
                    </p>
                  </div>
                  <span class="px-2.5 py-1 rounded-full text-[10px] font-bold bg-primary/10 text-primary border border-primary/20">
                    ${entityReport.total_federal_obligations.toLocaleString()} Federal Obligations
                  </span>
                </div>

                {#if entityReport.nppes_entity}
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                    <div class="p-3 rounded-xl bg-secondary/30 border border-border/50 space-y-1">
                      <span class="text-[10px] uppercase font-bold text-muted-foreground">Practice Location</span>
                      <p class="font-medium text-foreground">
                        {entityReport.nppes_entity.practice_location.address_1} {entityReport.nppes_entity.practice_location.address_2 || ""}
                      </p>
                      <p class="text-muted-foreground">
                        {entityReport.nppes_entity.practice_location.city}, {entityReport.nppes_entity.practice_location.state} {entityReport.nppes_entity.practice_location.postal_code}
                      </p>
                    </div>

                    <div class="p-3 rounded-xl bg-secondary/30 border border-border/50 space-y-1">
                      <span class="text-[10px] uppercase font-bold text-muted-foreground">Mailing Address</span>
                      <p class="font-medium text-foreground">
                        {entityReport.nppes_entity.mailing_address.address_1}
                      </p>
                      <p class="text-muted-foreground">
                        {entityReport.nppes_entity.mailing_address.city}, {entityReport.nppes_entity.mailing_address.state} {entityReport.nppes_entity.mailing_address.postal_code}
                      </p>
                    </div>
                  </div>

                  <!-- Taxonomies -->
                  <div class="space-y-1">
                    <span class="text-[10px] font-bold uppercase text-muted-foreground">Registered Healthcare Taxonomies</span>
                    <div class="flex flex-wrap gap-1.5">
                      {#each entityReport.nppes_entity.taxonomies as tax}
                        <span class="px-2 py-0.5 rounded bg-secondary text-foreground text-[11px] border border-border">
                          {tax.code}: {tax.desc} {tax.primary ? "⭐ (Primary)" : ""}
                        </span>
                      {/each}
                    </div>
                  </div>
                {/if}

                <!-- Red Flags -->
                {#if entityReport.red_flags.length > 0}
                  <div class="p-3 rounded-xl bg-red-500/10 border border-red-500/30 space-y-1 text-red-300">
                    <span class="text-[10px] font-bold uppercase tracking-wider text-red-400">Automated Regulatory Red Flags:</span>
                    <ul class="list-disc list-inside space-y-0.5 text-[11px]">
                      {#each entityReport.red_flags as rf}
                        <li>{rf}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}
              </div>
            {/if}
          </div>

        <!-- STEP 2: LICENSING FOIA -->
        {:else if activeStep === 2}
          <div class="space-y-4">
            <div class="p-4 rounded-2xl bg-secondary/30 border border-border/70 space-y-2">
              <h4 class="font-bold text-sm text-foreground">Step 2: Generate State Licensing & Fire Inspection Requests</h4>
              <p class="text-muted-foreground">
                Legally obtain the smoking-gun government records (initial facility floor plans, approved bed capacity, fire occupancy inspections) that prove false certification without trespassing.
              </p>
            </div>

            {#if recordsPack}
              <div class="p-4 rounded-2xl bg-background/60 border border-border space-y-3">
                <div class="flex items-center justify-between border-b border-border/40 pb-2">
                  <span class="font-bold text-foreground">{recordsPack.statute_title}</span>
                  <span class="text-[11px] font-mono text-muted-foreground">Target Agency: {recordsPack.target_agency}</span>
                </div>

                <div class="space-y-2">
                  <span class="text-[10px] font-bold uppercase text-muted-foreground">Records Included in Request:</span>
                  <ul class="space-y-1 text-[11px] text-muted-foreground">
                    {#each recordsPack.records_items as item}
                      <li class="p-2 rounded-lg bg-secondary/40 border border-border/50">{item}</li>
                    {/each}
                  </ul>
                </div>

                <div class="space-y-1 pt-2">
                  <span class="text-[10px] font-bold uppercase text-muted-foreground">Formal Request Letter Text:</span>
                  <pre class="p-3 rounded-xl bg-card border border-border/60 text-[10px] font-mono text-foreground whitespace-pre-wrap max-h-48 overflow-y-auto">{recordsPack.formal_letter_text}</pre>
                </div>
              </div>
            {/if}
          </div>

        <!-- STEP 3: OBSERVATION LOG -->
        {:else if activeStep === 3}
          <div class="space-y-4">
            <div class="p-4 rounded-2xl bg-secondary/30 border border-border/70 space-y-3">
              <h4 class="font-bold text-sm text-foreground">Step 3: Cryptographic Field Observation Ingestion</h4>
              <p class="text-muted-foreground">
                Ingest photos of locked clinic doors, dark windows, or empty parking lots taken during stated operating hours. The engine extracts EXIF GPS/timestamps and locks an immutable SHA-256 chain of custody in provenance.json.
              </p>

              <div class="space-y-2 pt-1">
                <div>
                  <label class="text-[11px] font-semibold text-foreground">Photo / Document File Path</label>
                  <input
                    type="text"
                    bind:value={obsFilePath}
                    placeholder="e.g. C:/Photos/clinic_tuesday_closed.jpg"
                    class="w-full mt-1 px-3.5 py-2.5 rounded-xl border border-border bg-background text-foreground text-xs"
                  />
                </div>

                <div>
                  <label class="text-[11px] font-semibold text-foreground">Observation Notes (Observed Conditions)</label>
                  <input
                    type="text"
                    bind:value={obsNotes}
                    placeholder="e.g. Front door locked, lights out at 2:15 PM on Tuesday during required hours."
                    class="w-full mt-1 px-3.5 py-2.5 rounded-xl border border-border bg-background text-foreground text-xs"
                  />
                </div>

                <div class="flex justify-end pt-1">
                  <button
                    type="button"
                    onclick={handleIngestObservation}
                    disabled={loading || !obsFilePath.trim()}
                    class="px-5 py-2.5 bg-amber-500 hover:bg-amber-600 text-black font-semibold rounded-xl text-xs transition-colors"
                  >
                    Ingest & Lock SHA-256 Hash
                  </button>
                </div>
              </div>
            </div>

            <!-- Log Entries -->
            {#if observationsList.length > 0}
              <div class="space-y-2">
                <span class="text-[10px] font-bold uppercase text-muted-foreground">Chain of Custody Manifest ({observationsList.length} items):</span>
                {#each observationsList as obs}
                  <div class="p-3 rounded-xl bg-background/60 border border-border flex items-center justify-between gap-3">
                    <div class="space-y-0.5 truncate">
                      <div class="font-semibold text-foreground truncate">{obs.filename}</div>
                      <div class="text-[10px] font-mono text-muted-foreground truncate">SHA-256: {obs.sha256}</div>
                      <div class="text-[11px] text-muted-foreground">{obs.notes}</div>
                    </div>
                    <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-green-500/10 text-green-400 border border-green-500/20 shrink-0">
                      Locked Read-Only
                    </span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

        <!-- STEP 4: RULE 9(b) & DOSSIER EXPORT -->
        {:else if activeStep === 4}
          <div class="space-y-4">
            {#if rule9bAudit}
              <div class="p-4 rounded-2xl bg-secondary/30 border border-border/70 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="font-bold text-sm text-foreground">Step 4: FRCP Rule 9(b) Viability Evaluation</h4>
                    <p class="text-xs text-muted-foreground">Statutory readiness for DOJ 60-Day Review under EO 14395</p>
                  </div>
                  <div class="text-right">
                    <div class="text-2xl font-black text-primary">{rule9bAudit.score}/100</div>
                    <span class="text-[10px] font-mono uppercase font-semibold text-muted-foreground">{rule9bAudit.viability_tier}</span>
                  </div>
                </div>

                <div class="space-y-2 pt-2">
                  {#each rule9bAudit.elements as elem}
                    <div class="p-2.5 rounded-xl border border-border/60 bg-background/50 flex items-start justify-between gap-3">
                      <div>
                        <div class="font-semibold text-xs text-foreground">{elem.title}</div>
                        <div class="text-[10px] font-mono text-muted-foreground">{elem.statutory_reference}</div>
                        <div class="text-[11px] text-muted-foreground mt-0.5">{elem.details}</div>
                      </div>
                      <span class="px-2 py-0.5 rounded text-[10px] font-bold shrink-0 {elem.satisfied ? 'bg-green-500/10 text-green-400 border border-green-500/20' : 'bg-red-500/10 text-red-400 border border-red-500/20'}">
                        {elem.satisfied ? "PASS" : "DEFICIENT"}
                      </span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Export Card -->
            <div class="p-4 rounded-2xl bg-amber-500/10 border border-amber-500/30 space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="font-bold text-xs text-amber-300">Compile DOJ 60-Day Triage & Relator Disclosure Packet</h4>
                  <p class="text-[11px] text-muted-foreground">Generates openseason-disclosure-v1.json + court-ready Typst PDF under mandatory seal.</p>
                </div>
                <button
                  type="button"
                  onclick={handleExportDisclosure}
                  disabled={loading}
                  class="px-5 py-2.5 bg-primary text-primary-foreground font-semibold rounded-xl text-xs hover:opacity-90 shadow-md transition-opacity"
                >
                  {loading ? "Compiling Packet..." : "Export Official Disclosure Packet 📄"}
                </button>
              </div>

              {#if exportedPacket}
                <div class="p-3 rounded-xl bg-background/80 border border-border space-y-1 text-xs">
                  <div class="text-green-400 font-semibold">✓ Relator Disclosure Packet Compiled Successfully:</div>
                  <div class="font-mono text-[11px] text-foreground truncate">PDF: {exportedPacket.pdf_export_path}</div>
                  <div class="font-mono text-[11px] text-foreground truncate">JSON: {exportedPacket.json_manifest_path}</div>
                  <div class="text-[10px] text-red-400 font-semibold pt-1">
                    ⚠️ Mandatory Notice: Keep this file under seal. Do not upload to third-party public cloud servers.
                  </div>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer Navigation -->
      <div class="flex items-center justify-between border-t border-border/60 pt-4 shrink-0">
        <button
          type="button"
          disabled={activeStep === 1}
          onclick={() => (activeStep = Math.max(1, activeStep - 1) as any)}
          class="px-4 py-2 rounded-xl border border-border text-xs text-foreground hover:bg-secondary/40 disabled:opacity-30 transition-colors"
        >
          ← Previous
        </button>

        <div class="text-xs font-mono text-muted-foreground">
          Step {activeStep} of 4
        </div>

        {#if activeStep < 4}
          <button
            type="button"
            onclick={() => (activeStep = Math.min(4, activeStep + 1) as any)}
            class="px-4 py-2 rounded-xl bg-primary text-primary-foreground text-xs font-semibold hover:opacity-90 transition-opacity"
          >
            Next →
          </button>
        {:else}
          <button
            type="button"
            onclick={handleClose}
            class="px-4 py-2 rounded-xl bg-secondary text-foreground text-xs font-semibold hover:bg-secondary/80 transition-colors"
          >
            Done
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
