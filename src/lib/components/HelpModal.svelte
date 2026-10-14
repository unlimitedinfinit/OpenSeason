<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { invoke } from "@tauri-apps/api/core";
  
  let { isOpen = $bindable(false) } = $props();

  function close() {
    isOpen = false;
  }

  async function triggerPurge() {
    const confirmation = prompt(
      "This deletes unsealed local hunts only. Sealed Confidential vaults are often the only encrypted copy and are skipped unless you also type the vault password.\n\nType PURGE to delete unsealed hunts (sealed cases are kept):"
    );
    if (confirmation !== "PURGE") {
      alert("Purge cancelled.");
      return;
    }
    const password = prompt(
      "To also delete sealed Confidential vaults, enter the vault password. Leave empty to keep sealed cases."
    );
    try {
      const result = await invoke<string>("purge_vault_cache", {
        password: password && password.length ? password : null,
        includeSealed: !!(password && password.length),
      });
      alert(result);
      close();
      window.location.reload();
    } catch (e) {
      alert("Failed to delete hunts: " + e);
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6" transition:fade={{ duration: 200 }}>
    <!-- Backdrop -->
    <div 
      class="absolute inset-0 bg-background/80 backdrop-blur-sm" 
      onclick={close}
      aria-hidden="true"
    ></div>

    <!-- Modal Content -->
    <div 
      class="relative w-full max-w-4xl max-h-[90vh] flex flex-col bg-card border rounded-lg shadow-xl overflow-hidden"
      transition:scale={{ start: 0.95, duration: 200 }}
      role="dialog"
      aria-modal="true"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-6 border-b">
        <div>
          <h2 class="text-2xl font-bold">How Qui Tam Works</h2>
          <p class="text-sm text-muted-foreground mt-1">A Practical Overview of the False Claims Act</p>
        </div>
        <button onclick={close} class="p-2 hover:bg-accent rounded-full transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
        </button>
      </div>

      <!-- Scrollable Content -->
      <div class="flex-1 overflow-y-auto p-6 space-y-8 text-sm sm:text-base leading-relaxed">
        
        <!-- Power Tool Disclaimer -->
        <div class="p-4 bg-destructive/10 border border-destructive/20 rounded-md text-destructive dark:text-red-400 text-sm font-medium space-y-2">
          <p class="font-bold flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/></svg>
            DISCLAIMER & TERMS OF USE
          </p>
          <p>
            This app provides <strong>automation tools</strong> to help you prepare and organize a qui tam filing. 
            It includes wizards, document generators, and evidence bundling features. 
            <strong>These are automation aids, NOT legal advice.</strong>
          </p>
          <p>
            By using these tools, you acknowledge that Open Season is provided <strong>AS-IS with NO WARRANTY</strong>, 
            does not create an attorney-client relationship, and that <strong>you assume all risk</strong>.
            You must consult an experienced qui tam attorney before filing.
          </p>
        </div>

        <!-- Section 1 -->
        <section>
          <h3 class="text-xl font-semibold mb-3 flex items-center gap-2">
            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary text-sm font-bold">1</span>
            Understanding Your Tools
          </h3>
          <ul class="list-disc pl-12 space-y-2 text-muted-foreground">
            <li><strong>The Goal:</strong> To organize evidence "with particularity" (Rule 9(b)) for a False Claims Act lawsuit.</li>
            <li><strong>The Reward:</strong> 15 to 30% of recovered funds if successful (31 U.S.C. §§ 3729-3733).</li>
            <li><strong>The Risk:</strong> Public disclosure bars your reward. <em>Keep everything in this vault until filed under seal.</em></li>
          </ul>
        </section>

        <!-- Section 2 -->
        <section>
          <h3 class="text-xl font-semibold mb-3 flex items-center gap-2">
            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary text-sm font-bold">2</span>
            Filing Requirements (Technical Details)
          </h3>
          <div class="ml-12 space-y-4">
            <div>
              <strong class="block text-foreground mb-1">Required Forms</strong>
              <ul class="list-disc pl-5 text-muted-foreground space-y-1">
                <li><strong>Civil Complaint:</strong> The main lawsuit. Must be filed <em>in camera</em> and <em>under seal</em>.</li>
                <li><strong>Disclosure Statement:</strong> (Generated by app) "Substantially all material evidence" possessed. Served on DOJ only.</li>
                <li><strong>Civil Cover Sheet (JS 44):</strong> Mandatory form. Check "Federal Question" and Nature of Suit "False Claims Act" (375/376).</li>
                <li><strong>Summons (AO 440):</strong> Prepared but <strong>NOT</strong> served on the defendant initially.</li>
              </ul>
            </div>
            <div>
              <strong class="block text-foreground mb-1">Evidence Organization</strong>
              <p class="text-muted-foreground">
                Judges look for <em>specificity</em>. Don't just say "fraud happened." 
                Use the app to link specific invoices (Who) to specific dates (When) and amounts (How much).
                Organize attachments chronologically in the generated bundle.
              </p>
            </div>
          </div>
        </section>

        <!-- Section 3 -->
        <section>
          <h3 class="text-xl font-semibold mb-3 flex items-center gap-2">
            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary text-sm font-bold">3</span>
            The Filing Workflow
          </h3>
          <div class="ml-12 space-y-4 text-muted-foreground">
             <ol class="list-decimal pl-5 space-y-3">
              <li><strong>Draft & Bundle:</strong> Use "Generate Report" to create your Disclosure Statement. Export your evidence bundle properly labeled.</li>
              <li><strong>Secure Counsel:</strong> Take these organized files to a qui tam attorney. (See resources below).</li>
              <li><strong>File & Serve Government:</strong> Your attorney will file the Complaint under seal and serve the Disclosure Statement on the U.S. Attorney + Attorney General via Certified Mail (Rule 4(i)).</li>
              <li><strong>Wait (The Seal):</strong> The case stays secret for 60+ days (often years) while DOJ investigates. Do not talk to the press.</li>
             </ol>
          </div>
        </section>

        <!-- Section 4 -->
        <section>
          <h3 class="text-xl font-semibold mb-3 flex items-center gap-2">
            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary text-sm font-bold">4</span>
            Important Warnings & Resources
          </h3>
          <div class="ml-12 space-y-3 text-muted-foreground">
            <p>The process is highly technical and risk. <strong>Public disclosure can destroy your case.</strong> Retaliation protection exists under 31 U.S.C. § 3730(h).</p>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
              <a href="https://www.justice.gov/civil/false-claims-act" target="_blank" class="p-3 border rounded hover:bg-muted transition-colors text-sm flex items-center justify-between group">
                DOJ False Claims Act
                <svg class="opacity-0 group-hover:opacity-100 transition-opacity" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
              </a>
              <a href="https://www.taf.org" target="_blank" class="p-3 border rounded hover:bg-muted transition-colors text-sm flex items-center justify-between group">
                Taxpayers Against Fraud
                <svg class="opacity-0 group-hover:opacity-100 transition-opacity" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
              </a>
              <a href="https://www.whistleblowers.org" target="_blank" class="p-3 border rounded hover:bg-muted transition-colors text-sm flex items-center justify-between group">
                National Whistleblower Center
                <svg class="opacity-0 group-hover:opacity-100 transition-opacity" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
              </a>
              <a href="https://www.uscourts.gov/forms" target="_blank" class="p-3 border rounded hover:bg-muted transition-colors text-sm flex items-center justify-between group">
                Federal Court Forms
                <svg class="opacity-0 group-hover:opacity-100 transition-opacity" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
              </a>
            </div>
          </div>
        </section>

        <!-- Section 5 -->
        <section>
          <h3 class="text-xl font-semibold mb-3 flex items-center gap-2 text-destructive">
            <span class="flex items-center justify-center w-8 h-8 rounded-full bg-destructive/10 text-destructive text-sm font-bold">5</span>
            Data Hygiene & Delete Local Hunts
          </h3>
          <div class="ml-12 space-y-4">
            <p class="text-muted-foreground">
              Case files stay on this computer. The only network call in this app is the optional USAspending lookup: it sends the target name you type to api.usaspending.gov and does not upload your vault, evidence, or case folder. Account sync, cloud backup, and publish are stubbed and refuse Confidential cases. There is no telemetry.
            </p>
            
            <div class="p-4 bg-muted/50 rounded-md border border-border space-y-2">
              <strong class="block text-foreground text-sm font-mono">Local Vault Locations:</strong>
              <ul class="list-disc pl-5 text-xs text-muted-foreground space-y-1 font-mono">
                <li><strong>Windows:</strong> %LOCALAPPDATA%\com.openseason.app\vaults</li>
                <li><strong>macOS:</strong> $HOME/Library/Application Support/com.openseason.app/vaults</li>
                <li><strong>Linux:</strong> $HOME/.local/share/com.openseason.app/vaults</li>
              </ul>
              <p class="text-xs text-muted-foreground mt-2">
                To delete evidence by hand, you may remove individual unsealed hunt folders. Do not delete a sealed vault folder unless you have a copy: that folder is often the only encrypted case.
              </p>
            </div>

            <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between p-4 border border-destructive/20 bg-destructive/5 rounded-md gap-4">
              <div class="space-y-1 flex-1">
                <strong class="block text-destructive">Delete local hunts</strong>
                <p class="text-xs text-muted-foreground">
                  Deletes unsealed hunts. Sealed Confidential vaults (often the only copy) are skipped unless you re-enter the vault password.
                </p>
              </div>
              <button 
                onclick={triggerPurge}
                class="px-4 py-2 bg-destructive text-destructive-foreground hover:bg-destructive/90 rounded-md font-medium text-xs transition-colors shrink-0"
              >
                Delete local hunts
              </button>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="p-4 border-t bg-muted/50 flex justify-end">
        <button 
          onclick={close} 
          class="px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md font-medium transition-colors"
        >
          Close Instructions
        </button>
      </div>
    </div>
  </div>
{/if}
