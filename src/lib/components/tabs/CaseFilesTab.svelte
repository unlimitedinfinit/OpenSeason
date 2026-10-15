<script lang="ts">
  import type { CaseArtifact } from "$lib/types/case";

  interface Props {
    artifacts: CaseArtifact[];
  }

  let { artifacts }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Drafts and exports</h2>
  {#if artifacts.length === 0}
    <p class="text-sm text-muted-foreground border border-dashed rounded-md p-4">Nothing exported yet. Use the Documents or Motions tab to create Word and PDF files.</p>
  {:else}
    <ul class="text-sm space-y-2">
      {#each artifacts as item}
        <li class="border rounded-md p-3">
          <p class="text-[10px] uppercase text-muted-foreground">{item.kind}</p>
          <p class="font-medium">{item.name}</p>
          <p class="text-xs font-mono text-muted-foreground">{item.path}</p>
        </li>
      {/each}
    </ul>
  {/if}
</section>
