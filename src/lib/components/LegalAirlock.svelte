<script lang="ts">
  let { onUnlock } = $props();
  let confirmationText = $state("");
  let error = $state("");

  function handleSubmit() {
    if (confirmationText.trim() === "I UNDERSTAND") {
      onUnlock();
    } else {
      error = "You must type 'I UNDERSTAND' exactly to proceed.";
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-sm">
  <div class="w-full max-w-2xl p-8 space-y-8 bg-card border border-border rounded-lg shadow-2xl">
    <div class="space-y-4 text-center">
      <h1 class="text-3xl font-bold tracking-tighter text-destructive">LEGAL AIRLOCK</h1>
      <p class="text-lg font-medium text-muted-foreground uppercase tracking-widest">Mandatory Disclosure</p>
    </div>

    <div class="p-6 bg-muted/20 border border-muted rounded-md text-sm leading-relaxed space-y-4 h-64 overflow-y-auto">
      <div class="prose dark:prose-invert max-w-none">
        <p class="font-bold text-red-500">WARNING: USE AT YOUR OWN RISK.</p>
        <p>
          Open Season is free, open-source software provided AS-IS, without any warranty of any kind, express or implied. 
          By using this software, you acknowledge that you are solely responsible for your actions.
        </p>
        <p>
          This software is <strong>NOT</strong> legal advice. The developers, contributors, and maintainers of Open Season 
          assume no liability for any legal consequences, damages, or losses resulting from the use or misuse of this toolkit.
        </p>
        <p>
          Evidence collected using this tool may or may not be admissible in court. Laws regarding recording, data collection, 
          and privacy vary by jurisdiction. You are responsible for knowing and complying with all applicable local, state, 
          and federal laws.
        </p>
        <p>
          Do not use this software to violate the Computer Fraud and Abuse Act (CFAA) or any other statute. 
          <strong>This is a tool for legitimate oversight, not a weapon for harassment.</strong>
        </p>
      </div>
    </div>

    <div class="space-y-4">
      <label for="confirmation" class="block text-sm font-medium text-center">
        Type <span class="font-mono font-bold text-primary">I UNDERSTAND</span> to bypass this airlock.
      </label>
      <input
        id="confirmation"
        type="text"
        bind:value={confirmationText}
        placeholder="I UNDERSTAND"
        class="flex h-12 w-full rounded-md border border-input bg-background px-3 py-2 text-lg text-center ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
        onkeydown={(e) => e.key === 'Enter' && handleSubmit()}
      />
      {#if error}
        <p class="text-sm font-medium text-destructive text-center">{error}</p>
      {/if}
    </div>

    <button
      onclick={handleSubmit}
      class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2 w-full"
    >
      Initialize Hunt
    </button>
  </div>
</div>
