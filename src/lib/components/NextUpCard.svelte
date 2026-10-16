<script lang="ts">
  import { goto } from "$app/navigation";
  import type { PortfolioCaseItem } from "$lib/types/case";

  let {
    matter = null,
    onEnterDate = () => {},
  }: {
    matter: PortfolioCaseItem | null;
    onEnterDate?: () => void;
  } = $props();

  const ENTER = "Enter the order date to compute this deadline";

  function tone(days: number | null | undefined, status?: string | null): "none" | "amber" | "red" {
    if (status === "needs_trigger_date" || days == null) return "none";
    if (days <= 7) return "red";
    return "amber";
  }

  const t = $derived(tone(matter?.days_until_due, matter?.urgent_status));
</script>

<section class="rounded-lg border border-border bg-card p-5 space-y-3" aria-labelledby="next-up-heading">
  <p class="text-xs uppercase tracking-widest text-muted-foreground">Next up</p>
  {#if !matter}
    <h2 id="next-up-heading" class="text-xl font-semibold">Start a case</h2>
    <p class="text-sm text-muted-foreground">OpenSeason formats papers you already have. It never files or serves.</p>
    <button type="button" class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm" onclick={() => goto("/case/new?kind=reconsideration")}>
      Start a motion
    </button>
  {:else if matter.urgent_status === "needs_trigger_date" || !matter.urgent_deadline}
    <h2 id="next-up-heading" class="text-xl font-semibold">{matter.title}</h2>
    <p class="text-sm">{ENTER}</p>
    <p class="text-xs text-muted-foreground">
      {matter.urgent_branch_title || "Deadline"} · {matter.urgent_rule || "rule not computed yet"}
    </p>
    <button type="button" class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm" onclick={() => { onEnterDate(); goto(`/case/${matter.case_id}`); }}>
      Open this case and enter the order date
    </button>
  {:else}
    <h2 id="next-up-heading" class="text-xl font-semibold">{matter.urgent_branch_title || "Upcoming deadline"}</h2>
    <p class="text-lg font-medium {t === 'red' ? 'text-red-400' : 'text-amber-300'}">
      Due {matter.urgent_deadline}
      {#if matter.days_until_due != null}
        · {matter.days_until_due >= 0 ? `${matter.days_until_due} days left` : `${Math.abs(matter.days_until_due)} days past`}
      {/if}
    </p>
    <p class="text-sm">{matter.title}</p>
    {#if matter.urgent_math}
      <p class="text-xs text-muted-foreground font-mono">{matter.urgent_math}</p>
    {:else}
      <p class="text-xs text-muted-foreground">
        Trigger {matter.urgent_trigger_date || "(order date)"} · {matter.urgent_rule || "check the rule"}
      </p>
    {/if}
    {#if matter.unverified}
      <p class="text-xs text-amber-200">unverified, check the order and rule</p>
    {/if}
    <button type="button" class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm" onclick={() => goto(`/case/${matter.case_id}`)}>
      Start this document
    </button>
  {/if}
</section>
