<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { onComplete, onCancel, huntId = null, initialName = "" } = $props();

  let step = $state(1);
  let targetName = $state(initialName);
  let isVerifying = $state(false);
  let verificationResults = $state<any[]>([]);
  let error = $state("");
  
  async function handleVerify() {
    if (!targetName) return;
    isVerifying = true;
    error = "";
    try {
      verificationResults = await invoke("verify_target_cmd", { name: targetName });
      step = 2;
    } catch (e) {
      error = "Verification Failed: " + e;
      // Allow proceeding anyway? "Zero-Trust" might imply we don't trust the API either, but this is a tool.
      // Let's allow manual override if API fails.
      step = 2;
    } finally {
      isVerifying = false;
    }
  }

  async function handleCreate() {
    try {
      if (huntId) {
          // If we have an ID, we are just updating the target name of the existing draft
          await invoke("update_hunt", { huntId, name: targetName });
      } else {
          // Fallback if used without ID (shouldn't happen in new flow)
          console.error("HuntWizard used without ID in new flow");
      }
      onComplete();
    } catch (e) {
      error = "Operation Failed: " + e;
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm p-4">
  <div class="w-full max-w-3xl bg-card border rounded-lg shadow-xl overflow-hidden flex flex-col max-h-[90vh]">
    <div class="p-6 border-b">
      <h2 class="text-2xl font-semibold">New Hunt Configuration</h2>
      <p class="text-muted-foreground text-sm">Step {step}: {step === 1 ? "Identify Target" : "Confirm Intelligence"}</p>
    </div>

    <div class="p-6 flex-1 overflow-y-auto space-y-6">
      {#if step === 1}
        <div class="space-y-4">
          <label for="target" class="block text-sm font-medium">Target Entity Name</label>
          <input 
             id="target" 
             bind:value={targetName}
             class="flex h-12 w-full rounded-md border border-input bg-background px-3 py-2 text-lg"
             placeholder="e.g. Acme Defense Corp"
             onkeydown={(e) => e.key === 'Enter' && handleVerify()}
          />
          <p class="text-sm text-muted-foreground">
            We will cross-reference this against public spending data for contracts >$100k.
          </p>
          {#if error}
            <p class="text-destructive text-sm">{error}</p>
          {/if}
        </div>

      {:else if step === 2}
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-medium">
               {#if verificationResults.length > 0}
                  <span class="text-green-600 flex items-center gap-2">
                     <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                     High-value matches found!
                  </span>
               {:else}
                  <span class="text-yellow-600 flex items-center gap-2">
                     <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                     No direct matches
                  </span>
               {/if}
            </h3>
            <span class="text-xs bg-muted px-2 py-1 rounded">USASpending.gov</span>
          </div>
          
          {#if verificationResults.length > 0}
             <div class="border rounded-md divide-y max-h-60 overflow-y-auto">
                 {#each verificationResults as res}
                   <div class="p-3 text-sm hover:bg-muted/50">
                     <div class="flex justify-between">
                       <span class="font-bold">{res.recipient_name}</span>
                       <span class="font-mono text-green-600">${res.total_obligation.toLocaleString()}</span>
                     </div>
                     <p class="text-muted-foreground text-xs truncate">{res.description || "No description"}</p>
                     <div class="text-xs text-muted-foreground mt-1">{res.awarding_agency} • {res.date_signed}</div>
                   </div>
                 {/each}
             </div>
             
             <button
               onclick={handleCreate}
               class="w-full bg-green-600 hover:bg-green-700 text-white py-3 rounded-md font-bold flex items-center justify-center gap-2"
             >
                Proceed to Wizard &rarr;
             </button>

          {:else}
             <!-- No matches advisory -->
             <div class="bg-yellow-500/10 border border-yellow-500/50 p-4 rounded-md text-sm text-yellow-600 dark:text-yellow-500 space-y-2">
                <p class="font-bold">Advisory: No direct high-value matches found in USASpending.gov.</p>
                <p>This does NOT mean the claim is invalid — many frauds involve:</p>
                <ul class="list-disc pl-5 space-y-1 opacity-90">
                   <li>Subcontractors or subsidiaries</li>
                   <li>Grants, loans, or non-contract payments</li>
                   <li>False certifications without a specific award</li>
                   <li>Insider knowledge not yet public</li>
                </ul>
                <p class="pt-2 font-medium">Proceed manually if you have strong evidence of fraud.</p>
             </div>

             <div class="flex gap-3 pt-2">
                <button
                  onclick={handleCreate}
                  class="flex-1 bg-green-600 hover:bg-green-700 text-white py-2 rounded-md font-medium"
                >
                   Continue Manually
                </button>
                <button
                  onclick={() => step = 1}
                  class="flex-1 bg-muted hover:bg-muted/80 text-foreground py-2 rounded-md font-medium"
                >
                   Try Different Name
                </button>
             </div>
          {/if}
        </div>
      {/if}
    </div>

    {#if step === 1}
      <div class="p-6 border-t bg-muted/20 flex justify-between items-center">
        <button onclick={onCancel} class="text-sm hover:underline">Cancel</button>
        
        <button
          onclick={handleVerify}
          disabled={isVerifying || !targetName}
          class="bg-primary text-primary-foreground px-6 py-2 rounded-md font-medium disabled:opacity-50 flex items-center gap-2"
        >
          {#if isVerifying}
            <span>Scouting...</span>
          {:else}
             Verify Target
          {/if}
        </button>
      </div>
    {/if}
  </div>
</div>
