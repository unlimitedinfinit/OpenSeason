<script lang="ts">
  import FieldHelp from "$lib/components/FieldHelp.svelte";
  import type { CaseProfile } from "$lib/types/case";

  interface Props {
    profile: CaseProfile;
    addressText: string;
  }

  let { profile = $bindable(), addressText = $bindable() }: Props = $props();
</script>

<section class="space-y-4 bg-card border rounded-lg p-6">
  <h2 class="font-semibold">Case profile</h2>
  <label class="block space-y-1">
    <span class="text-xs text-muted-foreground">Title</span>
    <input bind:value={profile.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
  </label>
  <label class="block space-y-1">
    <span class="text-xs text-muted-foreground">Court name <FieldHelp text="Copy the court name from a paper you already have. Do not guess a court." /></span>
    <input bind:value={profile.court.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
  </label>
  <label class="block space-y-1">
    <span class="text-xs text-muted-foreground">Docket number <FieldHelp text="Copy the case number exactly. If you do not have it, leave this blank." /></span>
    <input bind:value={profile.docket_number} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
  </label>
  <div class="grid md:grid-cols-2 gap-3">
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Filer name</span>
      <input bind:value={profile.filer.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Signature name</span>
      <input bind:value={profile.filer.signature_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
  </div>
  <label class="block space-y-1">
    <span class="text-xs text-muted-foreground">Filer role</span>
    <input bind:value={profile.filer.role} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
  </label>
  <label class="block space-y-1">
    <span class="text-xs text-muted-foreground">Address (one line per row)</span>
    <textarea bind:value={addressText} class="w-full h-20 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
  </label>
  <div class="grid md:grid-cols-2 gap-3">
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Phone</span>
      <input bind:value={profile.filer.phone} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Email</span>
      <input bind:value={profile.filer.email} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
  </div>
</section>
