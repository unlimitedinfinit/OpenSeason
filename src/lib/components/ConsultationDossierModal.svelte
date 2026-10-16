<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { BaselineExtensionAssessment, ConsultationDossier, ExtensionProposal, IssueStressTestResult } from "$lib/types/consultation";
  import type { CaseProfile } from "$lib/types/case";

  interface Props {
    caseId: string;
    profile: CaseProfile | null;
    onClose: () => void;
  }

  let { caseId, profile, onClose }: Props = $props();

  let activeTab = $state<"dossier" | "extension">("dossier");
  let loading = $state(false);
  let error = $state("");
  let statusMessage = $state("");
  let copiedNotice = $state("");

  // Dossier State
  let dossier = $state<ConsultationDossier | null>(null);
  let customQuestionInput = $state("");
  let customQuestions = $state<string[]>([]);

  // Extension Wizard State
  let extensionTarget = $state("Appellant Brief");
  let extensionDays = $state(30);
  let consentObtained = $state(true);
  let extensionProposal = $state<ExtensionProposal | null>(null);
  let baselineAssessment = $state<BaselineExtensionAssessment | null>(null);
  let extensionSubTab = $state<"baseline" | "custom">("baseline");

  onMount(() => {
    loadDossier();
    loadExtensionAssessment();
  });

  async function loadExtensionAssessment() {
    try {
      baselineAssessment = await ipc<BaselineExtensionAssessment>("extension_suggest_cmd", { caseId });
      if (baselineAssessment?.is_extension_recommended) {
        extensionSubTab = "baseline";
      }
    } catch (e) {
      // non-fatal
    }
  }

  function copyText(text: string, label: string) {
    navigator.clipboard.writeText(text);
    copiedNotice = `Copied ${label} to clipboard!`;
    setTimeout(() => { copiedNotice = ""; }, 3000);
  }

  async function loadDossier() {
    loading = true;
    error = "";
    try {
      dossier = await ipc<ConsultationDossier>("consultation_dossier_cmd", {
        caseId,
        customQuestions: customQuestions.length ? customQuestions : null,
      });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function addQuestion() {
    if (!customQuestionInput.trim()) return;
    customQuestions = [...customQuestions, customQuestionInput.trim()];
    customQuestionInput = "";
    loadDossier();
  }

  async function evaluateExtension() {
    loading = true;
    error = "";
    try {
      extensionProposal = await ipc<ExtensionProposal>("extension_evaluate_cmd", {
        caseId,
        targetDeadline: extensionTarget,
        requestedDays: extensionDays,
        consentObtained,
      });
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 overflow-y-auto">
  <div class="bg-white dark:bg-stone-900 rounded-xl shadow-2xl max-w-4xl w-full max-h-[92vh] flex flex-col border border-stone-200 dark:border-stone-800">
    <div class="px-6 py-4 border-b border-stone-200 dark:border-stone-800 flex items-center justify-between">
      <div>
        <div class="flex items-center gap-2">
          <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
            Attorney Consultation & Extension Suite
          </span>
        </div>
        <h2 class="text-xl font-bold text-stone-900 dark:text-stone-100 mt-1">Counsel Preparation & Procedural Enlargement</h2>
      </div>
      <button onclick={onClose} class="text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 text-2xl font-light leading-none">
        &times;
      </button>
    </div>

    <!-- Mode Selector -->
    <div class="px-6 pt-3 border-b border-stone-200 dark:border-stone-800 flex gap-4 text-sm font-medium">
      <button
        onclick={() => (activeTab = "dossier")}
        class="pb-2 border-b-2 {activeTab === 'dossier' ? 'border-emerald-600 text-emerald-600 dark:text-emerald-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
      >
        2-Page Executive Dossier & Devil's Advocate
      </button>
      <button
        onclick={() => { activeTab = "extension"; evaluateExtension(); }}
        class="pb-2 border-b-2 {activeTab === 'extension' ? 'border-emerald-600 text-emerald-600 dark:text-emerald-400 font-semibold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
      >
        Extension Wizard (Consent Letter vs Motion CN 10503)
      </button>
    </div>

    <div class="p-6 overflow-y-auto flex-1 space-y-6">
      {#if error}
        <div class="p-3 bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800 rounded-lg text-sm text-rose-700 dark:text-rose-300">
          {error}
        </div>
      {/if}

      {#if activeTab === "dossier"}
        {#if loading && !dossier}
          <div class="py-12 text-center text-sm text-stone-500">Assembling attorney dossier and running stress-tests...</div>
        {:else if dossier}
          <div class="space-y-6">
            <!-- Header Summary -->
            <div class="bg-stone-50 dark:bg-stone-800/60 p-4 rounded-xl border border-stone-200 dark:border-stone-700">
              <div class="flex items-start justify-between">
                <div>
                  <h3 class="text-base font-bold text-stone-900 dark:text-stone-100">{dossier.title}</h3>
                  <div class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
                    Jurisdiction: <span class="font-semibold text-stone-700 dark:text-stone-300">{dossier.jurisdiction}</span> ·
                    Docket: <span class="font-mono">{dossier.docket_number || "[Pending]"}</span>
                  </div>
                </div>
                {#if dossier.exported_pdf_path}
                  <span class="text-xs font-mono text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/30 px-2.5 py-1 rounded-md border border-emerald-300 dark:border-emerald-700">
                    Saved to exports/consultation_dossier.md
                  </span>
                {/if}
              </div>
            </div>

            <!-- Stress Tested Issues -->
            <div class="space-y-3">
              <h4 class="text-xs font-bold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                Devil's Advocate Counter-Arguments & Vulnerabilities
              </h4>
              {#if dossier.issue_stress_tests.length === 0}
                <div class="p-4 rounded-lg border border-dashed border-stone-300 dark:border-stone-700 text-sm text-stone-500 text-center">
                  No issues currently registered in case profile. The dossier compiles general standard of review and interlocutory checks.
                </div>
              {:else}
                <div class="space-y-3">
                  {#each dossier.issue_stress_tests as test}
                    <div class="p-4 rounded-lg border border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-800/30 space-y-2">
                      <div class="flex items-center justify-between">
                        <div class="text-sm font-bold text-stone-900 dark:text-stone-100">{test.issue_title}</div>
                        <span class="text-[10px] font-bold px-2 py-0.5 rounded {test.overall_risk_rating.includes('High') ? 'bg-rose-100 text-rose-800 dark:bg-rose-900/40 dark:text-rose-300' : 'bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300'}">
                          {test.overall_risk_rating}
                        </span>
                      </div>
                      <p class="text-xs text-stone-600 dark:text-stone-400">{test.procedural_posture_summary}</p>
                      <div class="space-y-2 mt-2 pt-2 border-t border-stone-200 dark:border-stone-700">
                        {#each test.vulnerabilities as v}
                          <div class="text-xs space-y-0.5">
                            <div class="font-semibold text-rose-700 dark:text-rose-400 flex items-center gap-1.5">
                              <span>⚠ [{v.vulnerability_type}] {v.title}</span>
                              <span class="text-[10px] uppercase font-normal text-stone-400">({v.severity})</span>
                            </div>
                            <div class="text-stone-600 dark:text-stone-300 pl-4"><span class="font-medium text-stone-500">Predicted Opposition:</span> {v.opposing_argument_prediction}</div>
                            <div class="text-stone-600 dark:text-stone-300 pl-4"><span class="font-medium text-stone-500">Suggested Action:</span> {v.suggested_cure_or_inquiry}</div>
                            {#if v.citation_anchor}
                              <div class="text-stone-500 font-mono text-[11px] pl-4">Anchor: {v.citation_anchor}</div>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <!-- Exhibits Inventory -->
            <div class="space-y-3">
              <h4 class="text-xs font-bold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                Evidentiary Exhibits & Transcripts (SHA-256 Provenance)
              </h4>
              <div class="border border-stone-200 dark:border-stone-800 rounded-lg overflow-hidden">
                <table class="w-full text-xs text-left">
                  <thead class="bg-stone-100 dark:bg-stone-800 text-stone-600 dark:text-stone-400 border-b border-stone-200 dark:border-stone-700">
                    <tr>
                      <th class="p-2.5 font-semibold">Ex.</th>
                      <th class="p-2.5 font-semibold">Filename</th>
                      <th class="p-2.5 font-semibold">Evidentiary Tier</th>
                      <th class="p-2.5 font-semibold font-mono">SHA-256</th>
                      <th class="p-2.5 font-semibold">Status</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-stone-200 dark:divide-stone-800">
                    {#each dossier.exhibits_inventory as ex}
                      <tr class="hover:bg-stone-50 dark:hover:bg-stone-800/50">
                        <td class="p-2.5 font-bold">{ex.letter}</td>
                        <td class="p-2.5 text-stone-800 dark:text-stone-200">{ex.filename}</td>
                        <td class="p-2.5 text-stone-500">{ex.tier}</td>
                        <td class="p-2.5 font-mono text-stone-400">{ex.sha256_short}</td>
                        <td class="p-2.5">
                          <span class="inline-flex px-1.5 py-0.5 rounded text-[10px] font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300">
                            {ex.relevancy_tag}
                          </span>
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Targeted Questions for Counsel -->
            <div class="space-y-3">
              <h4 class="text-xs font-bold uppercase tracking-wider text-stone-500 dark:text-stone-400">
                Targeted Questions for Counsel
              </h4>
              <div class="space-y-2">
                {#each dossier.targeted_questions_for_counsel as q}
                  <div class="text-xs p-2.5 bg-stone-50 dark:bg-stone-800/40 rounded-lg border border-stone-200 dark:border-stone-700 text-stone-800 dark:text-stone-200">
                    {q}
                  </div>
                {/each}
              </div>
              <div class="flex gap-2 pt-2">
                <input
                  type="text"
                  bind:value={customQuestionInput}
                  placeholder="Add custom question for attorney consultation..."
                  class="flex-1 px-3 py-1.5 text-xs rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 text-stone-900 dark:text-stone-100"
                />
                <button
                  onclick={addQuestion}
                  disabled={!customQuestionInput.trim()}
                  class="px-3 py-1.5 bg-stone-800 hover:bg-stone-700 dark:bg-stone-700 dark:hover:bg-stone-600 disabled:opacity-50 text-white text-xs font-medium rounded-lg"
                >
                  Add Question
                </button>
              </div>
            </div>
          </div>
        {/if}
      {:else}
        <!-- Extension Wizard Tab -->
        <div class="space-y-5">
          {#if copiedNotice}
            <div class="p-2.5 bg-emerald-100 dark:bg-emerald-950/60 border border-emerald-300 dark:border-emerald-700 rounded-lg text-xs font-semibold text-emerald-900 dark:text-emerald-200">
              ✓ {copiedNotice}
            </div>
          {/if}

          <!-- Sub-Tab Mode Selector -->
          <div class="flex gap-2 border-b border-stone-200 dark:border-stone-800 pb-2 text-xs">
            <button
              onclick={() => (extensionSubTab = "baseline")}
              class="px-3 py-1.5 rounded-lg font-medium {extensionSubTab === 'baseline' ? 'bg-amber-100 text-amber-900 dark:bg-amber-900/40 dark:text-amber-200 font-semibold' : 'text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
            >
              🕒 30-Day Baseline Extension (Recommended)
            </button>
            <button
              onclick={() => { extensionSubTab = "custom"; evaluateExtension(); }}
              class="px-3 py-1.5 rounded-lg font-medium {extensionSubTab === 'custom' ? 'bg-stone-200 text-stone-900 dark:bg-stone-800 dark:text-stone-100 font-semibold' : 'text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
            >
              Custom Deadline & Rule Evaluator
            </button>
          </div>

          {#if extensionSubTab === "baseline" && baselineAssessment}
            <div class="space-y-4">
              <div class="p-4 rounded-xl border border-amber-300 dark:border-amber-700/60 bg-amber-50 dark:bg-amber-950/20 space-y-2">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <span class="text-xs font-bold uppercase tracking-wider px-2 py-0.5 rounded bg-amber-200 text-amber-900 dark:bg-amber-900/60 dark:text-amber-200">
                      Standard Practice
                    </span>
                    <span class="text-xs font-semibold text-stone-800 dark:text-stone-200">
                      Rule: {baselineAssessment.governing_rule}
                    </span>
                  </div>
                  {#if baselineAssessment.recommended_form_number}
                    <span class="text-xs font-mono text-stone-600 dark:text-stone-400">
                      Form: {baselineAssessment.recommended_form_number}
                    </span>
                  {/if}
                </div>
                <p class="text-xs text-stone-700 dark:text-stone-300 leading-relaxed">
                  When a case file is newly opened or incomplete, self-represented litigants routinely need 30 days to obtain verbatim transcripts from the court reporter and assemble the official record. Courts recognize this as good cause.
                </p>
                {#if baselineAssessment.missing_elements.length > 0}
                  <div class="pt-1">
                    <span class="text-[11px] font-bold uppercase tracking-wider text-amber-900 dark:text-amber-300">Pending Case Elements:</span>
                    <ul class="text-xs text-stone-600 dark:text-stone-400 list-disc pl-5 space-y-0.5 mt-0.5">
                      {#each baselineAssessment.missing_elements as elem}
                        <li>{elem}</li>
                      {/each}
                    </ul>
                  </div>
                {/if}
              </div>

              <!-- Step 1: Conferral & Consent -->
              <div class="p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="text-xs font-bold uppercase tracking-wider text-emerald-700 dark:text-emerald-400">
                      Step 1: Informal Conferral Email / Letter (Saves Fees & Court Time)
                    </h4>
                    <p class="text-[11px] text-stone-500 mt-0.5">
                      Send this email to opposing counsel. If consented, file a simple consent stipulation letter.
                    </p>
                  </div>
                  <button
                    onclick={() => copyText(baselineAssessment?.conferral_letter_draft || "", "Conferral Letter")}
                    class="px-2.5 py-1 text-xs font-medium rounded-md bg-stone-100 hover:bg-stone-200 dark:bg-stone-800 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 border border-stone-300 dark:border-stone-700"
                  >
                    📋 Copy Email
                  </button>
                </div>
                <pre class="text-xs p-3 bg-stone-50 dark:bg-stone-950 rounded-lg border border-stone-200 dark:border-stone-800 font-mono whitespace-pre-wrap">{baselineAssessment.conferral_letter_draft}</pre>
              </div>

              <!-- Step 2: Formal Motion for Enlargement -->
              <div class="p-4 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900/60 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <h4 class="text-xs font-bold uppercase tracking-wider text-amber-700 dark:text-amber-400">
                      Step 2: Formal Motion for Enlargement (If Consent Is Withheld or Unanswered)
                    </h4>
                    <p class="text-[11px] text-stone-500 mt-0.5">
                      File with the court clerk prior to the deadline, accompanied by the good cause certification.
                    </p>
                  </div>
                  <button
                    onclick={() => copyText(baselineAssessment?.formal_motion_draft || "", "Formal Motion")}
                    class="px-2.5 py-1 text-xs font-medium rounded-md bg-stone-100 hover:bg-stone-200 dark:bg-stone-800 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 border border-stone-300 dark:border-stone-700"
                  >
                    📋 Copy Motion
                  </button>
                </div>
                <pre class="text-xs p-3 bg-stone-50 dark:bg-stone-950 rounded-lg border border-stone-200 dark:border-stone-800 font-mono whitespace-pre-wrap">{baselineAssessment.formal_motion_draft}</pre>
              </div>
            </div>
          {:else}
            <!-- Custom Calculator -->
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div class="space-y-1">
                <label class="text-xs font-semibold text-stone-700 dark:text-stone-300">Target Filing / Deadline</label>
                <select
                  bind:value={extensionTarget}
                  onchange={() => evaluateExtension()}
                  class="w-full px-3 py-2 text-xs rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 text-stone-900 dark:text-stone-100"
                >
                  <option value="Appellant Brief">Appellant Brief</option>
                  <option value="Motion Response / Opposition">Motion Response / Opposition</option>
                  <option value="Notice of Appeal">Notice of Appeal (Appeal Clock)</option>
                  <option value="Trial Discovery Response">Trial Discovery Response</option>
                </select>
              </div>

              <div class="space-y-1">
                <label class="text-xs font-semibold text-stone-700 dark:text-stone-300">Requested Days</label>
                <input
                  type="number"
                  bind:value={extensionDays}
                  oninput={() => evaluateExtension()}
                  min="1"
                  max="90"
                  class="w-full px-3 py-2 text-xs rounded-lg border border-stone-300 dark:border-stone-700 bg-white dark:bg-stone-800 text-stone-900 dark:text-stone-100"
                />
              </div>

              <div class="space-y-1 flex flex-col justify-end">
                <label class="flex items-center gap-2 cursor-pointer pb-2">
                  <input
                    type="checkbox"
                    bind:checked={consentObtained}
                    onchange={() => evaluateExtension()}
                    class="rounded border-stone-300 text-emerald-600 focus:ring-emerald-500"
                  />
                  <span class="text-xs font-semibold text-stone-700 dark:text-stone-300">Opposing Party Consents</span>
                </label>
              </div>
            </div>

            {#if extensionProposal}
              <div class="p-4 rounded-xl border {extensionProposal.route === 'ProhibitedByRule' ? 'bg-rose-50 dark:bg-rose-950/30 border-rose-300 dark:border-rose-700' : extensionProposal.route === 'ConsentStipulationLetter' ? 'bg-emerald-50 dark:bg-emerald-950/30 border-emerald-300 dark:border-emerald-700' : 'bg-amber-50 dark:bg-amber-950/30 border-amber-300 dark:border-amber-700'} space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded {extensionProposal.route === 'ProhibitedByRule' ? 'bg-rose-200 text-rose-900 dark:bg-rose-900/60 dark:text-rose-200' : extensionProposal.route === 'ConsentStipulationLetter' ? 'bg-emerald-200 text-emerald-900 dark:bg-emerald-900/60 dark:text-emerald-200' : 'bg-amber-200 text-amber-900 dark:bg-amber-900/60 dark:text-amber-200'}">
                      Route: {extensionProposal.route}
                    </span>
                    <span class="text-xs text-stone-600 dark:text-stone-300 ml-2">Rule: <strong>{extensionProposal.governing_rule}</strong></span>
                  </div>
                  {#if extensionProposal.recommended_form_number}
                    <span class="text-xs font-mono text-stone-600 dark:text-stone-400">
                      Form: {extensionProposal.recommended_form_number}
                    </span>
                  {/if}
                </div>

                {#if extensionProposal.non_alterable_warning}
                  <div class="p-2.5 bg-rose-100 dark:bg-rose-900/40 rounded-lg text-xs font-semibold text-rose-900 dark:text-rose-200">
                    ⚠ {extensionProposal.non_alterable_warning}
                  </div>
                {/if}

                {#if !extensionProposal.rendered_document_text}
                  <p class="text-xs text-stone-600 dark:text-stone-400">
                    This enlargement cannot be granted under the specified rule. See instructions below.
                  </p>
                {:else}
                  <div class="space-y-1">
                    <div class="flex items-center justify-between">
                      <div class="text-xs font-semibold text-stone-700 dark:text-stone-300">Generated Submission Text:</div>
                      <button
                        onclick={() => copyText(extensionProposal?.rendered_document_text || "", "Submission Text")}
                        class="px-2 py-0.5 text-xs font-medium rounded bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300 border border-stone-300 dark:border-stone-700"
                      >
                        📋 Copy
                      </button>
                    </div>
                    <pre class="text-xs p-3 bg-white dark:bg-stone-900 rounded-lg border border-stone-200 dark:border-stone-700 font-mono whitespace-pre-wrap">{extensionProposal.rendered_document_text}</pre>
                  </div>
                {/if}

                <div class="space-y-1 pt-1">
                  <div class="text-[11px] font-bold uppercase tracking-wider text-stone-500">Next Steps:</div>
                  <ul class="text-xs text-stone-600 dark:text-stone-400 list-disc pl-5 space-y-0.5">
                    {#each extensionProposal.step_instructions as step}
                      <li>{step}</li>
                    {/each}
                  </ul>
                </div>
              </div>
            {/if}
          {/if}
        </div>
      {/if}
    </div>

    <div class="px-6 py-4 border-t border-stone-200 dark:border-stone-800 flex items-center justify-between bg-stone-50 dark:bg-stone-900/50">
      <div class="text-xs text-stone-500 dark:text-stone-400">
        Administrative tool only. OpenSeason never files or serves. Always review with a licensed attorney.
      </div>
      <div class="flex items-center gap-3">
        <button
          onclick={onClose}
          class="px-4 py-2 border border-stone-300 dark:border-stone-700 text-stone-700 dark:text-stone-300 rounded-lg text-sm hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
        >
          Close
        </button>
        {#if activeTab === "dossier"}
          <button
            onclick={loadDossier}
            disabled={loading}
            class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors shadow-sm"
          >
            {loading ? "Re-generating..." : "Refresh Dossier"}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>
