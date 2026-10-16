<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseProfile } from "$lib/types/case";
  import type { AccountabilityRoute, AccountabilityResource } from "$lib/types/record";

  interface Props {
    open: boolean;
    caseId: string;
    profile?: CaseProfile | null;
    preselectedIssueId?: string | null;
    onclose: () => void;
  }

  let { open = $bindable(false), caseId, profile = null, preselectedIssueId = null, onclose }: Props = $props();

  let activeTrack = $state<"review" | "judicial" | "attorney" | "records">("review");
  let loading = $state(false);
  let routes = $state<AccountabilityRoute[]>([]);
  let resources = $state<AccountabilityResource[]>([]);
  let errorMessage = $state("");

  async function loadData() {
    if (!caseId) return;
    loading = true;
    errorMessage = "";
    try {
      const state = profile?.governing_state || "NJ";
      const [routeList, resList] = await Promise.all([
        ipc<AccountabilityRoute[]>("accountability_routes_cmd", {
          caseId,
          issueId: preselectedIssueId || undefined,
        }),
        ipc<AccountabilityResource[]>("accountability_resources_cmd", {
          state,
          category: activeTrack === "review" ? "judicial" : activeTrack,
        }),
      ]);
      routes = routeList;
      resources = resList;
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to load accountability tracks.";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) {
      loadData();
    }
  });

  let filteredResources = $derived(
    resources.filter((r) => {
      if (activeTrack === "review") return r.category === "judicial";
      return r.category === activeTrack;
    })
  );
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-stone-950/60 backdrop-blur-xs overflow-y-auto">
    <div class="relative w-full max-w-3xl my-8 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Modal Header -->
      <div class="p-5 border-b border-stone-100 dark:border-stone-800 flex items-center justify-between gap-3 bg-stone-50/70 dark:bg-stone-950/40">
        <div>
          <div class="flex items-center gap-2">
            <span class="inline-flex items-center justify-center w-6 h-6 rounded-md bg-teal-500/10 text-teal-600 dark:text-teal-400 font-bold text-xs">
              RT
            </span>
            <h3 class="text-base font-semibold text-stone-900 dark:text-stone-100">
              Procedural & Accountability Routing Engine
            </h3>
          </div>
          <p class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
            Strict procedural separation of court merits review, misconduct grievances, attorney ethics, and records requests.
          </p>
        </div>
        <button
          onclick={onclose}
          class="p-1 rounded-lg text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Mandatory Non-Tolling Alert Banner -->
      <div class="px-5 py-3 bg-rose-500/10 border-b border-rose-500/20 text-xs flex items-start gap-2.5">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div class="space-y-0.5">
          <span class="font-bold text-rose-900 dark:text-rose-300">CRITICAL LEGAL PROCEDURAL NOTICE:</span>
          <p class="text-[11px] text-rose-800/90 dark:text-rose-300/80 leading-relaxed">
            Filing an ethics complaint against a judge or attorney <strong>NEVER tolls, stays, or extends court deadlines</strong>, and <strong>CANNOT overturn a court order</strong>. If a judge made an incorrect factual or legal ruling, your only legal remedies are <strong>Reconsideration in the Trial Court</strong> or a <strong>Notice of Appeal</strong>.
          </p>
        </div>
      </div>

      <!-- Track Switcher Tabs -->
      <div class="flex border-b border-stone-200 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-950/30 px-5 pt-2 gap-2 text-xs font-medium overflow-x-auto">
        <button
          onclick={() => (activeTrack = "review")}
          class="pb-2.5 px-3 border-b-2 transition whitespace-nowrap {activeTrack === 'review' ? 'border-teal-600 text-teal-600 dark:text-teal-400 font-bold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
        >
          Track 1: Merits & Court Review (Tolls Deadlines)
        </button>
        <button
          onclick={() => (activeTrack = "judicial")}
          class="pb-2.5 px-3 border-b-2 transition whitespace-nowrap {activeTrack === 'judicial' ? 'border-rose-600 text-rose-600 dark:text-rose-400 font-bold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
        >
          Track 2: Judicial Misconduct (ACJC / CJD)
        </button>
        <button
          onclick={() => (activeTrack = "attorney")}
          class="pb-2.5 px-3 border-b-2 transition whitespace-nowrap {activeTrack === 'attorney' ? 'border-amber-600 text-amber-600 dark:text-amber-400 font-bold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
        >
          Track 3: Attorney Ethics (OAE)
        </button>
        <button
          onclick={() => (activeTrack = "records")}
          class="pb-2.5 px-3 border-b-2 transition whitespace-nowrap {activeTrack === 'records' ? 'border-blue-600 text-blue-600 dark:text-blue-400 font-bold' : 'border-transparent text-stone-500 hover:text-stone-800 dark:hover:text-stone-200'}"
        >
          Track 4: Public Records / FOIA (CN 10200)
        </button>
      </div>

      <!-- Modal Content Body -->
      <div class="p-5 space-y-4 overflow-y-auto flex-1">
        {#if errorMessage}
          <div class="p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 text-xs text-rose-700 dark:text-rose-400">
            {errorMessage}
          </div>
        {/if}

        {#if activeTrack === "review"}
          <!-- Merits Review Routes -->
          <div class="space-y-3">
            <h4 class="text-xs font-bold text-stone-900 dark:text-stone-100 uppercase tracking-wider">
              Available Judicial Remedies in Court
            </h4>
            <div class="space-y-3">
              {#each routes as route}
                <div class="p-3.5 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-950 space-y-2 text-xs">
                  <div class="flex flex-wrap items-center justify-between gap-2">
                    <span class="font-bold text-stone-900 dark:text-stone-100">{route.title}</span>
                    <span class="px-2 py-0.5 rounded text-[10px] font-mono bg-teal-50 dark:bg-teal-950/40 text-teal-700 dark:text-teal-300 border border-teal-200 dark:border-teal-800">
                      {route.rule_citation}
                    </span>
                  </div>
                  <p class="text-stone-600 dark:text-stone-300 text-[11px] leading-relaxed">
                    <strong>Standard:</strong> {route.governing_standard}
                  </p>
                  {#if route.deadline_notice}
                    <div class="text-[11px] font-semibold text-amber-700 dark:text-amber-400">
                      Deadline: {route.deadline_notice}
                    </div>
                  {/if}
                  <div class="p-2 rounded bg-stone-50 dark:bg-stone-900 border border-stone-100 dark:border-stone-800 text-[11px] text-stone-500">
                    {route.warning_notice}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <!-- Disciplinary / Records Resources List -->
          <div class="space-y-3">
            <h4 class="text-xs font-bold text-stone-900 dark:text-stone-100 uppercase tracking-wider">
              Authoritative Agency Information & Forms ({profile?.governing_state || "NJ"})
            </h4>
            <div class="space-y-3">
              {#each filteredResources as resource}
                <div class="p-3.5 rounded-xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-950 space-y-2 text-xs">
                  <div class="flex flex-wrap items-center justify-between gap-2">
                    <div>
                      <span class="font-bold text-stone-900 dark:text-stone-100">{resource.title}</span>
                      <div class="text-[11px] text-stone-500">{resource.agency} · Jurisdiction: {resource.state}</div>
                    </div>
                    {#if resource.form_number}
                      <span class="px-2 py-0.5 rounded text-[10px] font-semibold bg-stone-100 dark:bg-stone-800 text-stone-700 dark:text-stone-300">
                        Form: {resource.form_number}
                      </span>
                    {/if}
                  </div>

                  <p class="text-stone-600 dark:text-stone-300 text-[11px] leading-relaxed">
                    <strong>Scope:</strong> {resource.jurisdictional_scope}
                  </p>

                  <div class="p-2 rounded bg-rose-50 dark:bg-rose-950/20 border border-rose-200 dark:border-rose-900/40 text-[11px] font-medium text-rose-800 dark:text-rose-300">
                    {resource.non_appeal_warning}
                  </div>

                  <div class="flex flex-wrap items-center justify-between gap-2 pt-1">
                    <span class="text-[10px] text-stone-400 font-medium">Fee: {resource.fee_schedule}</span>
                    <div class="flex items-center gap-2">
                      {#if resource.direct_pdf_url}
                        <a
                          href={resource.direct_pdf_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="px-2.5 py-1 rounded border border-stone-300 dark:border-stone-700 hover:bg-stone-100 dark:hover:bg-stone-800 text-stone-700 dark:text-stone-200 font-medium transition"
                        >
                          Official PDF Form &rarr;
                        </a>
                      {/if}
                      <a
                        href={resource.portal_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="px-2.5 py-1 rounded bg-teal-600 hover:bg-teal-700 text-white font-medium transition"
                      >
                        Agency Portal &rarr;
                      </a>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="p-4 border-t border-stone-100 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-950/40 flex justify-end">
        <button
          onclick={onclose}
          class="px-4 py-1.5 text-xs font-medium rounded-lg bg-stone-200 dark:bg-stone-800 hover:bg-stone-300 dark:hover:bg-stone-700 text-stone-800 dark:text-stone-200 transition"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
