<script lang="ts">
  import PartyEditor from "$lib/components/PartyEditor.svelte";
  import type { Party } from "$lib/types/case";

  interface Props {
    parties: Party[];
  }

  let { parties = $bindable() }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Parties</h2>
  <PartyEditor bind:parties />
</section>
