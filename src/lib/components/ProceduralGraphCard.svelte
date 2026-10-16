<script lang="ts">
  import { goto } from "$app/navigation";
  import { ipc } from "$lib/ipc";
  import type {
    CaseFilingItem,
    CaseProfile,
    EscalationResult,
    NextStepsReport,
    ProceduralBranch,
  } from "$lib/types/case";

  let {
    caseId,
    profile = $bindable(),
    onOpenWizard = () => {},
    onProfileSaved = () => {},
  }: {
    caseId: string;
    profile: CaseProfile;
    onOpenWizard?: () => void;
    onProfileSaved?: () => void;
  } = $props();

  let report = $state<NextStepsReport | null>(null);
  let busy = $state(false);
  let statusMsg = $state("");
  let errorMsg = $state("");
  let quickEventTitle = $state("Order Denying Motion");
  let quickEventDate = $state(new Date().toISOString().slice(0, 10));

  $effect(() => {
    if (caseId && profile) {
      loadGraph();
    }
  });

  async function loadGraph() {
    try {
      report = await ipc<NextStepsReport>("case_next_steps_cmd", { idOrPath: caseId });
    } catch (e) {
      errorMsg = String(e);
    }
  }

  async function toggleMailService(method: string) {
    busy = true;
    errorMsg = "";
    try {
      profile.service_method = method;
      profile = await ipc<CaseProfile>("save_case", { profile });
      onProfileSaved();
      await loadGraph();
      statusMsg = `Service method set to ${method.toUpperCase()}. Deadlines recalculated.`;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function logTriggerEvent() {
    if (!quickEventTitle.trim() || !quickEventDate.trim()) return;
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      profile.timeline = [
        ...(profile.timeline || []),
        {
          id: crypto.randomUUID(),
          title: quickEventTitle.trim(),
          date: quickEventDate.trim(),
          notes: "Logged via Predictive Procedural Graph",
        },
      ];
      profile = await ipc<CaseProfile>("save_case", { profile });
      onProfileSaved();
      await loadGraph();
      statusMsg = `Logged "${quickEventTitle}" (${quickEventDate}) and recalculated procedural branches.`;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function initializeBranchFiling(branch: ProceduralBranch) {
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      if (branch.escalation_target) {
        const res = await ipc<EscalationResult>("escalate_case_cmd", {
          idOrPath: caseId,
          targetStage: branch.escalation_target,
        });
        statusMsg = `${res.message} (${res.copied_record_files} record file(s) carried over into Appendix/Evidence).`;
        if (res.new_profile?.id) {
          await goto(`/case/${res.new_profile.id}`);
        }
        return;
      }

      const now = new Date().toISOString();
      const newFiling: CaseFilingItem = {
        id: crypto.randomUUID(),
        title: branch.title,
        kind: branch.template_kind || "motion",
        rule: branch.rule_citation,
        status: "draft",
        user_text: "",
        relief_sought: "",
        created_at: now,
        updated_at: now,
      };
      await ipc("save_case_filing_cmd", { caseId, filing: newFiling });
      profile.document_kind = branch.template_kind || profile.document_kind;
      profile = await ipc<CaseProfile>("save_case", { profile });
      onProfileSaved();
      statusMsg = `Initialized "${branch.title}" (${branch.rule_citation}) on the Multi-Filing Docket!`;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function escalateMatter(targetStage: string) {
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      const res = await ipc<EscalationResult>("escalate_case_cmd", {
        idOrPath: caseId,
        targetStage,
      });
      statusMsg = `${res.message} (${res.copied_record_files} record file(s) copied).`;
      if (res.new_profile?.id) {
        await goto(`/case/${res.new_profile.id}`);
      }
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="graph-card">
  <div class="graph-header">
    <div>
      <span class="eyebrow">🧭 PREDICTIVE PROCEDURAL STATE GRAPH & ESCALATION ENGINE</span>
      <h3>"If This Happens, What Are My Next Moves?"</h3>
    </div>
    <div class="header-actions">
      <button type="button" class="btn-wizard" onclick={onOpenWizard}>
        🧙 Launch Guided Intake Wizard
      </button>
      <div class="service-pill">
        <span>Service:</span>
        <button
          type="button"
          class:active={(profile.service_method || "electronic") !== "mail"}
          onclick={() => toggleMailService("electronic")}
        >
          ⚡ Electronic / Personal (+0d)
        </button>
        <button
          type="button"
          class:active={profile.service_method === "mail"}
          onclick={() => toggleMailService("mail")}
        >
          📬 Mail (+3d Rule)
        </button>
      </div>
    </div>
  </div>

  <!-- Quick Event Logger to drive the Procedural Graph -->
  <div class="event-bar">
    <span class="event-label">⚡ Log Trigger Event:</span>
    <select bind:value={quickEventTitle}>
      <option value="Order Denying Motion">Order Denying Motion / Judgment Entered</option>
      <option value="Motion for Reconsideration Filed">Motion for Reconsideration Filed (Triggers Tolling)</option>
      <option value="Complaint / Motion Served">Complaint or Motion Served on Party</option>
      <option value="Clerical Refusal / Fee Waiver Delay">Clerical Refusal / Unlawful Docket Delay</option>
      <option value="Demand / Notice Delivered">Pre-Suit Demand / Cure Notice Delivered</option>
    </select>
    <input type="date" bind:value={quickEventDate} />
    <button type="button" class="btn-log" disabled={busy} onclick={logTriggerEvent}>
      + Log & Compute Branches
    </button>
  </div>

  {#if report}
    <div class="triggers-strip">
      <span><strong>Venue:</strong> {report.venue_code}</span>
      <span><strong>Tier:</strong> {report.matter_tier.toUpperCase()}</span>
      <span>
        <strong>Detected Triggers:</strong>
        {report.detected_triggers.length > 0 ? report.detected_triggers.join(" • ") : "Default stage branches"}
      </span>
    </div>

    <div class="branch-grid">
      {#each report.branches as b}
        <article class="branch-item" class:tolled={b.status === "paused_tolled"}>
          <div class="branch-top">
            <span class="rule-badge">{b.rule_citation}</span>
            <span class="clock-badge" class:pause={b.clock_interaction === "pause_remaining"}>
              {#if b.clock_interaction === "pause_remaining"}
                ⏸️ NJ Pause/Toll Clock
              {:else if b.clock_interaction === "reset_full"}
                🔄 CO/Fed Full Clock Reset
              {:else}
                ⏱️ {b.base_days}d {b.mail_offset_days > 0 ? `+ ${b.mail_offset_days}d mail` : ""}
              {/if}
            </span>
          </div>

          <h4>{b.title}</h4>
          <p class="branch-exp">{b.explanation}</p>

          <div class="deadline-row">
            {#if b.status === "paused_tolled"}
              <span class="due-pill tolled">
                ⏸️ PAUSED ({b.days_remaining_after_toll ?? "?"} days remaining after tolling motion is decided)
              </span>
            {:else if b.computed_deadline}
              <span class="due-pill" class:urgent={(b.days_until_due ?? 99) <= 7}>
                📅 Due: <strong>{b.computed_deadline}</strong>
                {#if b.days_until_due !== null && b.days_until_due !== undefined}
                  ({b.days_until_due >= 0 ? `${b.days_until_due}d left` : `${Math.abs(b.days_until_due)}d ago`})
                {/if}
              </span>
            {:else}
              <span class="due-pill neutral">📋 Ready when triggered</span>
            {/if}
          </div>

          <button
            type="button"
            class="btn-branch-action"
            disabled={busy}
            onclick={() => initializeBranchFiling(b)}
          >
            {#if b.escalation_target}
              🚀 1-Click Escalate & Carry Over Record →
            {:else}
              📝 1-Click Initialize "{b.title}" Packet
            {/if}
          </button>
        </article>
      {/each}
    </div>
  {/if}

  <!-- Cross-Venue Escalation Bar -->
  <div class="escalation-bar">
    <span class="esc-label">🪜 Cross-Venue Escalation (Copies Lower Record to New Appendix):</span>
    <div class="esc-buttons">
      {#if (profile.matter_tier || "litigation") === "administrative"}
        <button type="button" disabled={busy} onclick={() => escalateMatter("trial_lawsuit")}>
          ⚖️ Graduate Pre-Suit Notice → Tier 3 Civil Lawsuit
        </button>
      {/if}
      <button type="button" disabled={busy} onclick={() => escalateMatter("appellate_appeal")}>
        🏛️ Escalate Trial → Appellate Appeal
      </button>
      <button type="button" disabled={busy} onclick={() => escalateMatter("mandamus")}>
        ⚡ Escalate → Mandamus / C.A.R. 21 / Prerogative Writ
      </button>
      <button type="button" disabled={busy} onclick={() => escalateMatter("scotus_cert")}>
        🦅 Escalate → U.S. Supreme Court Certiorari
      </button>
    </div>
  </div>

  {#if statusMsg}
    <p class="msg-ok">{statusMsg}</p>
  {/if}
  {#if errorMsg}
    <p class="msg-err">{errorMsg}</p>
  {/if}
</section>

<style>
  .graph-card {
    background: linear-gradient(180deg, #fcfaf4 0%, #f5efe2 100%);
    border: 2px solid #2c3540;
    border-radius: 10px;
    padding: 1.1rem 1.25rem;
    margin-bottom: 1.25rem;
  }
  .graph-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-bottom: 0.85rem;
  }
  .eyebrow {
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.06em;
    color: #1e3a5f;
  }
  .graph-header h3 {
    margin: 0.15rem 0 0;
    font-size: 1.15rem;
  }
  .header-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    align-items: center;
  }
  .btn-wizard {
    background: #1e3a5f;
    color: #fff;
    border: none;
    border-radius: 7px;
    padding: 0.5rem 0.95rem;
    font-weight: 700;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .service-pill {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    background: #e9e2d2;
    padding: 0.25rem 0.45rem;
    border-radius: 7px;
    font-size: 0.78rem;
    font-weight: 600;
  }
  .service-pill button {
    border: 1px solid transparent;
    background: transparent;
    padding: 0.25rem 0.55rem;
    border-radius: 5px;
    cursor: pointer;
    font-size: 0.76rem;
    font-weight: 600;
  }
  .service-pill button.active {
    background: #fff;
    border-color: #2c3540;
  }
  .event-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    background: #fff;
    border: 1px solid #d5cebf;
    padding: 0.55rem 0.75rem;
    border-radius: 8px;
    margin-bottom: 0.75rem;
  }
  .event-label {
    font-size: 0.8rem;
    font-weight: 700;
  }
  .event-bar select,
  .event-bar input {
    padding: 0.38rem 0.55rem;
    border: 1px solid #b8b0a0;
    border-radius: 6px;
    font-size: 0.82rem;
  }
  .btn-log {
    background: #1b6b45;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 0.42rem 0.85rem;
    font-size: 0.8rem;
    font-weight: 700;
    cursor: pointer;
  }
  .triggers-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    font-size: 0.8rem;
    color: #3c434a;
    margin-bottom: 0.75rem;
  }
  .branch-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(265px, 1fr));
    gap: 0.85rem;
    margin-bottom: 0.9rem;
  }
  .branch-item {
    background: #fff;
    border: 1px solid #cfc7b6;
    border-radius: 8px;
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }
  .branch-item.tolled {
    border-color: #d97706;
    background: #fffbeb;
  }
  .branch-top {
    display: flex;
    justify-content: space-between;
    gap: 0.4rem;
    margin-bottom: 0.4rem;
  }
  .rule-badge {
    font-size: 0.72rem;
    font-weight: 700;
    background: #eef2ff;
    color: #1e3a8a;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
  }
  .clock-badge {
    font-size: 0.72rem;
    font-weight: 700;
    background: #f3f4f6;
    color: #374151;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
  }
  .clock-badge.pause {
    background: #fef3c7;
    color: #92400e;
  }
  .branch-item h4 {
    margin: 0 0 0.35rem;
    font-size: 0.98rem;
  }
  .branch-exp {
    font-size: 0.8rem;
    color: #4b5563;
    margin: 0 0 0.65rem;
    line-height: 1.35;
  }
  .deadline-row {
    margin-bottom: 0.65rem;
  }
  .due-pill {
    display: inline-block;
    font-size: 0.78rem;
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    background: #e6f6ed;
    color: #165036;
  }
  .due-pill.urgent {
    background: #fee2e2;
    color: #991b1b;
  }
  .due-pill.tolled {
    background: #fef3c7;
    color: #92400e;
    font-weight: 600;
  }
  .due-pill.neutral {
    background: #f3f4f6;
    color: #4b5563;
  }
  .btn-branch-action {
    width: 100%;
    padding: 0.48rem;
    border-radius: 6px;
    border: 1px solid #1e3a5f;
    background: #f0f5ff;
    color: #1e3a5f;
    font-weight: 700;
    font-size: 0.8rem;
    cursor: pointer;
  }
  .btn-branch-action:hover {
    background: #1e3a5f;
    color: #fff;
  }
  .escalation-bar {
    border-top: 1px solid #d5cebf;
    padding-top: 0.75rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    align-items: center;
    justify-content: space-between;
  }
  .esc-label {
    font-size: 0.78rem;
    font-weight: 700;
    color: #374151;
  }
  .esc-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }
  .esc-buttons button {
    background: #fff;
    border: 1px solid #6b7280;
    border-radius: 6px;
    padding: 0.35rem 0.65rem;
    font-size: 0.76rem;
    font-weight: 600;
    cursor: pointer;
  }
  .esc-buttons button:hover {
    background: #2c3540;
    color: #fff;
  }
  .msg-ok {
    color: #1b6b45;
    font-size: 0.82rem;
    font-weight: 600;
    margin: 0.5rem 0 0;
  }
  .msg-err {
    color: #b42318;
    font-size: 0.82rem;
    font-weight: 600;
    margin: 0.5rem 0 0;
  }
</style>
