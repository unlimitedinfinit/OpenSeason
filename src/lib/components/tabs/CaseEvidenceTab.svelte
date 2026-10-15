<script lang="ts">
  import type { CaseProfile, CaseArtifact, ExhibitPacketReport } from "$lib/types/case";

  interface Props {
    profile: CaseProfile;
    artifacts: CaseArtifact[];
    assemblingPacket: boolean;
    batesPrefix: string;
    batesStart: number;
    exhibitPacketReport: ExhibitPacketReport | null;
    onaddevidence: () => void;
    ongeneratepacket: () => void;
  }

  let {
    profile,
    artifacts,
    assemblingPacket = $bindable(),
    batesPrefix = $bindable(),
    batesStart = $bindable(),
    exhibitPacketReport = $bindable(),
    onaddevidence,
    ongeneratepacket,
  }: Props = $props();
</script>

<div class="space-y-6">
  <section class="space-y-4 bg-card border rounded-lg p-6">
    <h2 class="font-semibold">Evidence</h2>
    <p class="text-sm text-muted-foreground">
      {#if profile.mode === "confidential"}
        Confidential exhibits are encrypted in the vault. A sealed case asks for the vault password again before a new file is stored.
      {:else}
        Standard exhibits are copied into this case folder. They are not encrypted.
      {/if}
    </p>
    <button onclick={onaddevidence} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Add evidence</button>
    {#if artifacts.filter((a) => a.kind === "evidence").length === 0}
      <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No exhibits yet. Add a file to start a record.</p>
    {:else}
      <ul class="text-sm space-y-2">
        {#each artifacts.filter((a) => a.kind === "evidence") as item}
          <li class="border rounded-md p-3">
            <p class="font-medium">{item.name}</p>
            <p class="text-xs font-mono text-muted-foreground">{item.path}</p>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between items-center">
      <div>
        <h2 class="font-semibold">Exhibit Packet Assembler</h2>
        <p class="text-xs text-muted-foreground">Generates a court-ready Table of Exhibits with SHA-256 hashes and a unified Bates-stamped PDF.</p>
      </div>
      <button
        onclick={ongeneratepacket}
        disabled={assemblingPacket || artifacts.filter((a) => a.kind === "evidence").length === 0}
        class="bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2 rounded-md text-sm font-medium disabled:opacity-50"
      >
        {assemblingPacket ? "Assembling..." : "Generate Bates Packet"}
      </button>
    </div>
    
    <div class="grid md:grid-cols-2 gap-3 pt-2">
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Bates Prefix (optional, e.g. ABRAMS- or ANDERSON-)</span>
        <input
          bind:value={batesPrefix}
          placeholder="e.g. EX-"
          class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
        />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Starting Page Number</span>
        <input
          type="number"
          bind:value={batesStart}
          min="1"
          class="w-full h-9 rounded-md border border-input bg-background px-3 text-sm"
        />
      </label>
    </div>

    {#if exhibitPacketReport}
      <div class="mt-4 p-4 border border-indigo-500/30 bg-indigo-500/10 rounded-md space-y-2">
        <p class="text-sm font-semibold text-indigo-300">Packet Assembly Complete ({exhibitPacketReport.total_pages} pages)</p>
        <div class="text-xs font-mono space-y-1">
          <p><span class="text-muted-foreground">Table of Exhibits:</span> {exhibitPacketReport.table_of_exhibits_pdf}</p>
          <p><span class="text-muted-foreground">Bates Stamped Packet:</span> {exhibitPacketReport.exhibit_packet_pdf}</p>
        </div>
        {#if exhibitPacketReport.exhibits.length}
          <div class="mt-3 overflow-x-auto">
            <table class="w-full text-xs text-left">
              <thead class="border-b border-border text-muted-foreground">
                <tr>
                  <th class="py-1">Ex #</th>
                  <th class="py-1">Bates Range</th>
                  <th class="py-1">Pages</th>
                  <th class="py-1">Description</th>
                </tr>
              </thead>
              <tbody>
                {#each exhibitPacketReport.exhibits as ex}
                  <tr class="border-b border-border/50">
                    <td class="py-1 font-mono">{ex.letter}</td>
                    <td class="py-1 font-mono text-indigo-400">{ex.bates_start} - {ex.bates_end}</td>
                    <td class="py-1">{ex.page_count}</td>
                    <td class="py-1">{ex.description}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {/if}
  </section>
</div>
