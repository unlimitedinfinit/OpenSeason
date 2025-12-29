<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { onLaunch } = $props();
  
  let password = $state("");
  let status = $state("Checking Vault...");
  let isChecking = $state(true);
  let error = $state("");
  let salt = $state("");

  onMount(async () => {
    try {
      salt = await invoke("get_salt");
      status = "Vault Locked";
    } catch (e) {
      error = "Failed to access Vault Storage: " + e;
    } finally {
      isChecking = false;
    }
  });

  async function handleUnlock() {
    if (!password) {
      error = "Password required";
      return;
    }
    
    status = "Deriving Key...";
    error = "";
    
    try {
      const success = await invoke("unlock_vault", { password, salt });
      if (success) {
        onLaunch();
      } else {
        error = "Decryption Failed (Invalid Password?)";
      }
    } catch (e) {
      error = "Unlock Error: " + e;
      status = "Error";
    }
  }
</script>

<div class="max-w-md mx-auto mt-20 p-8 bg-card border rounded-lg shadow-lg space-y-6">
  <div class="text-center">
    <h2 class="text-2xl font-bold">Hunter's Vault</h2>
    <p class="text-muted-foreground text-sm mt-1">Session-Only Encryption</p>
  </div>

  {#if isChecking}
    <div class="flex justify-center p-4">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>
  {:else}
    <div class="space-y-4">
      <div class="space-y-2">
        <label for="master-pass" class="text-sm font-medium">Master Password</label>
        <input 
          id="master-pass"
          type="password" 
          bind:value={password}
          class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          placeholder="Enter passphrase..."
          onkeydown={(e) => e.key === 'Enter' && handleUnlock()}
        />
      </div>

      {#if error}
        <div class="p-3 text-sm text-destructive bg-destructive/10 rounded-md border border-destructive/20">
          {error}
        </div>
      {/if}

      <button
        onclick={handleUnlock}
        class="w-full h-10 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md font-medium transition-colors"
      >
        Unlock Vault
      </button>

      <div class="mt-4 p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md text-xs text-yellow-600 dark:text-yellow-500">
        <p class="font-bold flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/></svg>
          WARNING: NO RECOVERY
        </p>
        <p class="mt-1 opacity-90">
          If you lose this password, your data is lost forever. 
          Open Season uses zero-knowledge encryption. We cannot reset it for you.
        </p>
      </div>

      <p class="text-xs text-center text-muted-foreground mt-2">
        Key is held in RAM only. Zeroized on exit.
      </p>
    </div>
  {/if}
</div>
