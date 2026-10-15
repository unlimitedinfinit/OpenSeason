<script lang="ts">
  import FieldHelp from "$lib/components/FieldHelp.svelte";

  interface Props {
    deadlinePack: string;
    deadlineRule: string;
    triggerDate: string;
    gapTitle: string;
    gapHard: string;
    exhibits: { letter: string; filename: string; description: string }[];
    checkReport: {
      ok?: boolean;
      never_files?: boolean;
      issues?: { field: string; message: string }[];
      fact?: { rows: { id: number; assertion: string; record_cite: string; exhibit?: string; verified: boolean }[] };
      authority?: { rows: { id: number; cite: string; status: string }[] };
      deadline?: { due: string; math: string; rule: string };
      signoff?: { items: { id: string; label: string; checked: boolean }[]; ok?: boolean };
    } | null;
    onrunchecks: () => void;
    onexportics: () => void;
    onexportpacket: () => void;
    onlinkfact: (rowId: number, letter: string) => void;
    ontogglesignoff: (itemId: string, checked: boolean) => void;
    onaddgap: () => void;
  }

  let {
    deadlinePack = $bindable(),
    deadlineRule = $bindable(),
    triggerDate = $bindable(),
    gapTitle = $bindable(),
    gapHard = $bindable(),
    exhibits,
    checkReport,
    onrunchecks,
    onexportics,
    onexportpacket,
    onlinkfact,
    ontogglesignoff,
    onaddgap,
  }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Check and fix</h2>
  <p class="text-sm text-muted-foreground">Issues sit next to the draft. The app never invents a cite and never files or serves. <FieldHelp text="Run checks after you write. Link each fact to Ex. A, Ex. B. You check every sign-off box." /></p>
  <div class="grid md:grid-cols-3 gap-3">
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Rule pack</span>
      <select bind:value={deadlinePack} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm">
        <option value="nj-app-div">New Jersey Appellate Division</option>
        <option value="co-mandamus">Colorado mandamus (C.A.R. 21)</option>
        <option value="generic-us-appellate">Generic US appellate</option>
      </select>
    </label>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Deadline rule</span>
      <select bind:value={deadlineRule} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm">
        <option value="reconsideration-appellate">R. 2:11-6 (10 days from entry)</option>
        <option value="reconsideration-trial">R. 4:49-2 (20 days from service)</option>
        <option value="motion-appellate">R. 2:8-1 (appellate motion)</option>
        <option value="toll-time-to-appeal">R. 2:4-3(e) (tolling note)</option>
        <option value="car-21-petition">C.A.R. 21 (no automatic day count)</option>
      </select>
    </label>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Trigger date (YYYY-MM-DD) <FieldHelp text="The date the order was entered or served. Use YYYY-MM-DD. Example: 2026-01-09." /></span>
      <input bind:value={triggerDate} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="2026-01-09" />
    </label>
  </div>
  <div class="flex flex-wrap gap-2">
    <button onclick={onrunchecks} class="bg-secondary text-secondary-foreground px-4 py-2 rounded-md text-sm">Run checks</button>
    <button onclick={onexportics} class="border border-border px-4 py-2 rounded-md text-sm">Save deadline on calendar (.ics)</button>
    <button onclick={onexportpacket} class="border border-border px-4 py-2 rounded-md text-sm">Export filing packet</button>
  </div>
  {#if exhibits.length}
    <p class="text-xs text-muted-foreground">Exhibits: {exhibits.map((e) => `Ex. ${e.letter} ${e.filename}`).join(" · ")}</p>
  {/if}
  {#if checkReport}
    <p class="text-sm">{checkReport.never_files ? "Never files. " : ""}{checkReport.ok ? "Gates passed." : "Issues remain. Fix them below. Do not invent a cite."}</p>
    {#if checkReport.deadline}
      <p class="text-xs font-mono">{checkReport.deadline.rule}: {checkReport.deadline.math}</p>
    {/if}
    {#if checkReport.fact?.rows?.length}
      <ul class="text-sm space-y-3">
        {#each checkReport.fact.rows as row}
          <li class="border rounded-md p-3 space-y-2">
            <p>{row.id}. {row.assertion}</p>
            <p class="text-xs text-muted-foreground">{row.record_cite || "No record cite yet."}</p>
            <div class="flex flex-wrap gap-2">
              {#each exhibits as ex}
                <button onclick={() => onlinkfact(row.id, ex.letter)} class="text-xs border border-border px-2 py-1 rounded-md">Link Ex. {ex.letter}</button>
              {/each}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
    {#if checkReport.authority?.rows?.length}
      <ul class="text-sm space-y-1">
        {#each checkReport.authority.rows as row}
          <li>{row.id}. {row.status} {row.cite}</li>
        {/each}
      </ul>
    {/if}
    {#if checkReport.signoff?.items?.length}
      <ul class="text-sm space-y-1">
        {#each checkReport.signoff.items as item}
          <li>
            <label class="flex gap-2 items-start">
              <input type="checkbox" checked={item.checked} onchange={(e) => ontogglesignoff(item.id, (e.currentTarget as HTMLInputElement).checked)} />
              <span>{item.label}</span>
            </label>
          </li>
        {/each}
      </ul>
    {/if}
    {#if checkReport.issues?.length}
      <ul class="text-sm text-destructive list-disc pl-5">
        {#each checkReport.issues as issue}
          <li>{issue.field}: {issue.message}</li>
        {/each}
      </ul>
    {/if}
  {:else}
    <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No check run yet. Write the guided sections or import Markdown, add an exhibit, then run checks.</p>
  {/if}
  <div class="border-t pt-4 space-y-2">
    <h3 class="text-sm font-semibold">Justice-gap note</h3>
    <p class="text-xs text-muted-foreground">What was hard? Fake names only. This is for later content, not the court paper.</p>
    <input bind:value={gapTitle} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Short title" />
    <textarea bind:value={gapHard} class="w-full h-16 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="What was hard"></textarea>
    <button onclick={onaddgap} class="border border-border px-3 py-1.5 rounded-md text-sm">Save gap note</button>
  </div>
</section>
