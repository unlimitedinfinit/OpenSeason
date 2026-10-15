<script lang="ts">
  import type { CaseProfile } from "$lib/types/case";

  interface Props {
    profile: CaseProfile;
    eventTitle: string;
    eventDate: string;
    eventNotes: string;
    onaddevent: () => void;
    onremoveevent: (id: string) => void;
  }

  let {
    profile = $bindable(),
    eventTitle = $bindable(),
    eventDate = $bindable(),
    eventNotes = $bindable(),
    onaddevent,
    onremoveevent,
  }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Timeline</h2>
  <div class="grid md:grid-cols-3 gap-3">
    <input bind:value={eventTitle} class="h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Event title" />
    <input type="date" bind:value={eventDate} class="h-10 rounded-md border border-input bg-background px-3 text-sm" />
    <input bind:value={eventNotes} class="h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Notes" />
  </div>
  <button onclick={onaddevent} class="border border-border px-4 py-2 rounded-md text-sm">Add event</button>
  {#if !profile.timeline || profile.timeline.length === 0}
    <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">No events yet. Dates you add here stay with the case profile.</p>
  {:else}
    <ul class="text-sm space-y-2">
      {#each profile.timeline as event}
        <li class="border rounded-md p-3 flex justify-between gap-3">
          <div>
            <p class="font-medium">{event.date} · {event.title}</p>
            <p class="text-muted-foreground">{event.notes}</p>
          </div>
          <button onclick={() => onremoveevent(event.id)} class="text-xs text-red-400">Remove</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
