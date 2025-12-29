<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import HuntWizard from "$lib/components/HuntWizard.svelte";

  let huntId = $derived($page.params.id);
  // Show wizard if ?demo=true OR ?wizard=true
  let showWizard = $derived($page.url.searchParams.get('demo') === 'true' || $page.url.searchParams.get('wizard') === 'true');
  let isDemo = $derived($page.url.searchParams.get('demo') === 'true');
  let isEditingName = $state(false);
  let editNameValue = $state("");

  function startEditName() {
      if (!huntData) return;
      editNameValue = huntData.name;
      isEditingName = true;
  }

  async function saveName() {
      if (!huntData || !editNameValue) return;
      try {
          await invoke("update_hunt", { huntId: huntData.id, name: editNameValue });
          huntData.name = editNameValue;
          isEditingName = false;
      } catch (e) {
          alert("Failed to update name: " + e);
      }
  }

  // ... (existing code)

  // In the Quick Actions section or Header:
  // Let's modify the Quick Actions "Edit Case Details" to trigger this inline edit


  onMount(async () => {
     await loadHuntData();
  });

  async function loadHuntData() {
      // Temporary: fetch list and find matching ID
      // Ideal: backend "get_hunt" command
      try {
          const list: any[] = await invoke('list_hunts');
          huntData = list.find(h => h.id === huntId);
      } catch(e) {
          console.error("Failed to load hunt", e);
      }
  }

  function handleWizardComplete() {
      // Params removal triggers showWizard -> false automatically via $derived
      // Remove params via history state (simplest without reload)
      const url = new URL(window.location.href);
      url.searchParams.delete('demo');
      url.searchParams.delete('wizard');
      window.history.replaceState({}, '', url);
      loadHuntData(); // Refresh data
  }

  async function generateReport() {
    if (!huntData) return;
    if (!confirm(`Generate Disclosure Statement for ${huntData.name}?`)) return;
    try {
      const path = await invoke("save_disclosure_cmd", { 
        huntId: huntData.id, 
        target: huntData.name, 
        count: 0, // Placeholder
        value: 0.0 // Placeholder
      });
      alert("Report Saved: " + path);
    } catch (e) {
      alert("Report Failed: " + e);
    }
  }
  
  async function exportHunt() {
      if (!huntData) return;
      // User update: Auto-save to Downloads
      try {
          const path = await invoke("export_hunt_cmd", { huntId: huntData.id, targetPath: "DOWNLOADS" });
          alert("Export Successful!\nSaved to: " + path);
      } catch(e) {
          alert("Export Failed: " + e);
      }
  }

  let activeTab = $state("overview"); // overview, evidence, documents

  // Mock Evidence (Live would list from vault)
  let evidenceList = $state<string[]>(isDemo ? ["Contract_Award_2021.pdf", "Internal_Email_Thread.eml", "Financial_Spreadsheet_Q3.xlsx"] : []);

  // Checklist Items
  let checklist = $state([
      { id: "cover_sheet", label: "Civil Cover Sheet (JS 44)", required: true, checked: false },
      { id: "complaint", label: "Complaint (Sealed)", required: true, checked: false },
      { id: "disclosure", label: "Disclosure Statement", required: true, checked: false },
      { id: "summons", label: "Summons (AO 440)", required: true, checked: false },
      { id: "service", label: "Certificate of Service", required: false, checked: false }
  ]);
  
  function addEvidence() {
      // Placeholder for file picker
      const input = document.createElement('input');
      input.type = 'file';
      input.onchange = (e: any) => {
          if (e.target.files.length > 0) {
              const file = e.target.files[0];
              evidenceList = [...evidenceList, file.name];
              alert(`Encrypted & Added: ${file.name}`);
              // TODO: Implement actual file copy/encrypt to vault
          }
      };
      input.click();
  }

  function toggleCheck(id: string) {
      checklist = checklist.map(i => i.id === id ? {...i, checked: !i.checked} : i);
  }
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col">
   <!-- Header -->
   <header class="border-b p-4 px-8 flex justify-between items-center bg-card shadow-sm sticky top-0 z-10">
      <div class="flex items-center gap-4">
         <button onclick={() => goto('/')} class="text-muted-foreground hover:text-foreground">
             <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
         </button>
         <div>
            {#if isEditingName}
                <div class="flex items-center gap-2">
                    <input 
                        type="text" 
                        bind:value={editNameValue} 
                        class="border rounded px-2 py-1 text-lg font-bold bg-background text-foreground"
                        onkeydown={(e) => e.key === 'Enter' && saveName()}
                    />
                    <button onclick={saveName} class="bg-primary text-primary-foreground px-3 py-1 rounded text-xs">Save</button>
                    <button onclick={() => isEditingName = false} class="bg-muted text-muted-foreground px-3 py-1 rounded text-xs">Cancel</button>
                </div>
            {:else}
                <h1 class="text-xl font-bold tracking-tight flex items-center gap-2">
                    {huntData ? huntData.name : "Loading..."}
                    <span class="text-xs font-normal px-2 py-0.5 rounded-full bg-yellow-100 text-yellow-800 border border-yellow-200">Draft</span>
                    <button onclick={startEditName} class="ml-2 text-muted-foreground hover:text-primary opacity-50 hover:opacity-100" title="Edit Name">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>
                    </button>
                </h1>
            {/if}
         </div>
      </div>
      <div class="flex gap-2">
         <button onclick={exportHunt} class="text-sm border bg-background hover:bg-accent px-3 py-1.5 rounded flex items-center gap-2">
             <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
             Export .osb
         </button>
      </div>
   </header>



  <!-- Navigation Tabs -->
  <div class="border-b bg-muted/30 px-8">
      <div class="flex gap-6">
          <button 
            onclick={() => activeTab = "overview"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'overview' ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Overview
          </button>
          <button 
            onclick={() => activeTab = "evidence"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'evidence' ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Evidence Locker
          </button>
          <button 
            onclick={() => activeTab = "documents"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'documents' ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Legal Documents
          </button>
      </div>
  </div>

  <!-- Main Content Area -->
  <main class="flex-1 p-8 max-w-6xl mx-auto w-full">
      
      <!-- OVERVIEW TAB -->
      {#if activeTab === "overview"}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
              <div class="md:col-span-2 space-y-6">
                  <section class="bg-card border rounded-lg p-6 shadow-sm">
                      <h2 class="text-lg font-semibold mb-4">Case Narrative / Memo</h2>
                      <textarea class="w-full h-64 bg-muted/10 border rounded p-3 text-sm resize-none focus:ring-1 focus:ring-primary" placeholder="Draft your internal case memo here. Document the who, what, where, when, and how of the fraud scheme..."></textarea>
                      <div class="mt-2 text-right">
                          <button class="text-xs bg-primary text-primary-foreground px-3 py-1 rounded hover:bg-primary/90">Save Memo</button>
                      </div>
                  </section>
              </div>

              <div class="space-y-6">
                  <div class="bg-card border rounded-lg p-6 space-y-4">
                      <h3 class="font-medium text-sm text-muted-foreground uppercase tracking-wider">Quick Actions</h3>
                      <button onclick={startEditName} class="w-full text-left text-sm py-2 px-3 hover:bg-muted rounded flex items-center gap-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                          Rename Case
                      </button>
                      <button onclick={generateReport} class="w-full text-left text-sm py-2 px-3 hover:bg-muted rounded flex items-center gap-2 text-red-600 hover:bg-red-50">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
                          Generate Disclosure PDF
                      </button>
                      <hr />
                      <button onclick={deleteHunt} class="w-full text-left text-sm py-2 px-3 hover:bg-muted rounded flex items-center gap-2 text-muted-foreground hover:text-red-500">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
                          Delete Case
                      </button>
                  </div>
              </div>
          </div>

      <!-- EVIDENCE TAB -->
      {:else if activeTab === "evidence"}
          <div class="bg-card border rounded-lg shadow-sm">
              <div class="p-4 border-b flex justify-between items-center bg-muted/10">
                  <h2 class="font-semibold">Evidence Locker</h2>
                  <button onclick={addEvidence} class="bg-primary text-primary-foreground px-4 py-2 rounded text-sm hover:bg-primary/90 flex items-center gap-2">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                      Add Evidence
                  </button>
              </div>
              <div class="p-0">
                  {#if evidenceList.length === 0}
                      <div class="p-12 text-center text-muted-foreground">
                          <div class="mx-auto h-12 w-12 text-muted-foreground/30 mb-3">
                              <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="13 2 13 9 20 9"/></svg>
                          </div>
                          <p>No evidence files encrypted yet.</p>
                          <p class="text-xs opacity-70 mt-1">Files are stored securely in the vault.</p>
                      </div>
                  {:else}
                      <ul class="divide-y">
                          {#each evidenceList as file}
                              <li class="p-4 flex items-center justify-between hover:bg-muted/50">
                                  <div class="flex items-center gap-3">
                                      <div class="h-10 w-10 bg-blue-50 text-blue-600 rounded flex items-center justify-center">
                                          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
                                      </div>
                                      <div>
                                          <p class="font-medium text-sm">{file}</p>
                                          <p class="text-xs text-muted-foreground">Encrypted • Added just now</p>
                                      </div>
                                  </div>
                                  <button class="text-muted-foreground hover:text-destructive p-2">
                                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                  </button>
                              </li>
                          {/each}
                      </ul>
                  {/if}
              </div>
          </div>

      <!-- DOCUMENTS TAB -->
      {:else if activeTab === "documents"}
          <div class="space-y-6">
              <div class="bg-blue-50 border border-blue-200 p-4 rounded text-sm text-blue-800 flex gap-3">
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
                  <div>
                      <strong>Qui Tam Filing Checklist:</strong>
                      <p class="mt-1">Standard documents required for a False Claims Act filing. Generating standard forms will download a prepopulated PDF to your Downloads folder.</p>
                  </div>
              </div>

              <div class="bg-card border rounded-lg shadow-sm">
                  <div class="divide-y">
                      {#each checklist as item}
                          <div class="p-4 flex items-center justify-between hover:bg-muted/20">
                              <div class="flex items-center gap-4">
                                  <button 
                                      onclick={() => toggleCheck(item.id)}
                                      class="h-6 w-6 rounded border flex items-center justify-center transition-colors {item.checked ? 'bg-green-600 border-green-600 text-white' : 'border-input hover:border-primary'}"
                                  >
                                      {#if item.checked}
                                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                                      {/if}
                                  </button>
                                  <div>
                                      <p class="font-medium {item.checked ? 'text-muted-foreground line-through' : 'text-foreground'}">
                                          {item.label}
                                          {#if item.required}<span class="text-destructive text-xs ml-2">*Required</span>{/if}
                                      </p>
                                  </div>
                              </div>
                              <div class="flex gap-2">
                                  {#if item.id === 'disclosure'}
                                      <button onclick={generateReport} class="text-xs bg-muted hover:bg-accent border px-3 py-1.5 rounded">Generate Draft</button>
                                  {:else if item.id === 'cover_sheet' || item.id === 'summons'}
                                      <a href="https://www.uscourts.gov/forms/civil-forms" target="_blank" class="text-xs text-primary hover:underline px-3 py-1.5">Download Official Form</a>
                                  {:else}
                                       <span class="text-xs text-muted-foreground italic px-3 py-1.5">Template N/A</span>
                                  {/if}
                              </div>
                          </div>
                      {/each}
                  </div>
              </div>
          </div>
      {/if}
  </main>
</div>

{#if showWizard}
   <HuntWizard 
     huntId={huntId}
     demoMode={isDemo}
     initialName={huntData ? huntData.name : ""}
     onComplete={handleWizardComplete}
     onCancel={() => {
        // Params removal via handleWizardComplete triggers showWizard -> false
        handleWizardComplete();
     }}
   />
{/if}
