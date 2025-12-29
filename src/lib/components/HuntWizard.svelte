<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { onComplete, onCancel, huntId = null, initialName = "", demoMode = false } = $props();

  let step = $state(1);
  let targetName = $state(demoMode ? "Feeding Our Future" : initialName);
  let isVerifying = $state(false);
  let verificationResults = $state<any[]>([]);
  let error = $state("");
  let mounted = false;
  
  // Sync logic for Edit Mode:
  // If initialName exists + not demo + step 1 + target is empty, sync it.
  $effect(() => {
     if (initialName && !demoMode && step === 1 && !targetName) {
         targetName = initialName;
     }
  });

  // Auto-run verify REMOVED per user request to restore proper flow.
  // The wizard should always start at Step 1 for new operations.
  onMount(() => {
    mounted = true;
  });

  async function handleVerify() {
    if (!targetName) return;
    isVerifying = true;
    error = "";
    try {
      if (demoMode && targetName === "Feeding Our Future") {
          // Force wait for effect
          await new Promise(r => setTimeout(r, 1000));
      }
      console.log("Verifying target:", targetName);
      verificationResults = await invoke("verify_target_cmd", { name: targetName });
      console.log("Results:", verificationResults);
      step = 2;
    } catch (e) {
      console.error("Link Error:", e);
      error = "Verification Failed: " + e;
      step = 2;
    } finally {
      isVerifying = false;
    }
  }

  async function handleCreate() {
    try {
      if (huntId) {
          await invoke("update_hunt", { huntId, name: targetName });
      }
      // "Proceed" or "Continue Manually" just needs to close the wizard
      onComplete(); 
    } catch (e) {
      error = "Operation Failed: " + e;
    }
  }
</script>

<div class="p-6 max-w-2xl mx-auto bg-card rounded-xl shadow-lg border relative">
  <!-- Close Button -->
  <button onclick={onCancel} class="absolute top-4 right-4 text-muted-foreground hover:text-foreground">
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
  </button>

  <div class="mb-6">
      <h2 class="text-2xl font-bold tracking-tight">
          {huntId ? "Edit Operation Details" : "New Operation"}
      </h2>
      <p class="text-muted-foreground">
          {step === 1 ? "Identify the target entity for this Qui Tam action." : "Confirm federal nexus intelligence."}
      </p>
  </div>
  
  {#if error}
      <div class="bg-destructive/10 text-destructive p-3 rounded-md mb-4 text-sm font-medium">
          {error}
      </div>
  {/if}

  {#if step === 1}
      <div class="space-y-4">
          <div class="space-y-2">
              <label for="targetName" class="text-sm font-medium">Target Entity Name</label>
              <input 
                  id="targetName"
                  type="text" 
                  bind:value={targetName}
                  placeholder="e.g. Acme Healthcare Services, LLC"
                  class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                  onkeydown={(e) => e.key === 'Enter' && handleVerify()}
              />
              <p class="text-xs text-muted-foreground">
                  Use the exact legal name if known.
              </p>
          </div>
          
          <div class="pt-4 flex justify-end">
              <button 
                  onclick={handleVerify} 
                  disabled={!targetName || isVerifying}
                  class="bg-primary text-primary-foreground hover:bg-primary/90 px-4 py-2 rounded-md font-medium transition-colors disabled:opacity-50 flex items-center gap-2"
              >
                  {#if isVerifying}
                      <span class="animate-spin">⏳</span> Verifying...
                  {:else}
                      Verify Target against USASpending &rarr;
                  {/if}
              </button>
          </div>
      </div>

  {:else if step === 2}
      <div class="space-y-6">
        
        <!-- Case 1: Results Found -->
        {#if verificationResults.length > 0}
           <div class="bg-green-500/10 border border-green-500/30 p-4 rounded text-sm space-y-3">
               <div class="flex items-center gap-2 text-green-700 font-bold text-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
                  High-value matches found!
               </div>
               <p class="text-muted-foreground">USASpending.gov returned records matching "{targetName}". This establishes a clear federal nexus.</p>
               
               <div class="bg-background border rounded-md divide-y max-h-60 overflow-y-auto mt-2">
                    {#each verificationResults as res}
                      <div class="p-3 text-sm hover:bg-muted/50">
                        <div class="flex justify-between items-center mb-1">
                          <span class="font-bold text-foreground">{res.recipient_name}</span>
                          <span class="font-mono text-green-600 font-medium">${res.total_obligation?.toLocaleString() ?? '0'}</span>
                        </div>
                        <div class="text-[10px] uppercase tracking-wide text-muted-foreground mt-1 flex gap-2">
                           <span>{res.awarding_agency}</span>
                           <span>•</span>
                           <span>{res.date_signed}</span>
                        </div>
                      </div>
                    {/each}
               </div>
           </div>

        <!-- Case 2: No Results -->
        {:else}
           <div class="bg-yellow-500/10 border-l-4 border-yellow-500 p-4 rounded text-sm text-yellow-700 dark:text-yellow-500 space-y-3">
              <div class="flex items-center gap-2 font-bold text-lg">
                  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
                  No Direct Matches Found
              </div>
              <p>USASpending.gov did not return high-value contract matches for <strong>"{targetName}"</strong>.</p>
              <div class="bg-background/50 p-3 rounded text-xs space-y-1 border border-yellow-500/20">
                  <strong class="block text-foreground/80 mb-1">This is common because:</strong>
                  <ul class="list-disc pl-4 space-y-0.5 opacity-90">
                     <li>Entity might be a sub-contractor (not direct recipient)</li>
                     <li>Grants/Loans might be under a different name or EIN</li>
                     <li>Fraud may involve false certifications without formal award</li>
                  </ul>
              </div>
              <p class="font-medium pt-2">Zero results does NOT invalidate your case. Proceed manually if you have evidence.</p>
           </div>
        {/if}

        <!-- Action Buttons -->
        <div class="flex gap-3 pt-4 border-t mt-4">
           <button
             type="button"
             onclick={() => step = 1}
             class="px-4 py-2 rounded-md font-medium text-muted-foreground hover:bg-muted transition-colors text-sm"
           >
              &larr; Back to Search
           </button>
           <button
             type="button"
             onclick={handleCreate}
             class="flex-1 bg-primary text-primary-foreground hover:bg-primary/90 py-2 rounded-md font-bold shadow-sm transition-colors flex items-center justify-center gap-2"
           >
              {#if verificationResults.length > 0}
                 Confirm & Proceed &rarr;
              {:else}
                 Continue Manually &rarr;
              {/if}
           </button>
        </div>
      </div>
  {/if}
</div>
