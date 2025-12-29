<script lang="ts">
  import LegalAirlock from "$lib/components/LegalAirlock.svelte";
  import VaultLauncher from "$lib/components/VaultLauncher.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  // Application State
  type AppState = "AIRLOCK" | "AUTH" | "DASHBOARD" | "HUNT";
  let currentState = $state<AppState>("AIRLOCK");
  
  // Dashboard Data
  let hunts = $state<{id: string, name: string, created: string}[]>([]);
  let newHuntName = $state("");
  let createError = $state("");

  async function loadHunts() {
    try {
      hunts = await invoke("list_hunts");
    } catch (e) {
      console.error("Failed to list hunts", e);
    }
  }

  function handleAirlockPassed() {
    currentState = "AUTH";
  }

  function handleVaultUnlocked() {
    currentState = "DASHBOARD";
    loadHunts();
  }

  async function importHunt() {
    const path = prompt("Enter full path to .osb file to import:");
    if (!path) return;
    try {
      await invoke("import_hunt_cmd", { osbPath: path });
      await loadHunts();
      alert("Hunt Imported Successfully!");
    } catch (e) {
      alert("Import Failed: " + e);
    }
  }

  async function exportHunt(id: string) {
    const defaultPath = `C:\\Users\\${navigator.userAgent.includes("Windows") ? "Public" : "Shared"}\\${id}.osb`; // weak guess, just let them type
    const path = prompt(`Enter target path for export (e.g. C:/Users/You/Desktop/${id}.osb):`, "");
    if (!path) return;
    try {
      await invoke("export_hunt_cmd", { huntId: id, targetPath: path });
      alert("Hunt Exported Successfully!");
    } catch (e) {
      alert("Export Failed: " + e);
    }
  }
</script>

{#if currentState === "AIRLOCK"}
  <LegalAirlock onUnlock={handleAirlockPassed} />
{:else if currentState === "AUTH"}
  <div class="min-h-screen flex items-center justify-center bg-background">
    <VaultLauncher onLaunch={handleVaultUnlocked} />
  </div>
{:else if currentState === "DASHBOARD"}
  <main class="container mx-auto p-8 space-y-8">
    <div class="flex justify-between items-center border-b pb-4">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Open Season</h1>
        <p class="text-muted-foreground">Active Operations</p>
      </div>
      <div class="flex items-center gap-4">
         <button onclick={importHunt} class="text-sm border border-input bg-background hover:bg-accent hover:text-accent-foreground px-3 py-1 rounded">
            Import .osb
         </button>
         <div class="h-4 w-px bg-border"></div>
         <span class="text-xs text-green-500 font-mono">VAULT UNLOCKED</span>
         <button class="text-xs text-destructive hover:underline" onclick={() => location.reload()}>LOCK</button>
      </div>
    </div>

    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <!-- New Hunt Card -->
        <div class="p-6 bg-card border rounded-lg shadow-sm flex flex-col justify-center space-y-4 border-dashed border-primary/50">
            <h3 class="font-semibold text-center">New Operation</h3>
            <input 
                bind:value={newHuntName}
                placeholder="Target Name (e.g. Project X)" 
                class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                onkeydown={(e) => e.key === 'Enter' && createHunt()}
            />
            <button 
                onclick={createHunt}
                class="bg-primary text-primary-foreground px-4 py-2 rounded-md w-full"
            >
                Initialize Hunt
            </button>
            {#if createError}
                <p class="text-xs text-destructive text-center">{createError}</p>
            {/if}
        </div>

        <!-- Existing Hunts -->
        {#each hunts as hunt}
            <div class="p-6 bg-card border rounded-lg shadow-sm hover:border-primary/50 transition-colors cursor-pointer group relative">
                <div class="flex justify-between items-start mb-2">
                    <h3 class="font-bold text-lg group-hover:text-primary transition-colors">{hunt.name}</h3>
                    <span class="text-xs text-muted-foreground font-mono">ID: {hunt.id}</span>
                </div>
                <div class="mt-4 flex justify-between items-center">
                    <button 
                        class="text-xs text-muted-foreground hover:text-foreground underline z-10"
                        onclick={(e) => { e.stopPropagation(); exportHunt(hunt.id); }}
                    >
                        Export
                    </button>
                    <button class="text-sm bg-secondary text-secondary-foreground px-3 py-1 rounded">Open Case</button>
                </div>
            </div>
        {/each}
        
        {#if hunts.length === 0}
             <div class="p-6 flex items-center justify-center text-muted-foreground col-span-2">
                 No active hunts found.
             </div>
        {/if}
    </div>
  </main>
{/if}
