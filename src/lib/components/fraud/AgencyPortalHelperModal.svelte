<script lang="ts">
  import { ipc } from "$lib/ipc";
  import toast from "svelte-french-toast";

  interface PortalSubmissionBlock {
    agency: string;
    portal_url: string;
    field_suspect_entity: string;
    field_program_classification: string;
    field_narrative_summary: string;
    character_count: number;
    calling_card_included: boolean;
  }

  interface Props {
    isOpen?: boolean;
    caseId: string;
    onClose?: () => void;
  }

  let { isOpen = $bindable(false), caseId, onClose }: Props = $props();

  let selectedAgency = $state("HHS");
  let loading = $state(false);
  let portalBlock = $state<PortalSubmissionBlock | null>(null);
  let copiedField = $state("");

  const agencies = [
    { id: "HHS", label: "HHS-OIG (Medicaid / Medicare)", url: "https://oig.hhs.gov/fraud/report-fraud/" },
    { id: "DOJ", label: "DOJ Civil / Criminal Fraud", url: "https://www.justice.gov/fraud" },
    { id: "STATE", label: "State Medicaid Fraud Control Unit (MFCU)", url: "https://oig.hhs.gov/fraud/medicaid-fraud-control-units-mfcu/" },
    { id: "USDA", label: "USDA-OIG (SNAP / Child Nutrition)", url: "https://usdaoig.oversight.gov/hotline" },
    { id: "SBA", label: "SBA-OIG (PPP / EIDL / Grants)", url: "https://www.sba.gov/about-sba/oversight-advocacy/office-inspector-general/office-inspector-general-hotline" },
  ];

  $effect(() => {
    if (isOpen && caseId) {
      loadPortalBlock(selectedAgency);
    }
  });

  async function loadPortalBlock(agency: string) {
    loading = true;
    try {
      portalBlock = await ipc<PortalSubmissionBlock>("quitam_format_portal_cmd", {
        caseId,
        agency,
      });
    } catch (e) {
      toast.error(`Failed to format portal block: ${e}`);
    } finally {
      loading = false;
    }
  }

  function copyToClipboard(text: string, fieldName: string) {
    navigator.clipboard.writeText(text);
    copiedField = fieldName;
    toast.success(`Copied ${fieldName} to clipboard!`);
    setTimeout(() => {
      if (copiedField === fieldName) copiedField = "";
    }, 2000);
  }

  async function openPortalUrl(url: string) {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch {
      window.open(url, "_blank", "noopener,noreferrer");
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
      class="relative w-full max-w-3xl rounded-3xl border border-primary/30 bg-card p-6 md:p-8 shadow-2xl space-y-6 text-foreground animate-in fade-in zoom-in-95 duration-200 max-h-[90vh] overflow-y-auto"
    >
      <!-- Header -->
      <div class="flex items-start justify-between gap-4 border-b border-border/50 pb-4">
        <div class="flex items-center gap-3">
          <div class="p-2.5 rounded-2xl bg-amber-500/10 text-amber-400 border border-amber-500/20 text-2xl shrink-0">
            📋
          </div>
          <div>
            <h3 class="text-lg font-bold text-foreground">Guided Agency Portal Submission</h3>
            <p class="text-xs text-muted-foreground font-mono">
              Track B: Direct Administrative / OIG Web Form Submission (Character-Limited Auto-Fill)
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

      <!-- Agency Selector -->
      <div class="space-y-2">
        <label class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          Target Federal or State Oversight Agency
        </label>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
          {#each agencies as ag}
            <button
              onclick={() => {
                selectedAgency = ag.id;
                loadPortalBlock(ag.id);
              }}
              class="text-xs text-left p-2.5 rounded-xl border transition-all {selectedAgency === ag.id ? 'border-primary bg-primary/10 text-foreground font-semibold shadow-sm' : 'border-border/60 hover:bg-muted/40 text-muted-foreground'}"
            >
              {ag.label}
            </button>
          {/each}
        </div>
      </div>

      {#if loading}
        <div class="p-8 text-center text-sm text-muted-foreground">Formatting portal fields...</div>
      {:else if portalBlock}
        <!-- Notice Banner -->
        <div class="p-3.5 rounded-2xl bg-amber-500/10 border border-amber-500/30 text-amber-200 text-xs flex items-center justify-between gap-4">
          <div class="space-y-0.5">
            <span class="font-bold">Avoid Session Timeouts:</span> Government web portals time out after 15–20 minutes. Open the portal, then click each button below to paste verified fields in 30 seconds.
          </div>
          <button
            onclick={() => openPortalUrl(portalBlock?.portal_url || "")}
            class="px-3 py-1.5 bg-amber-500 text-zinc-950 font-bold rounded-lg hover:bg-amber-400 text-xs shrink-0 flex items-center gap-1.5 shadow"
          >
            <span>Launch Official .gov Portal</span>
            <span>↗</span>
          </button>
        </div>

        <!-- Field 1 -->
        <div class="space-y-1.5 p-3.5 rounded-2xl bg-muted/20 border border-border/50">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-foreground">Field 1: Suspect Individual / Organization</span>
            <button
              onclick={() => copyToClipboard(portalBlock?.field_suspect_entity || "", "Field 1")}
              class="text-xs px-2.5 py-1 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground font-mono flex items-center gap-1"
            >
              <span>{copiedField === "Field 1" ? "✓ Copied" : "Copy Field 1"}</span>
            </button>
          </div>
          <pre class="text-xs font-mono text-muted-foreground whitespace-pre-wrap bg-background/50 p-2.5 rounded-lg border border-border/40 max-h-24 overflow-y-auto">{portalBlock.field_suspect_entity}</pre>
        </div>

        <!-- Field 2 -->
        <div class="space-y-1.5 p-3.5 rounded-2xl bg-muted/20 border border-border/50">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold text-foreground">Field 2: Program Involved & EO-14395 Classification</span>
            <button
              onclick={() => copyToClipboard(portalBlock?.field_program_classification || "", "Field 2")}
              class="text-xs px-2.5 py-1 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground font-mono flex items-center gap-1"
            >
              <span>{copiedField === "Field 2" ? "✓ Copied" : "Copy Field 2"}</span>
            </button>
          </div>
          <pre class="text-xs font-mono text-muted-foreground whitespace-pre-wrap bg-background/50 p-2.5 rounded-lg border border-border/40">{portalBlock.field_program_classification}</pre>
        </div>

        <!-- Field 3 -->
        <div class="space-y-1.5 p-3.5 rounded-2xl bg-muted/20 border border-border/50">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-foreground">Field 3: Narrative Description of Suspected Fraud</span>
              <span class="text-[11px] px-2 py-0.5 rounded-full {portalBlock.character_count < 2000 ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-rose-500/10 text-rose-400 border border-rose-500/30'} font-mono">
                {portalBlock.character_count} chars (under 2,000 limit)
              </span>
            </div>
            <button
              onclick={() => copyToClipboard(portalBlock?.field_narrative_summary || "", "Field 3")}
              class="text-xs px-2.5 py-1 rounded bg-secondary hover:bg-secondary/80 text-secondary-foreground font-mono flex items-center gap-1"
            >
              <span>{copiedField === "Field 3" ? "✓ Copied" : "Copy Narrative"}</span>
            </button>
          </div>
          <pre class="text-xs font-mono text-muted-foreground whitespace-pre-wrap bg-background/50 p-2.5 rounded-lg border border-border/40 max-h-48 overflow-y-auto">{portalBlock.field_narrative_summary}</pre>
        </div>

        <!-- Attachment Guidance -->
        <div class="text-xs text-muted-foreground border-t border-border/40 pt-3 flex items-center justify-between">
          <span>Attachment: Upload <code class="text-foreground font-mono">exports/DOJ_Relator_Disclosure_Packet.pdf</code> to the portal if file upload is supported.</span>
          <button
            onclick={handleClose}
            class="px-4 py-2 bg-primary text-primary-foreground font-semibold rounded-xl text-xs hover:opacity-90 transition-opacity"
          >
            Done
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
