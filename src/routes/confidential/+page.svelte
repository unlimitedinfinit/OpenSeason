<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import LegalAirlock from "$lib/components/LegalAirlock.svelte";
  import VaultLauncher from "$lib/components/VaultLauncher.svelte";
  import { ipc } from "$lib/ipc";
  import { session } from "$lib/session.svelte";

  let stage = $state<"airlock" | "auth" | "done">("airlock");

  onMount(async () => {
    try {
      const locked = await ipc<boolean>("is_locked");
      if (!locked) {
        session.unlocked = true;
        session.airlockPassed = true;
        goto("/");
        return;
      }
    } catch {
      // stay on the airlock
    }
    if (session.airlockPassed) {
      stage = "auth";
    }
  });

  function passedAirlock() {
    session.airlockPassed = true;
    stage = "auth";
  }

  function unlocked() {
    session.unlocked = true;
    goto("/");
  }
</script>

<main class="container mx-auto max-w-3xl p-8 space-y-6">
  <button onclick={() => goto("/")} class="text-sm text-muted-foreground hover:text-foreground">Back to home</button>
  <div>
    <p class="text-xs uppercase tracking-widest text-red-300">Confidential mode</p>
    <h1 class="text-2xl font-bold">Unlock the vault</h1>
    <p class="text-sm text-muted-foreground mt-2">
      Existing hunts show up as Confidential cases on the home list after unlock. USAspending lookup and the disclosure statement are tools inside those cases, not a second app.
    </p>
  </div>
  {#if stage === "airlock"}
    <LegalAirlock onUnlock={passedAirlock} />
  {:else if stage === "auth"}
    <VaultLauncher onLaunch={unlocked} />
  {/if}
</main>
