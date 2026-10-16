<script lang="ts">
  import { goto } from "$app/navigation";
  import { ipc } from "$lib/ipc";
  import type { CaseProfile, Filer, PortfolioStatusReport } from "$lib/types/case";

  let {
    onCreated = () => {},
  }: {
    onCreated?: () => void;
  } = $props();

  let portfolio = $state<PortfolioStatusReport | null>(null);
  let showFilerModal = $state(false);
  let filerDraft = $state<Filer>({
    name: "",
    role: "Proceeding without an attorney",
    address_lines: [""],
    phone: "",
    email: "",
    signature_name: "",
  });
  let addressText = $state("");
  let busy = $state(false);
  let statusMsg = $state("");
  let errorMsg = $state("");

  $effect(() => {
    loadPortfolio();
  });

  export async function loadPortfolio() {
    try {
      const res = await ipc<PortfolioStatusReport>("portfolio_status_cmd");
      portfolio = res;
      if (res.global_filer) {
        filerDraft = { ...res.global_filer };
        addressText = (res.global_filer.address_lines || []).join("\n");
      }
    } catch (e) {
      errorMsg = String(e);
    }
  }

  async function saveGlobalFiler() {
    busy = true;
    errorMsg = "";
    statusMsg = "";
    try {
      filerDraft.address_lines = addressText
        .split("\n")
        .map((l) => l.trim())
        .filter(Boolean);
      if (!filerDraft.signature_name.trim()) {
        filerDraft.signature_name = filerDraft.name.trim();
      }
      await ipc<Filer>("save_global_filer_cmd", { filer: filerDraft });
      await loadPortfolio();
      showFilerModal = false;
      statusMsg = "Saved Global Filer Profile (filer.json). Every new matter can auto-populate your identity!";
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function quickStartInstrument(
    titlePrompt: string,
    documentKind: string,
    matterTier: string,
    defaultVenue = "NJ"
  ) {
    busy = true;
    errorMsg = "";
    try {
      const created = await ipc<CaseProfile>("create_case", {
        title: titlePrompt,
        mode: "standard",
        documentKind,
      });
      created.matter_tier = matterTier;
      created.governing_state = defaultVenue;
      if (portfolio?.global_filer?.name) {
        created.filer = { ...portfolio.global_filer };
      }
      await ipc<CaseProfile>("save_case", { profile: created });
      onCreated();
      await goto(`/case/${created.id}`);
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="radar-card">
  <div class="radar-top">
    <div>
      <span class="eyebrow">🌐 LEGAL OPERATING SYSTEM • MULTI-STATE PORTFOLIO & 3-TIER LAUNCHPAD</span>
      <h2>From Everyday Wills & Contracts to State & Federal Appeals</h2>
    </div>
    <button type="button" class="btn-filer" onclick={() => (showFilerModal = !showFilerModal)}>
      🪪 Global Filer Profile {portfolio?.global_filer?.name ? `(${portfolio.global_filer.name})` : "(Set Once)"}
    </button>
  </div>

  <!-- 3-Tier Quick Launchpad -->
  <div class="tier-launchpad">
    <div class="tier-box">
      <div class="tier-tag t1">📜 TIER 1: TRANSACTIONAL</div>
      <h4>Everyday Instruments & Contracts</h4>
      <p>No court caption or docket required. Auto-includes state witness & notary blocks.</p>
      <div class="quick-btns">
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Last Will and Testament", "will", "transactional")}
        >
          ✍️ Last Will + Self-Proving Affidavit
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Residential Lease Agreement", "lease", "transactional")}
        >
          🏠 Residential Lease Agreement
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Independent Contractor Agreement", "contractor", "transactional")}
        >
          🤝 Contractor / Services Agreement
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Durable Power of Attorney", "power_of_attorney", "transactional")}
        >
          🛡️ Durable Power of Attorney
        </button>
      </div>
    </div>

    <div class="tier-box">
      <div class="tier-tag t2">📬 TIER 2: PRE-SUIT & ADMIN</div>
      <h4>Statutory Notices & Cure Clocks</h4>
      <p>Tracks certified-mail cure windows and 1-click graduates into a Tier 3 lawsuit if ignored.</p>
      <div class="quick-btns">
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Landlord-Tenant Notice to Cure", "cure_notice", "administrative")}
        >
          🔑 Landlord-Tenant Cure Notice
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Security Deposit Return Demand", "security_deposit_demand", "administrative")}
        >
          💰 Security Deposit Demand
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Pre-Suit Formal Demand Letter", "demand_letter", "administrative")}
        >
          📨 Formal Breach / Demand Letter
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("OPRA / CORA / FOIA Public Records Request", "records_request", "administrative")}
        >
          📂 OPRA / CORA / FOIA Request
        </button>
      </div>
    </div>

    <div class="tier-box">
      <div class="tier-tag t3">⚖️ TIER 3: COURT LITIGATION</div>
      <h4>Trial, Appellate, Mandamus & SCOTUS</h4>
      <p>Full venue competence (NJ R. 1:4-4(b), CO Rule 121 conferral, 28 U.S.C. § 1746, SCOTUS Rule 14).</p>
      <div class="quick-btns">
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Motion for Reconsideration", "reconsideration", "litigation", "NJ")}
        >
          🔄 NJ / CO Motion for Reconsideration
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Civil Complaint (Pre-Filing)", "complaint", "litigation", "NJ")}
        >
          🏛️ New Civil Complaint (Pre-Filing)
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Verified Petition for Mandamus", "mandamus", "litigation", "CO")}
        >
          ⚡ Mandamus / C.A.R. 21 Petition
        </button>
        <button
          type="button"
          disabled={busy}
          onclick={() => quickStartInstrument("Notice of Appeal", "notice_of_appeal", "litigation", "NJ")}
        >
          📣 Notice of Appeal
        </button>
      </div>
    </div>
  </div>

  <!-- Global Filer Drawer -->
  {#if showFilerModal}
    <div class="filer-drawer">
      <h4>🪪 Global Filer Profile (`Documents/JustLegal/filer.json`)</h4>
      <p class="drawer-sub">Fill your name, address, phone, and email once. Every new Will, Notice, Motion, or Appeal can auto-fill from this profile.</p>
      <div class="filer-grid">
        <label>
          Full Name
          <input type="text" bind:value={filerDraft.name} placeholder="e.g. Jordan Example" />
        </label>
        <label>
          Default Role / Capacity
          <input type="text" bind:value={filerDraft.role} placeholder="Proceeding without an attorney" />
        </label>
        <label>
          Phone
          <input type="text" bind:value={filerDraft.phone} placeholder="555-0100" />
        </label>
        <label>
          Email
          <input type="email" bind:value={filerDraft.email} placeholder="name@example.test" />
        </label>
        <label class="full-span">
          Mailing Address (one line per row)
          <textarea rows="2" bind:value={addressText} placeholder="100 Sample Street&#10;Example City, NJ 00000"></textarea>
        </label>
      </div>
      <div class="drawer-actions">
        <button type="button" class="btn-save" disabled={busy} onclick={saveGlobalFiler}>
          💾 Save Global Filer Profile
        </button>
        <button type="button" class="btn-cancel" onclick={() => (showFilerModal = false)}>Cancel</button>
      </div>
    </div>
  {/if}

  <!-- Multi-Venue Deadline Radar Table -->
  {#if portfolio && portfolio.matters.length > 0}
    <div class="radar-list">
      <div class="radar-list-head">
        <h4>📡 Multi-State Deadline & Intake Radar ({portfolio.total_matters} active matter{portfolio.total_matters === 1 ? "" : "s"})</h4>
        <button type="button" class="btn-refresh" onclick={loadPortfolio}>🔄 Refresh Radar</button>
      </div>
      <div class="radar-rows">
        {#each portfolio.matters.slice(0, 6) as m}
          <a class="radar-row" href={`/case/${m.case_id}`}>
            <div class="r-main">
              <span class="r-venue">{m.venue_code}</span>
              <span class="r-tier">{m.matter_tier}</span>
              <strong>{m.title}</strong>
              {#if m.docket_number}
                <span class="r-docket">#{m.docket_number}</span>
              {/if}
            </div>
            <div class="r-meta">
              {#if m.urgent_deadline}
                <span class="r-due" class:urgent={(m.days_until_due ?? 99) <= 7}>
                  ⏱️ {m.urgent_branch_title}: <strong>{m.urgent_deadline}</strong>
                  {#if m.days_until_due !== null && m.days_until_due !== undefined}
                    ({m.days_until_due}d)
                  {/if}
                </span>
              {/if}
              <span class="r-prog">Wizard: {m.completion_percent}%</span>
            </div>
          </a>
        {/each}
      </div>
    </div>
  {/if}

  {#if statusMsg}
    <p class="msg-ok">{statusMsg}</p>
  {/if}
  {#if errorMsg}
    <p class="msg-err">{errorMsg}</p>
  {/if}
</section>

<style>
  .radar-card {
    background: linear-gradient(180deg, #fcfaf5 0%, #f3ede0 100%);
    border: 2px solid #2c3540;
    border-radius: 12px;
    padding: 1.25rem 1.4rem;
    margin-bottom: 1.5rem;
  }
  .radar-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .eyebrow {
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.06em;
    color: #1e3a5f;
  }
  .radar-top h2 {
    margin: 0.2rem 0 0;
    font-size: 1.3rem;
  }
  .btn-filer {
    background: #1e3a5f;
    color: #fff;
    border: none;
    border-radius: 8px;
    padding: 0.5rem 0.95rem;
    font-weight: 700;
    font-size: 0.82rem;
    cursor: pointer;
  }
  .tier-launchpad {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 0.9rem;
    margin-bottom: 1rem;
  }
  .tier-box {
    background: #fff;
    border: 1px solid #d5cebf;
    border-radius: 9px;
    padding: 0.9rem;
  }
  .tier-tag {
    display: inline-block;
    font-size: 0.7rem;
    font-weight: 800;
    padding: 0.15rem 0.55rem;
    border-radius: 99px;
    margin-bottom: 0.4rem;
  }
  .tier-tag.t1 {
    background: #e6f6ed;
    color: #165036;
  }
  .tier-tag.t2 {
    background: #fef3c7;
    color: #92400e;
  }
  .tier-tag.t3 {
    background: #e8f0fe;
    color: #1e3a8a;
  }
  .tier-box h4 {
    margin: 0 0 0.25rem;
    font-size: 1rem;
  }
  .tier-box p {
    margin: 0 0 0.65rem;
    font-size: 0.78rem;
    color: #4b5563;
    line-height: 1.35;
  }
  .quick-btns {
    display: grid;
    gap: 0.38rem;
  }
  .quick-btns button {
    text-align: left;
    padding: 0.42rem 0.65rem;
    border-radius: 6px;
    border: 1px solid #cfc7b6;
    background: #faf8f2;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }
  .quick-btns button:hover {
    background: #1e3a5f;
    color: #fff;
    border-color: #1e3a5f;
  }
  .filer-drawer {
    background: #fff;
    border: 1px solid #2b579a;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
  }
  .filer-drawer h4 {
    margin: 0 0 0.25rem;
  }
  .drawer-sub {
    margin: 0 0 0.75rem;
    font-size: 0.82rem;
    color: #4b5563;
  }
  .filer-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.65rem;
    margin-bottom: 0.75rem;
  }
  .filer-grid label {
    display: flex;
    flex-direction: column;
    font-size: 0.78rem;
    font-weight: 700;
    gap: 0.25rem;
  }
  .filer-grid label.full-span {
    grid-column: 1 / -1;
  }
  .filer-grid input,
  .filer-grid textarea {
    padding: 0.45rem;
    border: 1px solid #b8b0a0;
    border-radius: 6px;
    font-size: 0.85rem;
  }
  .drawer-actions {
    display: flex;
    gap: 0.5rem;
  }
  .btn-save {
    background: #1b6b45;
    color: #fff;
    border: none;
    padding: 0.48rem 0.95rem;
    border-radius: 6px;
    font-weight: 700;
    cursor: pointer;
  }
  .btn-cancel {
    background: #eee8dc;
    border: 1px solid #b8b0a0;
    padding: 0.48rem 0.85rem;
    border-radius: 6px;
    cursor: pointer;
  }
  .radar-list-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .radar-list-head h4 {
    margin: 0;
    font-size: 0.92rem;
  }
  .btn-refresh {
    background: transparent;
    border: 1px solid #999;
    border-radius: 5px;
    padding: 0.2rem 0.55rem;
    font-size: 0.75rem;
    cursor: pointer;
  }
  .radar-rows {
    display: grid;
    gap: 0.4rem;
  }
  .radar-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem;
    background: #fff;
    border: 1px solid #d8d0c0;
    padding: 0.5rem 0.75rem;
    border-radius: 7px;
    text-decoration: none;
    color: inherit;
    font-size: 0.83rem;
  }
  .radar-row:hover {
    border-color: #1e3a5f;
    background: #f8faff;
  }
  .r-main {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
  }
  .r-venue {
    background: #1e3a5f;
    color: #fff;
    padding: 0.1rem 0.45rem;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
  }
  .r-tier {
    background: #e9e2d2;
    color: #333;
    padding: 0.1rem 0.45rem;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
  }
  .r-docket {
    color: #555;
    font-family: monospace;
  }
  .r-meta {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .r-due {
    background: #e6f6ed;
    color: #165036;
    padding: 0.15rem 0.5rem;
    border-radius: 5px;
    font-size: 0.75rem;
  }
  .r-due.urgent {
    background: #fee2e2;
    color: #991b1b;
  }
  .r-prog {
    font-size: 0.75rem;
    font-weight: 700;
    color: #1e3a5f;
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
