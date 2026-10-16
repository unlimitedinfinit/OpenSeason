<script lang="ts">
  import { ipc } from "$lib/ipc";
  import type {
    CaseProfile,
    IntakeStatusReport,
    VenueCompetenceReport,
  } from "$lib/types/case";

  let {
    open = $bindable(false),
    caseId,
    profile = $bindable(),
    onUpdated = () => {},
  }: {
    open: boolean;
    caseId: string;
    profile: CaseProfile;
    onUpdated?: () => void;
  } = $props();

  let intake = $state<IntakeStatusReport | null>(null);
  let venueReport = $state<VenueCompetenceReport | null>(null);
  let activeQuestionId = $state<string>("");
  let answerInput = $state<string>("");
  let busy = $state(false);
  let statusMsg = $state("");
  let errorMsg = $state("");

  $effect(() => {
    if (open && caseId) {
      refreshWizard();
    }
  });

  async function refreshWizard() {
    busy = true;
    errorMsg = "";
    try {
      const [statusRes, venueRes] = await Promise.all([
        ipc<IntakeStatusReport>("intake_status_cmd", { idOrPath: caseId }),
        ipc<VenueCompetenceReport>("check_venue_competence_cmd", { idOrPath: caseId }),
      ]);
      intake = statusRes;
      venueReport = venueRes;
      const targetQ =
        statusRes.questions.find((q) => q.id === activeQuestionId) ||
        statusRes.next_question ||
        statusRes.questions[0] ||
        null;
      if (targetQ) {
        activeQuestionId = targetQ.id;
        answerInput = targetQ.current_value || "";
      }
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  function selectQuestion(qId: string) {
    activeQuestionId = qId;
    const found = intake?.questions.find((q) => q.id === qId);
    if (found) {
      answerInput = found.current_value || "";
    }
    statusMsg = "";
  }

  async function submitAnswer(customVal?: string) {
    if (!activeQuestionId) return;
    const val = (customVal !== undefined ? customVal : answerInput).trim();
    if (!val) {
      errorMsg = "Please enter or choose an answer before saving.";
      return;
    }
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      const updated = await ipc<IntakeStatusReport>("intake_answer_cmd", {
        idOrPath: caseId,
        questionId: activeQuestionId,
        answer: val,
      });
      intake = updated;
      const reloaded = await ipc<CaseProfile>("get_case", { caseId });
      profile = reloaded;
      venueReport = await ipc<VenueCompetenceReport>("check_venue_competence_cmd", {
        idOrPath: caseId,
      });
      onUpdated();
      statusMsg = "Saved! Moving to next step...";
      if (updated.next_question) {
        activeQuestionId = updated.next_question.id;
        answerInput = updated.next_question.current_value || "";
      }
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function updateTierOrVenue(
    tier: string,
    stage: string,
    govState: string,
    docKind?: string
  ) {
    busy = true;
    errorMsg = "";
    try {
      profile.matter_tier = tier;
      profile.procedural_stage = stage;
      profile.governing_state = govState;
      if (docKind) {
        profile.document_kind = docKind;
      }
      if (govState === "NJ") profile.rule_pack_id = "nj-app-div";
      else if (govState === "CO") profile.rule_pack_id = "co-mandamus";
      else profile.rule_pack_id = "generic-us-appellate";

      profile = await ipc<CaseProfile>("save_case", { profile });
      onUpdated();
      await refreshWizard();
      statusMsg = `Updated architecture to ${tier.toUpperCase()} (${govState}).`;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  const currentQuestion = $derived(
    intake?.questions.find((q) => q.id === activeQuestionId) ||
      intake?.next_question ||
      null
  );
</script>

{#if open}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Guided Legal OS Intake Wizard">
    <div class="modal-panel">
      <header class="modal-header">
        <div>
          <span class="eyebrow">🧙 GUIDED LEGAL OS WIZARD & VENUE MATRIX</span>
          <h2>Step-by-Step Matter Intake & PDF Auto-Discovery</h2>
        </div>
        <button class="btn-close" type="button" onclick={() => (open = false)}>✕ Close</button>
      </header>

      <!-- Matter Tier, Stage & Venue Quick-Bar -->
      <section class="tier-bar">
        <div class="tier-group">
          <label for="wiz-tier">Matter Tier</label>
          <select
            id="wiz-tier"
            value={profile.matter_tier || "litigation"}
            onchange={(e) =>
              updateTierOrVenue(
                (e.currentTarget as HTMLSelectElement).value,
                profile.procedural_stage || "active",
                profile.governing_state || "NJ"
              )}
          >
            <option value="transactional">📜 Tier 1: Transactional (Will, Lease, Contract, POA)</option>
            <option value="administrative">📬 Tier 2: Pre-Suit / Notice (Demand, Cure, OPRA/FOIA)</option>
            <option value="litigation">⚖️ Tier 3: Court Litigation & Appeals</option>
          </select>
        </div>

        <div class="tier-group">
          <label for="wiz-kind">Instrument / Pleading Kind</label>
          <select
            id="wiz-kind"
            value={profile.document_kind}
            onchange={(e) => {
              const k = (e.currentTarget as HTMLSelectElement).value;
              const tier = ["will", "lease", "contractor", "power_of_attorney"].includes(k)
                ? "transactional"
                : ["demand_letter", "cure_notice", "security_deposit_demand", "records_request", "agency_appeal"].includes(k)
                  ? "administrative"
                  : "litigation";
              updateTierOrVenue(tier, profile.procedural_stage || "active", profile.governing_state || "NJ", k);
            }}
          >
            <optgroup label="📜 Tier 1: Everyday Contracts & Wills">
              <option value="will">Last Will & Testament (+ Self-Proving Affidavit)</option>
              <option value="lease">Residential Lease Agreement</option>
              <option value="contractor">Independent Contractor / Services Agreement</option>
              <option value="power_of_attorney">Durable Power of Attorney</option>
            </optgroup>
            <optgroup label="📬 Tier 2: Pre-Suit Notices & Administrative">
              <option value="demand_letter">Formal Demand Letter</option>
              <option value="cure_notice">Landlord-Tenant Notice to Cure / Quit</option>
              <option value="security_deposit_demand">Security Deposit Return Demand</option>
              <option value="records_request">Public Records Request (OPRA / CORA / FOIA)</option>
              <option value="agency_appeal">Administrative Agency Appeal</option>
            </optgroup>
            <optgroup label="⚖️ Tier 3: Court Pleadings, Motions & Appeals">
              <option value="reconsideration">Motion for Reconsideration</option>
              <option value="motion">General Trial Motion</option>
              <option value="enlargement">Motion for Extension of Time</option>
              <option value="stay">Motion to Stay Pending Appeal</option>
              <option value="complaint">Civil Complaint (Supports Pre-Filing Docket)</option>
              <option value="notice_of_appeal">Notice of Appeal</option>
              <option value="appellate_brief">Appellate Brief</option>
              <option value="mandamus">Petition for Mandamus / Original Proceeding</option>
            </optgroup>
          </select>
        </div>

        <div class="tier-group">
          <label for="wiz-venue">Venue / Jurisdiction</label>
          <select
            id="wiz-venue"
            value={profile.governing_state || "NJ"}
            onchange={(e) =>
              updateTierOrVenue(
                profile.matter_tier || "litigation",
                profile.procedural_stage || "active",
                (e.currentTarget as HTMLSelectElement).value
              )}
          >
            <option value="NJ">🏛️ New Jersey (R. 1:4-4(b) Certification)</option>
            <option value="CO">🏔️ Colorado (C.R.S. § 13-27-106 + Rule 121 Conferral)</option>
            <option value="FEDERAL">🦅 U.S. Federal Courts (28 U.S.C. § 1746)</option>
            <option value="SCOTUS">⚖️ U.S. Supreme Court (Rule 14.1 / Rule 39 IFP)</option>
            <option value="GENERIC">📄 Generic US Jurisdiction</option>
          </select>
        </div>

        <div class="tier-group">
          <label for="wiz-stage">Stage</label>
          <select
            id="wiz-stage"
            value={profile.procedural_stage || "active"}
            onchange={(e) =>
              updateTierOrVenue(
                profile.matter_tier || "litigation",
                (e.currentTarget as HTMLSelectElement).value,
                profile.governing_state || "NJ"
              )}
          >
            <option value="active">Active / Docket Assigned</option>
            <option value="pre_filing">Pre-Filing (No Docket Yet — Renders "To Be Assigned")</option>
            <option value="appellate">Appellate / Review</option>
          </select>
        </div>
      </section>

      {#if intake}
        <!-- Progress Bar -->
        <div class="progress-wrap">
          <div class="progress-meta">
            <span>
              <strong>{intake.completed_steps} of {intake.total_steps}</strong> steps complete ({intake.completion_percent}%)
            </span>
            <span class="badge-venue">{intake.venue_code} • {intake.matter_tier.toUpperCase()}</span>
          </div>
          <div class="progress-track">
            <div class="progress-fill" style="width: {intake.completion_percent}%"></div>
          </div>
        </div>

        <div class="wizard-grid">
          <!-- Left column: Question list -->
          <aside class="steps-sidebar">
            <h3>Interview Checklist</h3>
            {#each intake.questions as q}
              <button
                type="button"
                class="step-item"
                class:active={q.id === activeQuestionId}
                class:done={q.answered}
                onclick={() => selectQuestion(q.id)}
              >
                <span class="step-status">{q.answered ? "✅" : "⭕"}</span>
                <span class="step-text">
                  <strong>Step {q.step_number}</strong>: {q.prompt}
                </span>
              </button>
            {/each}
          </aside>

          <!-- Center column: Active Question Card -->
          <div class="question-stage">
            {#if currentQuestion}
              <div class="question-card">
                <div class="q-badge">
                  Step {currentQuestion.step_number} of {intake.total_steps} • {currentQuestion.category.toUpperCase()}
                </div>
                <h3>{currentQuestion.prompt}</h3>
                <p class="q-why">{currentQuestion.why}</p>

                {#if currentQuestion.choices && currentQuestion.choices.length > 0}
                  <div class="chip-row">
                    <span class="chip-label">Quick Choices:</span>
                    {#each currentQuestion.choices as choice}
                      <button
                        type="button"
                        class="choice-chip"
                        onclick={() => {
                          answerInput = choice;
                          submitAnswer(choice);
                        }}
                      >
                        {choice}
                      </button>
                    {/each}
                  </div>
                {/if}

                {#if currentQuestion.candidates && currentQuestion.candidates.length > 0}
                  <div class="chip-row candidate-box">
                    <span class="chip-label">📄 Auto-Detected from PDFs in orders/ & evidence/:</span>
                    {#each currentQuestion.candidates as cand}
                      <button
                        type="button"
                        class="candidate-chip"
                        onclick={() => {
                          answerInput = cand;
                        }}
                      >
                        ✨ Use "{cand}"
                      </button>
                    {/each}
                  </div>
                {/if}

                {#if currentQuestion.category === "draft_section"}
                  <textarea
                    rows="5"
                    bind:value={answerInput}
                    placeholder="Write in your own words (OpenSeason never invents facts or legal arguments)..."
                  ></textarea>
                {:else}
                  <input
                    type="text"
                    bind:value={answerInput}
                    placeholder="Enter value from your papers..."
                    onkeydown={(e) => e.key === "Enter" && submitAnswer()}
                  />
                {/if}

                <div class="q-actions">
                  <button
                    type="button"
                    class="btn-primary"
                    disabled={busy}
                    onclick={() => submitAnswer()}
                  >
                    {busy ? "Saving..." : "💾 Save & Next Question →"}
                  </button>
                  <button type="button" class="btn-secondary" onclick={refreshWizard}>
                    🔄 Rescan Folder PDFs
                  </button>
                </div>

                {#if statusMsg}
                  <p class="msg-ok">{statusMsg}</p>
                {/if}
                {#if errorMsg}
                  <p class="msg-err">{errorMsg}</p>
                {/if}
              </div>
            {/if}

            <!-- Venue Competence Matrix Card -->
            {#if venueReport}
              <div class="venue-card">
                <div class="venue-header">
                  <h4>🏛️ Venue Competence Matrix: {venueReport.venue.display_name}</h4>
                  <span class="status-pill" class:ok={venueReport.ok}>
                    {venueReport.ok ? "✅ Venue Rules Satisfied" : `⚠️ ${venueReport.issues.length} Rule Alert(s)`}
                  </span>
                </div>
                <p class="oath-box">
                  <strong>{venueReport.venue.sworn_statement_title} ({venueReport.venue.sworn_statement_citation}):</strong><br />
                  <em>"{venueReport.venue.mandatory_oath_text}"</em>
                </p>
                <div class="venue-meta">
                  <span>📬 Mail Service Adder: <strong>+{venueReport.venue.mail_service_adder_days} days</strong></span>
                  <span>⏱️ Tolling Mode: <strong>{venueReport.venue.tolling_mode === "pause_remaining" ? "NJ Pause Remaining Days" : "Full Clock Reset"}</strong></span>
                  {#if venueReport.venue.requires_conferral_certificate}
                    <span>🤝 Conferral Required: <strong>{venueReport.venue.conferral_rule}</strong></span>
                  {/if}
                </div>
                {#if venueReport.issues.length > 0}
                  <ul class="venue-issues">
                    {#each venueReport.issues as iss}
                      <li><strong>[{iss.field}]</strong> {iss.message}</li>
                    {/each}
                  </ul>
                {/if}
              </div>
            {/if}

            <!-- Offline Scanned PDFs Summary -->
            {#if intake.scan.scanned_files.length > 0}
              <div class="scan-summary">
                <h4>🔍 Offline PDF Text Extraction ({intake.scan.scanned_files.length} file(s) indexed)</h4>
                <div class="scan-list">
                  {#each intake.scan.scanned_files as sf}
                    <div class="scan-item">
                      <strong>[{sf.folder}/] {sf.filename}</strong> ({sf.page_count} p.)
                      <p>{sf.snippet}</p>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 20, 28, 0.72);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1200;
    padding: 1rem;
  }
  .modal-panel {
    background: var(--paper, #fcfaf5);
    color: var(--ink, #1b1e23);
    border: 2px solid var(--border-strong, #2c3540);
    border-radius: 10px;
    width: min(1080px, 96vw);
    max-height: 92vh;
    overflow-y: auto;
    padding: 1.25rem 1.5rem;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.35);
  }
  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 1px solid var(--border, #d8d2c5);
    padding-bottom: 0.75rem;
    margin-bottom: 1rem;
  }
  .eyebrow {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: #5b4636;
  }
  .modal-header h2 {
    margin: 0.2rem 0 0;
    font-size: 1.35rem;
  }
  .btn-close {
    background: transparent;
    border: 1px solid #999;
    border-radius: 6px;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
    font-weight: 600;
  }
  .tier-bar {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 0.75rem;
    background: #f2ede2;
    padding: 0.75rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }
  .tier-group label {
    display: block;
    font-size: 0.75rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
  }
  .tier-group select {
    width: 100%;
    padding: 0.45rem;
    border-radius: 6px;
    border: 1px solid #b8b0a0;
    background: #fff;
    font-size: 0.85rem;
  }
  .progress-wrap {
    margin-bottom: 1rem;
  }
  .progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    margin-bottom: 0.3rem;
  }
  .badge-venue {
    background: #1e3a5f;
    color: #fff;
    padding: 0.1rem 0.55rem;
    border-radius: 99px;
    font-size: 0.75rem;
    font-weight: 600;
  }
  .progress-track {
    height: 10px;
    background: #e2dcd0;
    border-radius: 99px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #1b6b45, #2ea36b);
    transition: width 0.25s ease;
  }
  .wizard-grid {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 1rem;
  }
  @media (max-width: 800px) {
    .wizard-grid {
      grid-template-columns: 1fr;
    }
  }
  .steps-sidebar {
    border-right: 1px solid #dcd5c7;
    padding-right: 0.75rem;
    max-height: 55vh;
    overflow-y: auto;
  }
  .steps-sidebar h3 {
    font-size: 0.9rem;
    margin: 0 0 0.5rem;
  }
  .step-item {
    display: flex;
    align-items: flex-start;
    gap: 0.45rem;
    width: 100%;
    text-align: left;
    padding: 0.5rem;
    margin-bottom: 0.35rem;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    font-size: 0.82rem;
  }
  .step-item:hover {
    background: #f0ebe0;
  }
  .step-item.active {
    background: #e8f0fe;
    border-color: #2b579a;
  }
  .question-card {
    background: #fff;
    border: 1px solid #d5cebf;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }
  .q-badge {
    font-size: 0.75rem;
    font-weight: 700;
    color: #2b579a;
    margin-bottom: 0.25rem;
  }
  .question-card h3 {
    margin: 0 0 0.35rem;
    font-size: 1.15rem;
  }
  .q-why {
    font-size: 0.88rem;
    color: #4f5660;
    margin: 0 0 0.75rem;
  }
  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    align-items: center;
    margin-bottom: 0.75rem;
  }
  .chip-label {
    font-size: 0.78rem;
    font-weight: 600;
    color: #444;
  }
  .choice-chip,
  .candidate-chip {
    padding: 0.3rem 0.65rem;
    border-radius: 99px;
    border: 1px solid #2b579a;
    background: #f0f5ff;
    color: #1c3d72;
    font-size: 0.8rem;
    cursor: pointer;
    font-weight: 600;
  }
  .candidate-box {
    background: #f6fbf8;
    padding: 0.5rem;
    border-radius: 6px;
    border: 1px dashed #257953;
  }
  .candidate-chip {
    border-color: #257953;
    background: #e6f6ed;
    color: #165036;
  }
  .question-card input,
  .question-card textarea {
    width: 100%;
    padding: 0.6rem;
    border: 1px solid #b5ae9e;
    border-radius: 6px;
    font-size: 0.95rem;
    margin-bottom: 0.75rem;
    box-sizing: border-box;
  }
  .q-actions {
    display: flex;
    gap: 0.6rem;
  }
  .btn-primary {
    background: #1b6b45;
    color: #fff;
    border: none;
    padding: 0.55rem 1.1rem;
    border-radius: 6px;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-secondary {
    background: #eee8dc;
    color: #222;
    border: 1px solid #bfb8a8;
    padding: 0.55rem 0.9rem;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
  }
  .msg-ok {
    color: #1b6b45;
    font-size: 0.85rem;
    font-weight: 600;
    margin-top: 0.5rem;
  }
  .msg-err {
    color: #b42318;
    font-size: 0.85rem;
    font-weight: 600;
    margin-top: 0.5rem;
  }
  .venue-card {
    background: #f8f5ec;
    border: 1px solid #d5cebf;
    border-radius: 8px;
    padding: 0.9rem;
    margin-bottom: 1rem;
  }
  .venue-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .venue-header h4 {
    margin: 0;
    font-size: 0.95rem;
  }
  .status-pill {
    font-size: 0.75rem;
    padding: 0.15rem 0.55rem;
    border-radius: 99px;
    background: #fef3c7;
    color: #92400e;
    font-weight: 700;
  }
  .status-pill.ok {
    background: #d1fae5;
    color: #065f46;
  }
  .oath-box {
    font-size: 0.82rem;
    background: #fff;
    padding: 0.55rem;
    border-left: 3px solid #1e3a5f;
    margin: 0 0 0.5rem;
  }
  .venue-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.85rem;
    font-size: 0.8rem;
  }
  .venue-issues {
    margin: 0.5rem 0 0;
    padding-left: 1.2rem;
    font-size: 0.82rem;
    color: #b42318;
  }
  .scan-summary h4 {
    margin: 0 0 0.4rem;
    font-size: 0.9rem;
  }
  .scan-list {
    display: grid;
    gap: 0.4rem;
  }
  .scan-item {
    background: #fff;
    border: 1px solid #e0d9ca;
    padding: 0.45rem 0.65rem;
    border-radius: 6px;
    font-size: 0.8rem;
  }
  .scan-item p {
    margin: 0.2rem 0 0;
    color: #555;
  }
</style>
