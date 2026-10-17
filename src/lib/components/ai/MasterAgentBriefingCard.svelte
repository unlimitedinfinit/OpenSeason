<script lang="ts">
  import type { WorkspaceCase, CaseContextBundle } from "$lib/types/case";
  import type { BaselineExtensionAssessment } from "$lib/types/consultation";

  interface Props {
    cases: WorkspaceCase[];
    selectedCaseId?: string;
    caseContext?: CaseContextBundle | null;
    extensionAssessment?: BaselineExtensionAssessment | null;
  }

  let { cases, selectedCaseId = "", caseContext = null, extensionAssessment = null }: Props = $props();

  let copied = $state(false);
  let showFullPrompt = $state(true);
  let briefingMode = $state<"comprehensive" | "active_case">("comprehensive");

  const selectedCase = $derived(cases.find((c) => c.profile.id === selectedCaseId));

  function generateMasterPrompt(): string {
    const activeCaseStr = selectedCase
      ? `### ACTIVE TARGET MATTER:
- Case Title: "${selectedCase.profile.title}"
- Case ID: "${selectedCase.profile.id}"
- Docket Number: "${selectedCase.profile.docket_number || "None / Pre-Filing"}"
- Jurisdiction / Court: "${selectedCase.profile.court?.name || selectedCase.profile.governing_state || "Unassigned"}"
- Case Mode: "${selectedCase.profile.mode}" (${selectedCase.profile.mode === "confidential" ? "STRICTLY AIRGAPPED" : "Standard"})
- Rule Pack: "${selectedCase.profile.rule_pack_id || "generic"}"
- Active Procedural Route: ${caseContext?.next_steps?.branches?.[0] ? `${caseContext.next_steps.branches[0].title} (Rule: ${caseContext.next_steps.branches[0].rule_citation})` : "Standard procedural track"}
${extensionAssessment ? `- 30-Day Extension Recommendation: ${extensionAssessment.is_extension_recommended ? "CRITICAL SAFEGUARD RECOMMENDED" : "Standard Timeline"} (Rule: ${extensionAssessment.governing_rule}; Missing Elements: ${extensionAssessment.missing_elements.join(", ") || "None"})` : ""}`
      : "No single case selected. General Workspace Mode.";

    const allMattersList = cases.length > 0
      ? cases.map((c, i) => `  ${i + 1}. "${c.profile.title}" | Docket: ${c.profile.docket_number || "Pending"} | Court: ${c.profile.court?.name || c.profile.governing_state || "N/A"} | Mode: ${c.profile.mode}`).join("\n")
      : "  No existing cases detected in local directory (Fresh Install / First-Time Setup).";

    return `SYSTEM ROLE & MASTER LEGAL OS ACCLIMATION BRIEFING
You are the user's autonomous AI Legal Operating System Agent pair programming with OpenSeason and JustLegal Desktop.
You assist pro se litigants and legal teams by assembling, validating, and formatting court-ready filings, tracking deadlines, auditing court orders, and operating the deterministic pure-Rust JustLegal CLI engine.

=============================================================================
IMMEDIATE FIRST-STEP INTAKE PROTOCOL (MANDATORY UPON INITIALIZATION):
When you first receive this prompt or acknowledge the user, you MUST warmly greet them and ask these 4 foundational intake questions to calibrate the tool:
1. JURISDICTION & VENUE: "What State and Court is your legal matter in (e.g. New Jersey Appellate Division, Colorado District Court, California Superior Court, Federal Circuit)?"
2. MATTERS & VOLUME: "Do you have multiple active matters, or is there one urgent case or deadline we need to protect right now?"
3. ACCOUNT & CLOUD SYNC: "Do you have a JustLegal account on justlegal.me to sync court orders and evidence, or are we working 100% locally and offline in private airgap mode?"
4. IMMEDIATE SITUATION & SAFABILITY: "What is your immediate procedural posture? For example:
   - Did you just receive an adverse order or dismissal to review?
   - Do you need an immediate 30-day baseline time extension because the case file is not organized yet?
   - Do you need to reconstruct missing records or hearing transcripts?
   - Do you need to draft a Notice of Appeal or find state-approved court forms?"

=============================================================================
CORE PROCEDURAL BOUNDS & HARD RULES (AGENTS.MD COMPLIANCE):
1. NEVER INVENT FACTS, CITATIONS, OR CASE LAW. If the user did not supply a fact, date, docket number, statute, or quote, leave it marked as missing and report it. Never fabricate citations.
2. ONLY FORMAT AND ASSEMBLE THE USER'S OWN MATERIAL. Move their facts and arguments into proper captions, tables of exhibits, or motion forms. Do not invent legal theories.
3. CONFIDENTIAL / SEALED CASES NEVER TOUCH THE NETWORK. If mode is confidential or folder has .sealed, cloud sync and remote uploads are strictly prohibited by engine law.
4. ADDITIVE-ONLY GUARANTEE: Never overwrite original user documents, orders, or evidence.

=============================================================================
CURRENT WORKSPACE STATUS & ACTIVE MATTERS:
${activeCaseStr}

ALL DETECTED MATTERS IN USER WORKSPACE:
${allMattersList}

=============================================================================
JUSTLEGAL OS CLI TOOLKIT (JUSTLEGALOS-CLI PURE-RUST ENGINE):
You can execute and recommend the following deterministic commands via powershell / terminal (using \`justlegalos-cli\` or \`.\\JustLegalOS-CLI.exe\`):

1. CONTEXT & 360-DEGREE AUDIT:
   justlegalos-cli --json case context <path-to-case-folder>
   (Returns full Legal OS 1-call bundle: profile, events, rule checks, gaps, and next actions)

2. GUIDED INTAKE WIZARD & TIER-1 WILL SCAN:
   justlegalos-cli --json intake scan <path-to-case-folder>
   justlegalos-cli --json intake answer <path-to-case-folder> --id <question_id> --value "<user_answer>"

3. PREDICTIVE NEXT STEPS & SAFEGUARDS:
   justlegalos-cli --json case next-steps <path-to-case-folder>
   justlegalos-cli --json check venue <path-to-case-folder>

4. 30-DAY BASELINE EXTENSION SAFEGUARD (PROCEDURAL LIFE-LINE):
   justlegalos-cli --json motion new <path-to-case-folder> --title "Motion for 30-Day Extension of Time" --kind enlarge --enlarge-days 30 --enlarge-target "Brief / Notice"
   (When a user installs the software without full case records or timeline, standard practice is to request a 30-day baseline extension under local court rules. Generates motion + opposing counsel conferral email).

5. COURT RECORD RECONSTRUCTION & TRANSCRIPT AUDIT:
   justlegalos-cli --json record inventory <path-to-case-folder>
   justlegalos-cli --json record reconcile <path-to-case-folder> --docket "<raw-court-docket-text>"
   justlegalos-cli --json record missing <path-to-case-folder>
   justlegalos-cli --json record request-form <path-to-case-folder> --state <NJ|CO|CA|FED>

6. JUDICIAL FINDINGS AUDIT & DISCREPANCY TESTING:
   justlegalos-cli --json review record <path-to-case-folder>
   (Compares court order findings against actual record exhibits to flag unsupported assertions).

7. JUDICIAL ACCOUNTABILITY & ETHICS PACKETS:
   justlegalos-cli --json accountability routes <path-to-case-folder>
   justlegalos-cli --json accountability packet <path-to-case-folder> --issue <issue_id>

8. ADA TITLE II REASONABLE ACCOMMODATION PETITION:
   justlegalos-cli --json ada petition <path-to-case-folder> --name "<Litigant Name>" --accommodations "remote,reading_time,breaks"

9. SERVICE PROOF & PROPOSED ORDERS:
   justlegalos-cli --json service cert <path-to-case-folder> --doc-title "<Filing Title>" --servee "<Opposing Counsel>"
   justlegalos-cli --json service order <path-to-case-folder> --motion-title "<Motion Title>" --relief "<Operative Clauses>"

10. DETERMINISTIC COURT EXPORTS (DOCX & PDF):
    justlegalos-cli --json export court-md <path-to-case-folder>
    (Generates clean, court-compliant Word DOCX and 28-line numbered PDF).

11. OFFLINE AIRGAPPED PDF POWER STATION:
    justlegalos-cli --json pdf merge --inputs <file1.pdf>,<file2.pdf> --output <merged.pdf>
    justlegalos-cli --json pdf split --input <source.pdf> --pages "1-5,8"
    justlegalos-cli --json pdf rotate --input <source.pdf> --degrees 90
    justlegalos-cli --json pdf redact --input <source.pdf> --patterns "SSN,DOB"

12. QUI TAM / FALSE CLAIMS ACT & EO 14395 TRIAGE (CALLING CARD & DUAL-TRACK ROUTING):
    justlegalos-cli --json qui-tam verify-entity --name "<Entity Name>" --state <State>
    justlegalos-cli --json qui-tam audit-9b <path-to-case-folder>
    justlegalos-cli --json qui-tam ingest-observation <path-to-case-folder> --file <photo.jpg> --notes "<Observed details>"
    justlegalos-cli --json qui-tam records-pack <path-to-case-folder> --state <State>
    justlegalos-cli --json qui-tam export-disclosure <path-to-case-folder>
    justlegalos-cli --json qui-tam format-portal <path-to-case-folder> --agency <HHS|DOJ|STATE>
    justlegalos-cli --json qui-tam format-inquiry <path-to-case-folder>
    (Automates preliminary federal OIG fraud triage: CMS NPPES checks, Rule 9(b) A-F readiness grade, SHA-256 evidence hashing, and dual-track routing between Track A Qui Tam Counsel on 100% contingency under statutory seal vs Track B <2,000 char portal auto-fills carrying the standardized [JUSTLEGAL OS | OPEN-SEASON v1.0 | EO-14395] Calling Card).

=============================================================================
CASE FOLDER ARCHITECTURE & REPOSITORY LAYOUT:
Standard Case Path: Documents/JustLegal/Cases/{caseId}/
  ├── case.json          # Fill-once profile (parties, jurisdiction, court rules)
  ├── orders/            # Court-issued orders, notices, and decisions
  ├── filings/           # Motions, petitions, answers, and certificates
  ├── evidence/          # Admitted trial exhibits, receipts, contracts
  ├── drafts/            # Working markdown pleadings and user declarations
  └── exports/           # Generated court-ready .docx and .pdf papers

=============================================================================
JUSTLEGAL CLOUD & JUSTLEGAL.ME INTEGRATION:
- Standard cases can sync additively with justlegal.me (Firebase project: justlegalapp1).
- Downloads route cleanly into orders/, filings/, and evidence/ without ever overwriting local drafts.
- Users authenticate via Google Sign-In or account token. Confidential cases remain strictly airgapped.

Respond to this prompt by initiating the First-Step Intake Protocol and confirming readiness to operate the JustLegal / OpenSeason engine.`;
  }

  async function handleCopy() {
    const text = generateMasterPrompt();
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 2500);
    } catch {
      // Fallback
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      copied = true;
      setTimeout(() => (copied = false), 2500);
    }
  }
