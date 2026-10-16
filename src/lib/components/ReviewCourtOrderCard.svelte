<script lang="ts">
  import type { DecodedOrder, MailAuditReport } from "$lib/types/case";

  let {
    order = $bindable<DecodedOrder | null>(null),
    mailAudit = $bindable<MailAuditReport | null>(null),
    onSelectAction = (_action: string) => {},
    onDismiss = () => {}
  } = $props<{
    order: DecodedOrder | null;
    mailAudit?: MailAuditReport | null;
    onSelectAction?: (action: string) => void;
    onDismiss?: () => void;
  }>();

  let activeTab = $state<"overview" | "operative" | "traps">("overview");
</script>

{#if order}
  <div class="rounded-xl border border-amber-500/30 bg-amber-950/10 backdrop-blur-md p-5 text-zinc-100 shadow-xl space-y-4">
    <!-- Header with Status Badges -->
    <div class="flex items-start justify-between gap-3 border-b border-amber-500/20 pb-3">
      <div>
        <div class="flex items-center gap-2">
          <span class="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30">
            Review required
          </span>
          {#if order.candidate_deadline}
            <span class="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-semibold bg-red-500/20 text-red-300 border border-red-500/30">
              Deadline unverified
            </span>
          {/if}
          {#if mailAudit?.status === "flagged_risk"}
            <span class="inline-flex items-center px-2 py-0.5 rounded text-[11px] font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/30">
              Mail Delay Trap Detected
            </span>
          {/if}
        </div>
        <h3 class="text-base font-bold text-amber-200 mt-1">{order.title}</h3>
        <p class="text-xs text-zinc-400">
          {order.court_name || "Court of Record"} {order.docket_number ? `· Docket ${order.docket_number}` : ""}
        </p>
      </div>

      <button
        onclick={onDismiss}
        class="text-zinc-400 hover:text-zinc-200 text-xs px-2 py-1 rounded bg-zinc-800/60 hover:bg-zinc-800 transition"
      >
        Dismiss
      </button>
    </div>

    <!-- What JustLegal Identified -->
    <div class="bg-zinc-900/60 rounded-lg p-3.5 border border-zinc-800/80 space-y-2">
      <h4 class="text-xs font-semibold uppercase tracking-wider text-amber-400/90">What JustLegal Identified</h4>
      <p class="text-xs text-zinc-200 leading-relaxed">
        {order.operative_clauses[0] || "This court order may require a response or filing within a specified window."}
      </p>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-2 pt-2 text-[11px]">
        <div class="bg-zinc-800/50 p-2 rounded border border-zinc-700/40">
          <span class="text-zinc-400 block">Signer Provenance:</span>
          <span class="font-medium text-zinc-200">{order.signer.kind === "ByTheCourt" ? "BY THE COURT (Screening Division)" : order.signer.detail?.name || order.signer.kind}</span>
        </div>
        <div class="bg-zinc-800/50 p-2 rounded border border-zinc-700/40">
          <span class="text-zinc-400 block">Source Location:</span>
          <span class="font-medium text-zinc-200">Page {order.source_page || 1}</span>
        </div>
        <div class="bg-zinc-800/50 p-2 rounded border border-zinc-700/40">
          <span class="text-zinc-400 block">Candidate Deadline:</span>
          <span class="font-semibold text-amber-300">{order.candidate_deadline || "Needs Verification"}</span>
        </div>
      </div>

      <!-- Jurisdictional Trap Alert if Mail Audit detected risk -->
      {#if mailAudit && mailAudit.jurisdictional_traps.length > 0}
        <div class="mt-2.5 p-2.5 rounded bg-red-950/30 border border-red-500/40 text-xs space-y-1">
          <span class="font-semibold text-red-300 flex items-center gap-1.5">
            <svg class="w-4 h-4 text-red-400 shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            Jurisdictional Trap Warning ({mailAudit.jurisdiction_receipt_rule})
          </span>
          {#each mailAudit.jurisdictional_traps as trap}
            <p class="text-[11px] text-zinc-300 leading-normal">• {trap}</p>
          {/each}
        </div>
      {/if}
    </div>

    <!-- What Would You Like To Do? 4-Action Workspace -->
    <div class="space-y-2">
      <h4 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">What would you like to do?</h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
        <button
          onclick={() => onSelectAction("review_rule")}
          class="flex items-start gap-3 p-3 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800 hover:border-amber-500/50 transition text-left group"
        >
          <span class="p-1.5 rounded bg-amber-500/10 text-amber-400 group-hover:bg-amber-500/20">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" /></svg>
          </span>
          <div>
            <div class="text-xs font-semibold text-zinc-200 group-hover:text-amber-200">Review the governing rule</div>
            <div class="text-[11px] text-zinc-400">Inspect cited rules ({order.governing_rules_cited.join(", ") || "Court Rules"}) and check tolling exceptions</div>
          </div>
        </button>

        <button
          onclick={() => onSelectAction("check_record")}
          class="flex items-start gap-3 p-3 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800 hover:border-amber-500/50 transition text-left group"
        >
          <span class="p-1.5 rounded bg-blue-500/10 text-blue-400 group-hover:bg-blue-500/20">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" /></svg>
          </span>
          <div>
            <div class="text-xs font-semibold text-zinc-200 group-hover:text-blue-200">Check the existing record</div>
            <div class="text-[11px] text-zinc-400">Compare postmark vs. clerk physical receipt timestamps</div>
          </div>
        </button>

        <button
          onclick={() => onSelectAction("explore_options")}
          class="flex items-start gap-3 p-3 rounded-lg border border-zinc-700/60 bg-zinc-800/40 hover:bg-zinc-800 hover:border-amber-500/50 transition text-left group"
        >
          <span class="p-1.5 rounded bg-purple-500/10 text-purple-400 group-hover:bg-purple-500/20">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
          </span>
          <div>
            <div class="text-xs font-semibold text-zinc-200 group-hover:text-purple-200">Explore response options</div>
            <div class="text-[11px] text-zinc-400">Compare show-cause defense routes and procedural barriers</div>
          </div>
        </button>

        <button
          onclick={() => onSelectAction("prepare_response")}
          class="flex items-start gap-3 p-3 rounded-lg border border-amber-500/40 bg-amber-500/10 hover:bg-amber-500/20 hover:border-amber-500/60 transition text-left group"
        >
          <span class="p-1.5 rounded bg-amber-500/20 text-amber-300 group-hover:bg-amber-500/30">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" /></svg>
          </span>
          <div>
            <div class="text-xs font-semibold text-amber-200 group-hover:text-amber-100">Prepare a response</div>
            <div class="text-[11px] text-zinc-300">Open drafting studio with verified docket citations & templates</div>
          </div>
        </button>
      </div>
    </div>
  </div>
{/if}
