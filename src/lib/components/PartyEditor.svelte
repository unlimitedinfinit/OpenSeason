<script lang="ts">
  import { PARTY_ROLES, type Party } from "$lib/types/case";

  let { parties = $bindable([] as Party[]) }: { parties: Party[] } = $props();

  function addParty() {
    parties = [
      ...parties,
      { name: "", role: "Plaintiff", address: "", email: "", phone: "", counsel: "Pro se" },
    ];
  }

  function removeParty(index: number) {
    parties = parties.filter((_, i) => i !== index);
  }
</script>

<div class="space-y-4">
  {#if parties.length === 0}
    <p class="text-sm text-muted-foreground border border-dashed border-border rounded-md p-4">
      No parties yet. Add each person or entity with a role (plaintiff, defendant, appellant, appellee), contact details, and whether they have an attorney or are pro se.
    </p>
  {/if}

  {#each parties as party, index}
    <div class="rounded-lg border border-border bg-card p-4 space-y-3">
      <div class="flex justify-between items-center">
        <p class="text-xs uppercase tracking-wide text-muted-foreground">Party {index + 1}</p>
        <button type="button" onclick={() => removeParty(index)} class="text-xs text-red-400 hover:text-red-300">
          Remove
        </button>
      </div>
      <div class="grid md:grid-cols-2 gap-3">
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Name</span>
          <input bind:value={party.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Full name or entity" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Role</span>
          <select bind:value={party.role} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm">
            {#each PARTY_ROLES as role}
              <option value={role}>{role}</option>
            {/each}
          </select>
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Address</span>
          <input bind:value={party.address} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Counsel</span>
          <input bind:value={party.counsel} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Pro se or attorney name" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Email</span>
          <input bind:value={party.email} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
        <label class="block space-y-1">
          <span class="text-xs text-muted-foreground">Phone</span>
          <input bind:value={party.phone} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
        </label>
      </div>
    </div>
  {/each}

  <button type="button" onclick={addParty} class="border border-border px-3 py-2 rounded-md text-sm">
    Add party
  </button>
</div>
