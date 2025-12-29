<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { onComplete, onCancel } = $props();

  let step = $state(1);
  let targetName = $state("");
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
      await invoke("create_hunt", { name: targetName });
      onComplete();
    } catch (e) {
      error = "Creation Failed: " + e;
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
            <h3 class="text-lg font-medium">Preliminary Scouts ({verificationResults.length})</h3>
            <span class="text-xs bg-muted px-2 py-1 rounded">USASpending.gov</span>
          </div>
          
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
             {#if verificationResults.length === 0}
               <div class="p-4 text-center text-muted-foreground">
                  No direct high-value matches found. Manual investigation required.
               </div>
             {/if}
          </div>

          <div class="bg-yellow-500/10 border border-yellow-500/50 p-4 rounded-md text-sm text-yellow-500">
             <strong>Advisory:</strong> Validating a False Claims Act case requires proof of *fraud*, not just government funds. Ensure you have specific evidence of distinct billing schemes.
          </div>
        </div>
      {/if}
    </div>

    <div class="p-6 border-t bg-muted/20 flex justify-between items-center">
      <button onclick={onCancel} class="text-sm hover:underline">Cancel</button>
      
      <button
        onclick={step === 1 ? handleVerify : handleCreate}
        disabled={isVerifying || !targetName}
        class="bg-primary text-primary-foreground px-6 py-2 rounded-md font-medium disabled:opacity-50 flex items-center gap-2"
      >
        {#if isVerifying}
          <span>Scouting...</span>
        {:else}
           {step === 1 ? "Verify Target" : "Establish Blind"}
        {/if}
      </button>
    </div>
  </div>
</div>
