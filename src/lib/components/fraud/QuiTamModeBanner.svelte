<script lang="ts">
  import type { CaseProfile } from "$lib/types/case";

  interface Props {
    profile: CaseProfile;
    onOpenGhostWizard?: () => void;
    onOpenDisclosureExport?: () => void;
  }

  let { profile, onOpenGhostWizard, onOpenDisclosureExport }: Props = $props();

  const isQuiTam = $derived(
    profile.mode === "confidential" ||
    profile.document_kind === "qui_tam" ||
    profile.matter_tier === "fraud_investigation" ||
    Boolean(profile.sealed_at)
  );
</script>

{#if isQuiTam}
  <div class="rounded-3xl border border-amber-500/40 bg-gradient-to-r from-amber-500/10 via-card/90 to-card/60 p-5 shadow-lg backdrop-blur-xl space-y-3 animate-in fade-in duration-200">
    <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-2xl bg-amber-500/20 text-amber-400 border border-amber-500/30 text-xl shrink-0">
          🔒
        </div>
        <div>
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-bold text-foreground">EO 14395 Qui Tam / Fraud Investigation Vault</h3>
            <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-400 border border-amber-500/30 uppercase tracking-wider font-semibold">
              Airgapped Local Mode
            </span>
          </div>
          <p class="text-xs text-muted-foreground mt-0.5">
            Cloud sync & external telemetry are permanently disabled under 31 U.S.C. § 3730(b)(2).
          </p>
        </div>
      </div>

      <!-- Quick Actions for Fraud Mode -->
      <div class="flex items-center gap-2 shrink-0">
        {#if onOpenGhostWizard}
          <button
            type="button"
            onclick={onOpenGhostWizard}
            class="px-3 py-1.5 rounded-xl border border-amber-500/40 bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 text-xs font-semibold transition-colors flex items-center gap-1.5 shadow-sm"
          >
            <span>👻 Ghost Clinic Wizard</span>
          </button>
        {/if}

        {#if onOpenDisclosureExport}
          <button
            type="button"
            onclick={onOpenDisclosureExport}
            class="px-3 py-1.5 rounded-xl bg-primary hover:opacity-90 text-primary-foreground text-xs font-semibold transition-opacity flex items-center gap-1.5 shadow-sm"
          >
            <span>📄 DOJ 60-Day Disclosure Packet</span>
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
