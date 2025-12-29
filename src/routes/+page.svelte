<script lang="ts">
  import LegalAirlock from "$lib/components/LegalAirlock.svelte";
  import VaultLauncher from "$lib/components/VaultLauncher.svelte";
  import HuntWizard from "$lib/components/HuntWizard.svelte";
  import HelpModal from "$lib/components/HelpModal.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  // Application State
  type AppState = "AIRLOCK" | "AUTH" | "DASHBOARD" | "HUNT";
  let currentState = $state<AppState>("AIRLOCK");
  
  import { goto } from '$app/navigation'; // Requires SvelteKit router
  
  // Dashboard Data
  let hunts = $state<{id: string, name: string, created: string}[]>([]);
  let isWizardOpen = $state(false);
  let isHelpOpen = $state(false);
  let isCreating = $state(false);

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

  function toggleWizard() {
    isWizardOpen = !isWizardOpen;
  }
  
  function handleWizardComplete() {
    isWizardOpen = false;
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

  async function generateReport(hunt: any) {
    if (!confirm(`Generate Disclosure Statement for ${hunt.name}?`)) return;
    try {
      // Mock data for MVP since we don't have full evidence DB yet
      const path = await invoke("save_disclosure_cmd", { 
        huntId: hunt.id, 
        target: hunt.name, 
        count: 12, 
        value: 1540000.0 
      });
      alert("Report Saved to Vault:\n" + path);
    } catch (e) {
      alert("Report Generation Failed: " + e);
    }
  }

  async function startNewOperation() {
    if (isCreating) return;
    isCreating = true;
    // alert("DEBUG: Clicked New Operation"); // Debug trigger

    try {
      // alert("DEBUG: Invoking create_new_hunt...");
      const newHunt: any = await invoke('create_new_hunt', {
        name: `Operation ${new Date().toLocaleDateString()}`
      });

      // alert("DEBUG: Created: " + JSON.stringify(newHunt));
      
      // Update local list
      hunts = [...hunts, newHunt];
      
      // Navigate to hunt page
      // alert("DEBUG: Navigating to " + `/hunt/${newHunt.id}`);
      goto(`/hunt/${newHunt.id}`);
    } catch (err) {
      console.error(err);
      alert(`Failed to create operation: ${err}`);
      if (typeof err === 'string' && err.includes("Vault Locked")) {
        alert("Session timed out - please lock and unlock the vault.");
      }
    } finally {
      isCreating = false;
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
         <button onclick={() => isHelpOpen = true} class="text-sm border border-input bg-background hover:bg-accent hover:text-accent-foreground px-3 py-1 rounded flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><path d="M12 17h.01"/></svg>
            Help / Instructions
         </button>
         <button onclick={importHunt} class="text-sm border border-input bg-background hover:bg-accent hover:text-accent-foreground px-3 py-1 rounded">
            Import .osb
         </button>
         <div class="h-4 w-px bg-border"></div>
         <span class="text-xs text-green-500 font-mono">VAULT UNLOCKED</span>
         <button class="text-xs text-destructive hover:underline" onclick={() => location.reload()}>LOCK</button>
      </div>
    </div>

    <div class="space-y-6">
       <!-- Additional Legal Resources Card -->
       <div class="bg-card/50 border border-indigo-500/20 rounded-lg p-6 shadow-sm backdrop-blur-sm relative overflow-hidden">
          <div class="relative z-10 flex flex-col sm:flex-row gap-6 items-start">
             <div class="flex-1 space-y-2">
                <h3 class="text-lg font-semibold flex items-center gap-2">
                   <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-indigo-400"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
                   Additional Legal Resources
                </h3>
                <p class="text-sm text-muted-foreground leading-relaxed">
                   Open Season helps you organize evidence and prepare your thinking.<br>
                   For professional document review, case tracking, printable summaries, consultations, or paid services by third-party providers, visit:
                </p>
                <div class="pt-2">
                   <strong class="block text-foreground">JustLegal.me</strong>
                   <a href="https://justlegal.me" target="_blank" rel="noopener noreferrer" class="text-xs text-indigo-400 hover:underline">https://justlegal.me</a>
                </div>
                <ul class="text-xs text-muted-foreground list-disc pl-4 pt-1 space-y-0.5">
                   <li>Track case progress & generate professional summaries</li>
                   <li>Find lawyers for review, unbundled help, or representation</li>
                </ul>
             </div>
             <div class="flex flex-col items-end gap-2 shrink-0">
                <button 
                   onclick={() => window.open('https://justlegal.me', '_blank')}
                   class="bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 text-white font-medium px-4 py-2 rounded-md shadow-md transition-all text-sm flex items-center gap-2"
                >
                   Visit JustLegal.me &rarr;
                </button>
                <small class="text-[10px] text-muted-foreground/70 max-w-[200px] text-right leading-tight">
                   Note: External service. Open Season provides organizational tools only — not legal advice or representation.
                </small>
             </div>
          </div>
       </div>

       <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <!-- New Hunt Card -->
        <button 
          onclick={startNewOperation}
          class="p-6 bg-card border rounded-lg shadow-sm flex flex-col items-center justify-center space-y-4 border-dashed border-primary/50 hover:bg-accent/10 transition-colors relative"
          disabled={isCreating}
        >
            {#if isCreating}
               <div class="h-12 w-12 flex items-center justify-center">
                 <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
               </div>
               <div class="text-center">
                  <h3 class="font-semibold">Initializing...</h3>
                  <p class="text-xs text-muted-foreground mt-1">Creating secure vault</p>
               </div>
            {:else}
              <div class="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
                <span class="text-2xl text-primary font-light">+</span>
              </div>
              <div class="text-center">
                <h3 class="font-semibold">New Operation</h3>
                <p class="text-xs text-muted-foreground mt-1">Start verification process</p>
              </div>
            {/if}
        </button>

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
                    <div class="flex gap-2">
                         <button 
                            class="text-sm border border-red-200 text-red-700 bg-red-50 hover:bg-red-100 px-3 py-1 rounded"
                            onclick={(e) => { e.stopPropagation(); generateReport(hunt); }}
                         >
                            Report
                         </button>
                         <button class="text-sm bg-secondary text-secondary-foreground px-3 py-1 rounded">Open Case</button>
                    </div>
                </div>
            </div>
        {/each}
        
        {#if hunts.length === 0}
             <div class="p-6 flex items-center justify-center text-muted-foreground col-span-2">
                 No active hunts found.
             </div>
        {/if}
    </div>
    </div>
    
    <HelpModal bind:isOpen={isHelpOpen} />
    {#if isWizardOpen}
       <HuntWizard onComplete={handleWizardComplete} onCancel={() => isWizardOpen = false} />
    {/if}
  </main>
{/if}
