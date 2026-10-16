<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseProfile, ServiceRecipient, ServiceCertParams, ServiceCertResult, ProposedOrderResult } from "$lib/types/case";

  interface Props {
    caseId: string;
    profile?: CaseProfile | null;
    defaultMotionTitle?: string;
  }

  let { caseId, profile = null, defaultMotionTitle = "" }: Props = $props();

  // Active Tab: Certificate of Service or Proposed Order
  let activeTab = $state<"cert" | "order">("cert");

  // Certificate State
  let filingTitle = $state(defaultMotionTitle || "Motion for Reconsideration");
  let serviceDate = $state(new Date().toISOString().slice(0, 10));
  let serviceMethod = $state("electronic");
  let signatureName = $state("");
  let recipients = $state<ServiceRecipient[]>([]);

  // Form of Order State
  let orderMotionTitle = $state(defaultMotionTitle || "Motion for Reconsideration");
  let orderRelief = $state("ORDERED that the motion is GRANTED in all respects.");

  // Execution State
  let generatingCert = $state(false);
  let generatingOrder = $state(false);
  let certResult = $state<ServiceCertResult | null>(null);
  let orderResult = $state<ProposedOrderResult | null>(null);
  let errorMessage = $state("");
  let successMessage = $state("");

  // New recipient form fields
  let newName = $state("");
  let newRole = $state("Counsel for Defendant");
  let newAddress = $state("");
  let newEmail = $state("");

  onMount(() => {
    initFromProfile();
  });

  $effect(() => {
    if (defaultMotionTitle && !filingTitle) {
      filingTitle = defaultMotionTitle;
      orderMotionTitle = defaultMotionTitle;
    }
  });

  function initFromProfile() {
    if (!profile) return;
    signatureName = profile.filer.signature_name || profile.filer.name || "";

    // Auto-populate recipients from opposing parties
    const opp = profile.caption.defendants.length > 0
      ? profile.caption.defendants
      : profile.caption.appellees;

    if (opp.length > 0 && recipients.length === 0) {
      recipients = opp.map((p) => ({
        name: p.counsel ? `${p.counsel} (Counsel for ${p.name})` : p.name,
        role: p.role || "Opposing Party",
        address: p.address || "",
        email: p.email || "",
      }));
    }
  }

  function addRecipient() {
    if (!newName.trim()) return;
    recipients = [
      ...recipients,
      {
        name: newName.trim(),
        role: newRole.trim(),
        address: newAddress.trim(),
        email: newEmail.trim(),
      },
    ];
    newName = "";
    newAddress = "";
    newEmail = "";
  }

  function removeRecipient(idx: number) {
    recipients = recipients.filter((_, i) => i !== idx);
  }

  async function generateCert() {
    if (!filingTitle.trim()) {
      errorMessage = "Please enter the title of the document being served.";
      return;
    }
    if (recipients.length === 0) {
      errorMessage = "Please add at least one recipient for service.";
      return;
    }

    generatingCert = true;
    errorMessage = "";
    successMessage = "";
    certResult = null;

    try {
      const params: ServiceCertParams = {
        filing_title: filingTitle.trim(),
        service_date: serviceDate,
        method: serviceMethod,
        recipients,
        signature_name: signatureName.trim(),
      };

      const res = await ipc<ServiceCertResult>("generate_certificate_of_service_cmd", {
        caseId,
        params,
      });

      certResult = res;
      successMessage = "Certificate of Service generated successfully!";
    } catch (e) {
      errorMessage = String(e);
    } finally {
      generatingCert = false;
    }
  }

  async function generateOrder() {
    if (!orderMotionTitle.trim()) {
      errorMessage = "Please enter the motion title for the proposed order.";
      return;
    }

    generatingOrder = true;
    errorMessage = "";
    successMessage = "";
    orderResult = null;

    try {
      const res = await ipc<ProposedOrderResult>("generate_proposed_order_cmd", {
        caseId,
        motionTitle: orderMotionTitle.trim(),
        relief: orderRelief.trim(),
      });

      orderResult = res;
      successMessage = "Proposed Form of Order generated in Word and PDF!";
    } catch (e) {
      errorMessage = String(e);
    } finally {
      generatingOrder = false;
    }
  }
</script>

