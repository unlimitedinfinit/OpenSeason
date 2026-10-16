<script lang="ts">
  import CaseRecordManager from "$lib/components/CaseRecordManager.svelte";
  import DiscrepancyAuditCard from "$lib/components/DiscrepancyAuditCard.svelte";
  import type { CaseProfile } from "$lib/types/case";
  import type { CourtRecordsRequestPacket, FactualPacketReport } from "$lib/types/record";

  interface Props {
    caseId: string;
    profile: CaseProfile | null;
    onExploreRoutes?: (issueId?: string) => void;
    onRequestFormReady?: (packet: CourtRecordsRequestPacket) => void;
    onPacketGenerated?: (packet: FactualPacketReport) => void;
  }

  let {
    caseId,
    profile,
    onExploreRoutes,
    onRequestFormReady,
    onPacketGenerated,
  }: Props = $props();
</script>

<div class="space-y-6">
  <CaseRecordManager
    {caseId}
    {profile}
    {onRequestFormReady}
  />

  <DiscrepancyAuditCard
    {caseId}
    {profile}
    {onExploreRoutes}
    {onPacketGenerated}
  />
</div>
