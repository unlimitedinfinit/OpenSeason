<script lang="ts">
  import CloudSyncModal from "$lib/components/CloudSyncModal.svelte";
  import PdfToolsModal from "$lib/components/PdfToolsModal.svelte";
  import SignatureStudioModal from "$lib/components/SignatureStudioModal.svelte";
  import IntakeWizardModal from "$lib/components/IntakeWizardModal.svelte";
  import AccountabilityModal from "$lib/components/AccountabilityModal.svelte";
  import AdaAccommodationModal from "$lib/components/AdaAccommodationModal.svelte";
  import CourtRecordRetrievalModal from "$lib/components/CourtRecordRetrievalModal.svelte";
  import AgentWorkflowModal from "$lib/components/AgentWorkflowModal.svelte";
  import ConsultationDossierModal from "$lib/components/ConsultationDossierModal.svelte";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import type { CaseProfile } from "$lib/types/case";

  interface Props {
    caseId: string;
    profile: CaseProfile | null;
    showCloudModal: boolean;
    showPdfToolsModal: boolean;
    showSignatureModal: boolean;
    showIntakeWizard: boolean;
    showAccountabilityModal: boolean;
    showAdaModal: boolean;
    showRecordRetrievalModal: boolean;
    showWorkflowModal: boolean;
    showConsultationModal: boolean;
    confirmSeal: boolean;
    preselectedIssueId: string | null;
    onReload: () => Promise<void>;
    onConfirmSeal: () => Promise<void>;
    onCloseCloud: () => void;
    onClosePdf: () => void;
    onCloseSignature: () => void;
    onCloseIntake: () => void;
    onCloseAccountability: () => void;
    onCloseAda: () => void;
    onCloseRecordRetrieval: () => void;
    onCloseWorkflow: () => void;
    onCloseConsultation: () => void;
    onCancelSeal: () => void;
  }

  let {
    caseId,
    profile = $bindable(),
    showCloudModal = $bindable(),
    showPdfToolsModal = $bindable(),
    showSignatureModal = $bindable(),
    showIntakeWizard = $bindable(),
    showAccountabilityModal = $bindable(),
    showAdaModal = $bindable(),
    showRecordRetrievalModal = $bindable(),
    showWorkflowModal = $bindable(),
    showConsultationModal = $bindable(),
    confirmSeal = $bindable(),
    preselectedIssueId,
    onReload,
    onConfirmSeal,
    onCloseCloud,
    onClosePdf,
    onCloseSignature,
    onCloseIntake,
    onCloseAccountability,
    onCloseAda,
    onCloseRecordRetrieval,
    onCloseWorkflow,
    onCloseConsultation,
    onCancelSeal,
  }: Props = $props();
</script>

<CloudSyncModal
  bind:open={showCloudModal}
  activeCaseId={caseId}
  activeCaseTitle={profile?.title}
  isConfidential={profile?.mode === "confidential"}
  onsynced={onReload}
/>

<PdfToolsModal
  bind:open={showPdfToolsModal}
  {caseId}
  onclose={onClosePdf}
/>

<SignatureStudioModal
  bind:open={showSignatureModal}
  {caseId}
  onclose={onCloseSignature}
/>

{#if profile}
  <IntakeWizardModal
    bind:open={showIntakeWizard}
    {caseId}
    bind:profile
    onUpdated={onReload}
  />
{/if}

<AccountabilityModal
  bind:open={showAccountabilityModal}
  {caseId}
  {profile}
  {preselectedIssueId}
  onclose={onCloseAccountability}
/>

<AdaAccommodationModal
  bind:open={showAdaModal}
  {caseId}
  {profile}
  onclose={onCloseAda}
/>

<CourtRecordRetrievalModal
  bind:open={showRecordRetrievalModal}
  {caseId}
  onclose={onCloseRecordRetrieval}
  onGenerated={onReload}
/>

{#if showWorkflowModal}
  <AgentWorkflowModal
    {caseId}
    onClose={onCloseWorkflow}
  />
{/if}

{#if showConsultationModal}
  <ConsultationDossierModal
    {caseId}
    {profile}
    onClose={onCloseConsultation}
  />
{/if}

<ConfirmDialog
  bind:open={confirmSeal}
  title="Seal this case?"
  body="Convert this case to a sealed Confidential vault? This cannot be reversed by editing case.json. Sync will be refused."
  confirmLabel="Seal"
  danger={true}
  onconfirm={onConfirmSeal}
  oncancel={onCancelSeal}
/>