<div class="rounded-2xl border border-emerald-500/30 bg-zinc-900/80 p-5 space-y-5 shadow-lg relative overflow-hidden">
  <!-- Header with pill switcher -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-zinc-800 pb-4">
    <div class="flex items-center gap-3">
      <div class="h-11 w-11 rounded-xl bg-emerald-500/15 border border-emerald-500/30 flex items-center justify-center text-2xl shadow-inner">
        {activeTab === 'cert' ? '📜' : '⚖️'}
      </div>
      <div>
        <div class="flex items-center gap-2">
          <h3 class="font-bold text-sm text-zinc-100">
            {activeTab === 'cert' ? 'Certificate of Service Generator' : 'Proposed Form of Order Generator'}
          </h3>
          <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
            Court Required
          </span>
        </div>
        <p class="text-xs text-zinc-400">
          Assemble mandatory proof of service and judge-ready signature forms without manual retyping.
        </p>
      </div>
    </div>

    <!-- Toggle Buttons -->
    <div class="flex items-center gap-1.5 bg-zinc-950/80 p-1 rounded-xl border border-zinc-800 self-stretch sm:self-auto">
      <button
        onclick={() => { activeTab = 'cert'; errorMessage = ''; }}
        class="flex-1 sm:flex-initial px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-all {activeTab === 'cert' ? 'bg-emerald-600 text-white shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <span>📜</span> Certificate of Service
      </button>
      <button
        onclick={() => { activeTab = 'order'; errorMessage = ''; }}
        class="flex-1 sm:flex-initial px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-all {activeTab === 'order' ? 'bg-primary text-primary-foreground shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
      >
        <span>⚖️</span> Proposed Order
      </button>
    </div>
  </div>

  <!-- Messages -->
  {#if successMessage}
    <div class="rounded-xl border border-emerald-500/30 bg-emerald-950/30 p-3 text-xs text-emerald-300 flex items-center gap-2">
      <span>✅</span>
      <span>{successMessage}</span>
    </div>
  {/if}

  {#if errorMessage}
    <div class="rounded-xl border border-rose-500/30 bg-rose-950/30 p-3 text-xs text-rose-300 flex items-center gap-2">
      <span>⚠️</span>
      <span>{errorMessage}</span>
    </div>
  {/if}

  <!-- TAB 1: CERTIFICATE OF SERVICE -->
  {#if activeTab === 'cert'}
    <div class="space-y-4">
      <div class="grid grid-cols-1 sm:grid-cols-12 gap-3">
        <!-- Filing Title -->
        <div class="sm:col-span-6 space-y-1">
          <label class="text-xs font-semibold text-zinc-300">Document Served</label>
          <input
            type="text"
            bind:value={filingTitle}
            placeholder="e.g. Notice of Motion for Reconsideration and Supporting Brief"
            class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-emerald-500 focus:outline-none"
          />
        </div>

        <!-- Service Date -->
        <div class="sm:col-span-3 space-y-1">
          <label class="text-xs font-semibold text-zinc-300">Date of Service</label>
          <input
            type="date"
            bind:value={serviceDate}
            class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 focus:border-emerald-500 focus:outline-none"
          />
        </div>

        <!-- Service Method -->
        <div class="sm:col-span-3 space-y-1">
          <label class="text-xs font-semibold text-zinc-300">Primary Method</label>
          <select
            bind:value={serviceMethod}
            class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 focus:border-emerald-500 focus:outline-none"
          >
            <option value="electronic">🌐 Electronic Court Filing / Portal</option>
            <option value="mail">📮 Certified First-Class Mail</option>
            <option value="email">📧 Direct Electronic Mail</option>
            <option value="hand_delivery">🤝 Personal Hand Delivery</option>
          </select>
        </div>
      </div>

      <!-- Signer Name -->
      <div class="space-y-1">
        <label class="text-xs font-semibold text-zinc-300">Certifying Person's Name</label>
        <input
          type="text"
          bind:value={signatureName}
          placeholder="e.g. Joshua Abrams, Pro Se"
          class="w-full sm:w-1/2 rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-emerald-500 focus:outline-none"
        />
      </div>

      <!-- Recipients Section -->
      <div class="space-y-2 pt-2 border-t border-zinc-800">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold text-zinc-200">Served Parties & Opposing Counsel ({recipients.length})</span>
            <button
              onclick={initFromProfile}
              type="button"
              class="text-[11px] text-emerald-400 hover:text-emerald-300 underline font-medium"
            >
              🔄 Re-sync from case parties
            </button>
          </div>
        </div>

        <!-- Recipient List -->
        {#if recipients.length === 0}
          <div class="p-4 rounded-xl border border-dashed border-zinc-800 bg-zinc-950/40 text-center text-xs text-zinc-400">
            No recipients added yet. Add an opposing counsel or party below.
          </div>
        {:else}
          <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
            {#each recipients as r, idx}
              <div class="flex items-center justify-between gap-3 p-2.5 rounded-lg border border-zinc-800 bg-zinc-950/50 text-xs">
                <div>
                  <div class="font-semibold text-zinc-200 flex items-center gap-2">
                    <span>👤 {r.name}</span>
                    <span class="text-[10px] px-2 py-0.2 rounded-full bg-zinc-800 text-zinc-400">{r.role}</span>
                  </div>
                  <div class="text-[11px] text-zinc-400 flex items-center gap-3 mt-0.5">
                    {#if r.address}<span>📍 {r.address}</span>{/if}
                    {#if r.email}<span>📧 {r.email}</span>{/if}
                  </div>
                </div>
                <button
                  type="button"
                  onclick={() => removeRecipient(idx)}
                  class="text-zinc-500 hover:text-rose-400 text-sm px-1.5 py-0.5 rounded"
                  title="Remove recipient"
                >
                  ✕
                </button>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Add Recipient Form -->
        <div class="grid grid-cols-1 sm:grid-cols-12 gap-2 pt-2 bg-zinc-950/30 p-3 rounded-xl border border-zinc-800/80">
          <div class="sm:col-span-3">
            <input
              type="text"
              bind:value={newName}
              placeholder="Name or Law Firm"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
            />
          </div>
          <div class="sm:col-span-3">
            <input
              type="text"
              bind:value={newRole}
              placeholder="Role (e.g. Counsel for Appellee)"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
            />
          </div>
          <div class="sm:col-span-3">
            <input
              type="text"
              bind:value={newAddress}
              placeholder="Postal Address"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
            />
          </div>
          <div class="sm:col-span-2">
            <input
              type="email"
              bind:value={newEmail}
              placeholder="Email"
              class="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none"
            />
          </div>
          <div class="sm:col-span-1">
            <button
              type="button"
              onclick={addRecipient}
              class="w-full h-full py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-bold transition-colors"
            >
              + Add
            </button>
          </div>
        </div>
      </div>

      <!-- Action Button -->
      <div class="flex items-center justify-between pt-3 border-t border-zinc-800">
        <div class="text-[11px] text-zinc-500">
          Output: exports/certificate-of-service.pdf & .txt
        </div>
        <button
          onclick={generateCert}
          disabled={generatingCert}
          class="px-4 py-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white text-xs font-bold flex items-center gap-2 shadow-md transition-all active:scale-95 disabled:opacity-50"
        >
          <span>📜</span> {generatingCert ? 'Compiling Proof...' : 'Generate Certificate of Service'}
        </button>
      </div>

      <!-- Result Card -->
      {#if certResult}
        <div class="rounded-xl border border-emerald-500/40 bg-emerald-950/20 p-4 space-y-2">
          <div class="text-xs font-semibold text-emerald-300 flex items-center gap-1.5">
            <span>✨</span> Certificate of Service Created:
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs font-mono">
            <div class="rounded-lg bg-zinc-900/80 p-2.5 border border-zinc-700/60 text-zinc-300 break-all">
              <span class="text-emerald-400 font-bold">PDF:</span> {certResult.cert_pdf}
            </div>
            <div class="rounded-lg bg-zinc-900/80 p-2.5 border border-zinc-700/60 text-zinc-300 break-all">
              <span class="text-teal-400 font-bold">Audit Text:</span> {certResult.cert_text}
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <!-- TAB 2: PROPOSED FORM OF ORDER -->
  {#if activeTab === 'order'}
    <div class="space-y-4">
      <div class="space-y-1">
        <label class="text-xs font-semibold text-zinc-300">Underlying Motion Title</label>
        <input
          type="text"
          bind:value={orderMotionTitle}
          placeholder="e.g. Motion for Reconsideration of Order Dismissing Appeal"
          class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 focus:border-primary focus:outline-none"
        />
      </div>

      <div class="space-y-1">
        <label class="text-xs font-semibold text-zinc-300">Proposed Judicial Order Clauses</label>
        <p class="text-[11px] text-zinc-400">
          The operative decretal language the judge will sign. Starts with "ORDERED that...".
        </p>
        <textarea
          bind:value={orderRelief}
          rows="5"
          placeholder="ORDERED that Appellant's Motion for Reconsideration is hereby GRANTED; and it is further&#10;ORDERED that the dismissal order dated January 16 is VACATED..."
          class="w-full rounded-xl border border-zinc-700 bg-zinc-950/80 p-3 text-xs text-zinc-100 placeholder-zinc-600 font-mono leading-relaxed focus:border-primary focus:outline-none"
        ></textarea>
      </div>

      <!-- Action Button -->
      <div class="flex items-center justify-between pt-3 border-t border-zinc-800">
        <div class="text-[11px] text-zinc-500">
          Includes Judge signature line and date blank. Exports to PDF & editable Word DOCX.
        </div>
        <button
          onclick={generateOrder}
          disabled={generatingOrder}
          class="px-4 py-2 rounded-xl bg-primary hover:bg-primary/90 text-primary-foreground text-xs font-bold flex items-center gap-2 shadow-sm transition-all active:scale-95 disabled:opacity-50"
        >
          <span>⚖️</span> {generatingOrder ? 'Assembling Order...' : 'Generate Proposed Order'}
        </button>
      </div>

      <!-- Result Card -->
      {#if orderResult}
        <div class="rounded-xl border border-primary/30 bg-primary/10 p-4 space-y-2">
          <div class="text-xs font-semibold text-primary flex items-center gap-1.5">
            <span>✨</span> Proposed Order Ready:
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs font-mono">
            <div class="rounded-lg bg-zinc-900/80 p-2.5 border border-zinc-700/60 text-zinc-300 break-all">
              <span class="text-primary font-bold">PDF:</span> {orderResult.order_pdf}
            </div>
            <div class="rounded-lg bg-zinc-900/80 p-2.5 border border-zinc-700/60 text-zinc-300 break-all">
              <span class="text-primary font-bold">Word DOCX:</span> {orderResult.order_docx}
            </div>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