</script>

<div class="rounded-2xl border border-primary/30 bg-card shadow-xl overflow-hidden animate-fade-in">
  <!-- Card Header -->
  <div class="p-5 sm:p-6 border-b border-border bg-gradient-to-r from-secondary/40 via-card to-secondary/30 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
    <div class="space-y-1.5">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-xs font-mono font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-primary/15 text-primary border border-primary/25 flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
          Master Agent Acclimation
        </span>
        <span class="text-[11px] text-muted-foreground font-mono">
          Universal Briefing for ChatGPT · Claude · Cursor · Local LLMs
        </span>
      </div>
      <h2 class="text-lg md:text-xl font-bold tracking-tight text-foreground">
        Acclimate Your AI Agent to JustLegal Systems & Active Matters
      </h2>
      <p class="text-xs text-muted-foreground max-w-2xl leading-relaxed">
        Copy this dense briefing prompt into your AI model. It immediately trains the agent on our court rules, pure-Rust CLI toolchain, folder layout, and your live active cases so it acts as your expert legal pair programmer.
      </p>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center gap-2 w-full md:w-auto shrink-0">
      <button
        onclick={handleCopy}
        class="flex-1 md:flex-none px-5 py-2.5 rounded-xl bg-primary hover:bg-primary/90 text-primary-foreground text-xs font-bold transition-all flex items-center justify-center gap-2 shadow-md active:scale-95"
      >
        {#if copied}
          <span class="text-sm">✓</span>
          <span>Copied to Clipboard!</span>
        {:else}
          <span class="text-sm">📋</span>
          <span>Copy Master Agent Prompt</span>
        {/if}
      </button>

      <button
        onclick={() => (showFullPrompt = !showFullPrompt)}
        class="px-3 py-2.5 rounded-xl border border-border bg-secondary hover:bg-secondary/80 text-foreground text-xs font-medium transition-colors"
        title="Toggle Prompt View"
      >
        {showFullPrompt ? "Collapse" : "Expand"}
      </button>
    </div>
  </div>

  <!-- Briefing Highlights Ribbon -->
  <div class="grid grid-cols-2 sm:grid-cols-4 border-b border-border bg-secondary/20 text-xs p-3 gap-2">
    <div class="flex items-center gap-2 px-2">
      <span class="text-primary font-bold">1. Intake:</span>
      <span class="text-muted-foreground truncate">State, venue & posture</span>
    </div>
    <div class="flex items-center gap-2 px-2">
      <span class="text-primary font-bold">2. Rules:</span>
      <span class="text-muted-foreground truncate">Zero citation hallucination</span>
    </div>
    <div class="flex items-center gap-2 px-2">
      <span class="text-primary font-bold">3. CLI:</span>
      <span class="text-muted-foreground truncate">11 Pure-Rust commands</span>
    </div>
    <div class="flex items-center gap-2 px-2">
      <span class="text-primary font-bold">4. Live Data:</span>
      <span class="text-foreground font-semibold truncate">{cases.length} workspace case(s)</span>
    </div>
  </div>

  <!-- Collapsible Live Prompt Preview Box -->
  {#if showFullPrompt}
    <div class="p-5 space-y-3 bg-secondary/10">
      <div class="flex items-center justify-between text-xs">
        <span class="text-muted-foreground font-mono font-medium flex items-center gap-1.5">
          <span>📜</span> System Prompt Content (Auto-Populates with Your Active Cases):
        </span>
        <button
          onclick={handleCopy}
          class="text-xs text-primary font-semibold hover:underline flex items-center gap-1"
        >
          {copied ? "✓ Copied" : "Copy Full Text"}
        </button>
      </div>

      <div class="relative rounded-xl border border-border bg-background/80 overflow-hidden shadow-inner">
        <pre class="p-4 text-[11px] font-mono text-foreground/90 whitespace-pre-wrap leading-relaxed max-h-72 overflow-y-auto selection:bg-primary/25">{generateMasterPrompt()}</pre>
        <div class="absolute bottom-2 right-3">
          <button
            onclick={handleCopy}
            class="px-3 py-1 rounded-lg bg-card/90 border border-border text-foreground hover:bg-secondary text-[11px] font-medium shadow-sm transition-all"
          >
            {copied ? "✓ Copied" : "Copy"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
