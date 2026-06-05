<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import HuntWizard from "$lib/components/HuntWizard.svelte";

  let huntId = $derived($page.params.id);
  let showWizard = $derived($page.url.searchParams.get('demo') === 'true' || $page.url.searchParams.get('wizard') === 'true');
  let isDemo = $derived($page.url.searchParams.get('demo') === 'true');
  
  // Reactivity State
  let huntData = $state<any>(null);
  let activeTab = $state("overview"); // overview, timeline, evidence, parties, pleading
  let isEditingName = $state(false);
  let editNameValue = $state("");
  let estimatedValue = $state<number>(0);

  // Timeline events state
  let events = $state<any[]>([]);
  let newEventTitle = $state("");
  let newEventDate = $state("");
  let newEventDesc = $state("");
  let newEventType = $state("other");
  let showAddEventModal = $state(false);

  // Witnesses/Parties state
  let parties = $state<any[]>([]);
  let newPartyName = $state("");
  let newPartyRole = $state("Witness");
  let newPartyEmail = $state("");
  let newPartyPhone = $state("");
  let newPartyNotes = $state("");
  let showAddPartyModal = $state(false);

  // Evidence files state
  let evidenceList = $state<any[]>([]);
  let isDragging = $state(false);

  // Complaint draft sections state
  let complaintSections = $state<Record<string, string>>({
      introduction: "",
      jurisdiction: "",
      parties: "",
      facts: "",
      violations: ""
  });
  let activeSection = $state("introduction");

  onMount(async () => {
     await loadHuntData();
  });

  async function loadHuntData() {
      try {
          const list: any[] = await invoke('list_hunts');
          huntData = list.find(h => h.id === huntId);
          if (huntData) {
              editNameValue = huntData.name;
              
              // Load subcollections from SQLite DB
              await Promise.all([
                  loadTimeline(),
                  loadParties(),
                  loadComplaintSections(),
                  loadEvidence()
              ]);
          }
      } catch(e) {
          console.error("Failed to load hunt", e);
      }
  }

  async function loadTimeline() {
      try {
          events = await invoke("get_hunt_timeline", { huntId });
      } catch (e) {
          console.error("Timeline error", e);
      }
  }

  async function loadParties() {
      try {
          parties = await invoke("get_hunt_parties", { huntId });
      } catch (e) {
          console.error("Parties error", e);
      }
  }

  async function loadComplaintSections() {
      try {
          const sections: any[] = await invoke("get_complaint_sections", { huntId });
          const temp: Record<string, string> = {
              introduction: "",
              jurisdiction: "",
              parties: "",
              facts: "",
              violations: ""
          };
          sections.forEach(s => {
              if (s.section_id in temp) {
                  temp[s.section_id] = s.content || "";
              }
          });
          complaintSections = temp;
      } catch (e) {
          console.error("Sections error", e);
      }
  }

  async function loadEvidence() {
      if (isDemo) {
          evidenceList = [
              { id: 1, file_path: "Contract_Award_2021.pdf", description: "Primary Fraud Contract", sha256_hash: "8f5a2e5d9c..." },
              { id: 2, file_path: "Internal_Billing_Spreadsheet.xlsx", description: "Falsified hours logs", sha256_hash: "a4b2c1d3e8..." },
              { id: 3, file_path: "Audit_Email_Conf.eml", description: "Audit trail warnings ignored", sha256_hash: "f7d2e8b1c4..." }
          ];
          return;
      }
      try {
          evidenceList = await invoke("get_hunt_evidence", { huntId });
      } catch (e) {
          console.error("Evidence fetch failed", e);
      }
  }

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

  async function updateContractValue() {
      if (!huntData) return;
      // Value updates can be logged as variables
      alert(`Contract value set to $${estimatedValue.toLocaleString()}`);
  }

  function handleWizardComplete() {
      const url = new URL(window.location.href);
      url.searchParams.delete('demo');
      url.searchParams.delete('wizard');
      window.history.replaceState({}, '', url);
      loadHuntData();
  }

  async function generateReport() {
    if (!huntData) return;
    if (!confirm(`Generate Confidential Qui Tam Disclosure Statement for ${huntData.name}?`)) return;
    try {
      const path = await invoke("save_disclosure_cmd", { 
        huntId: huntData.id, 
        target: huntData.name, 
        count: evidenceList.length, 
        value: estimatedValue
      });
      alert("Sealed Package generated successfully and saved to Downloads:\n" + path);
    } catch (e) {
      alert("Report Failed: " + e);
    }
  }
  
  async function exportHunt() {
      if (!huntData) return;
      try {
          const path = await invoke("export_hunt_cmd", { huntId: huntData.id, targetPath: "DOWNLOADS" });
          alert("Export Successful!\nOperation Bundle (.osb) saved to Downloads:\n" + path);
      } catch(e) {
          alert("Export Failed: " + e);
      }
  }

  async function deleteHunt() {
      if (!confirm("Are you sure you want to PERMANENTLY delete this case and wipe all evidence? This cannot be undone.")) return;
      try {
          await invoke("delete_hunt", { huntId });
          goto('/');
      } catch (e) {
          alert("Failed to delete: " + e);
      }
  }

  // Timeline add/delete
  async function addTimelineEvent() {
      if (!newEventTitle || !newEventDate) return;
      try {
          await invoke("add_hunt_event", {
              huntId,
              title: newEventTitle,
              description: newEventDesc,
              eventDate: newEventDate,
              eventType: newEventType
          });
          newEventTitle = "";
          newEventDate = "";
          newEventDesc = "";
          newEventType = "other";
          showAddEventModal = false;
          await loadTimeline();
      } catch (e) {
          alert("Failed to add event: " + e);
      }
  }

  async function deleteEvent(eventId: number) {
      if (!confirm("Remove this event from timeline?")) return;
      try {
          await invoke("delete_hunt_event", { huntId, eventId });
          await loadTimeline();
      } catch (e) {
          alert("Failed to delete event: " + e);
      }
  }

  // Parties add/delete
  async function addPartyRecord() {
      if (!newPartyName) return;
      try {
          await invoke("add_hunt_party", {
              huntId,
              name: newPartyName,
              role: newPartyRole,
              email: newPartyEmail,
              phone: newPartyPhone,
              notes: newPartyNotes
          });
          newPartyName = "";
          newPartyRole = "Witness";
          newPartyEmail = "";
          newPartyPhone = "";
          newPartyNotes = "";
          showAddPartyModal = false;
          await loadParties();
      } catch (e) {
          alert("Failed to add party: " + e);
      }
  }

  async function deletePartyRecord(partyId: number) {
      if (!confirm("Remove this individual from registry?")) return;
      try {
          await invoke("delete_hunt_party", { huntId, partyId });
          await loadParties();
      } catch (e) {
          alert("Failed to delete: " + e);
      }
  }

  // Complaint section saving
  async function saveComplaintSection(sectionId: string) {
      try {
          await invoke("save_complaint_section", {
              huntId,
              sectionId,
              content: complaintSections[sectionId]
          });
          alert("Narrative section saved to secure local vault.");
      } catch (e) {
          alert("Failed to save section: " + e);
      }
  }

  // Evidence file picker & uploader
  function addEvidenceFile() {
      const input = document.createElement('input');
      input.type = 'file';
      input.onchange = (e: any) => {
          if (e.target.files.length > 0) {
              const file = e.target.files[0];
              const desc = prompt("Enter a description for this evidence:", "User uploaded exhibit");
              if (desc === null) return; // user cancelled

              if (isDemo) {
                  evidenceList = [...evidenceList, {
                      id: Date.now(),
                      file_path: file.name,
                      description: desc,
                      sha256_hash: "8f5a2e5d9c7b6a5d4c3b2a1e0f9c8b7a6d5c4b3a2e1d0f9"
                  }];
                  alert(`[Demo Mode] Encrypted & Added: ${file.name}`);
                  return;
              }

              // Read file as ArrayBuffer, then Uint8Array
              const reader = new FileReader();
              reader.onload = async (event: any) => {
                  try {
                      const arrayBuffer = event.target.result;
                      const bytes = new Uint8Array(arrayBuffer);
                      // Invoke Rust command to strip metadata, calculate SHA-256, encrypt, and store in DB
                      await invoke("add_hunt_evidence_bytes", {
                          huntId,
                          filename: file.name,
                          fileBytes: Array.from(bytes),
                          description: desc
                      });
                      alert(`Encrypted & Added: ${file.name}\n(Stripped EXIF/Metadata, logged SHA-256 hash, and wrote encrypted bytes to secure vault)`);
                      await loadEvidence();
                  } catch (err) {
                      alert("Failed to encrypt and store evidence: " + err);
                  }
              };
              reader.readAsArrayBuffer(file);
          }
      };
      input.click();
  }

  async function deleteEvidence(evidenceId: number) {
      if (!confirm("Are you sure you want to PERMANENTLY delete this evidence from the vault? This cannot be undone.")) return;
      
      if (isDemo) {
          evidenceList = evidenceList.filter(e => e.id !== evidenceId);
          return;
      }

      try {
          await invoke("delete_hunt_evidence", { huntId, evidenceId });
          await loadEvidence();
      } catch (err) {
          alert("Failed to delete evidence: " + err);
      }
  }

  async function loadPreset(type: "medicare" | "defense" | "sba") {
      if (!confirm("This will overwrite the current draft content in your Memo and Pleading builder. Do you want to load this preset?")) return;

      if (type === "medicare") {
          complaintSections.introduction = `[MEDICARE FRAUD NARRATIVE MEMO]
SUMMARY OF SCHEME:
Relator has direct personal knowledge that the Defendant engaged in a systematic upcoding and billing fraud scheme. Specifically, Defendant knowingly submitted falsified bills to the Medicare Program for services that were: (a) medically unnecessary, (b) not rendered as billed, or (c) performed by unsupervised/unqualified assistants.

PERTINENT REGULATIONS DEFRAUDED:
- Medicare Act, 42 U.S.C. §§ 1395 et seq.
- False Claims Act, 31 U.S.C. §§ 3729(a)(1)(A) (Submission of false claims) & (a)(1)(B) (Use of false records).

TIMELINE OF ANOMALIES:
[Specify date range here, e.g. 2021 to Present]`;

          complaintSections.facts = `STATEMENT OF FACTS (MEDICARE PRESET):
1. Defendant operates a clinical facility providing services to Medicare beneficiaries.
2. Starting on or about [Start Date], Defendant initiated a billing policy of billing for Level 4/5 evaluation and management (E&M) codes (CPT 99214/99215) for short, routine visits that only warranted Level 2/3 billing.
3. Relator observed that patient charts were altered or templated with pre-populated notes indicating complex diagnoses that were never assessed.
4. Attached exhibits (e.g. internal billing audit logs) show that Medicare reimbursed Defendant approximately $[Amount] for these upcoded visits.`;

          complaintSections.violations = `COUNT I
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(A))
(Submission of False Claims)

1. Defendant knowingly presented or caused to be presented false or fraudulent claims for payment or approval to the United States.
2. By upcoding E&M codes and billing for unrendered services, Defendant received federal funds to which it was not entitled.

COUNT II
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(B))
(Use of False Records)

1. Defendant knowingly made, used, or caused to be made or used, false records or statements material to false or fraudulent claims.`;
      } else if (type === "defense") {
          complaintSections.introduction = `[DEFENSE PROCUREMENT OVERBILLING NARRATIVE MEMO]
SUMMARY OF SCHEME:
Relator has direct personal knowledge that Defendant knowingly defrauded the Department of Defense (DoD) by overbilling and billing for non-conforming or defective parts. Defendant falsified quality assurance certifications, manipulated cost and pricing data, and overcharged labor hours on cost-plus contracts.

PERTINENT REGULATIONS DEFRAUDED:
- Truth in Negotiations Act (TINA), 10 U.S.C. § 3701 et seq.
- False Claims Act, 31 U.S.C. §§ 3729(a)(1)(A) & (B).

TIMELINE OF ANOMALIES:
[Specify date range here]`;

          complaintSections.facts = `STATEMENT OF FACTS (DEFENSE PRESET):
1. Defendant holds prime defense contracts for manufacturing parts for [Military Branch/System].
2. Under the contracts, Defendant was required to perform rigorous testing (e.g. stress and temperature testing) and certify compliance.
3. On multiple occasions, Defendant's management ordered engineers to skip tests and sign off on quality assurance reports (falsified certs).
4. Furthermore, Defendant inflated hours on cost-plus contract billings by billing time spent on fixed-price projects.`;

          complaintSections.violations = `COUNT I
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(A))
(Submission of False Claims)

1. Defendant knowingly submitted false bills for non-conforming military hardware and inflated labor hours.

COUNT II
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(B))
(Use of False Records)

1. Defendant certified TINA compliance and QA testing despite knowing the pricing data and test results were fabricated.`;
      } else if (type === "sba") {
          complaintSections.introduction = `[SBA GRANT DIVERSION NARRATIVE MEMO]
SUMMARY OF SCHEME:
Relator has direct personal knowledge that Defendant knowingly defrauded the Small Business Administration (SBA) by diverting federal grant funding (e.g., SBIR/STTR or pandemic relief) for personal enrichment, unauthorized commercial uses, and shell company transactions.

PERTINENT REGULATIONS DEFRAUDED:
- Small Business Act, 15 U.S.C. § 638 et seq.
- False Claims Act, 31 U.S.C. §§ 3729(a)(1)(A) & (B).

TIMELINE OF ANOMALIES:
[Specify date range here]`;

          complaintSections.facts = `STATEMENT OF FACTS (SBA PRESET):
1. Defendant applied for and received SBA grant awards under the SBIR program for research into [Research Topic].
2. SBA regulations require that at least [X]% of research be conducted by the small business and that grant funds be spent on approved R&D costs.
3. Defendant did not perform the research; instead, funds were transferred directly to private personal bank accounts or used to pay down commercial debt.
4. Defendant submitted progress reports falsifying the research hours and achievements.`;

          complaintSections.violations = `COUNT I
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(A))
(Submission of False Claims)

1. Defendant knowingly submitted milestones claims and certifications to the SBA to draw down grant funds.

COUNT II
Violations of the False Claims Act (31 U.S.C. § 3729(a)(1)(B))
(Use of False Records)

1. Defendant fabricated progress reports and timesheets to justify SBA grant expenditures.`;
      }
      
      // Auto-save the sections to database
      if (!isDemo) {
          try {
              await invoke("save_complaint_section", { huntId, sectionId: "introduction", content: complaintSections.introduction });
              await invoke("save_complaint_section", { huntId, sectionId: "facts", content: complaintSections.facts });
              await invoke("save_complaint_section", { huntId, sectionId: "violations", content: complaintSections.violations });
              alert("Preset templates successfully loaded and saved to your secure offline vault.");
          } catch(e) {
              console.error("Auto-save failed", e);
          }
      } else {
          alert("Demo Preset loaded. Click 'Save' or edit sections as needed.");
      }
  }
</script>

<div class="min-h-screen bg-background text-foreground flex flex-col font-sans">
   <!-- Header -->
   <header class="border-b p-4 px-8 flex justify-between items-center bg-card shadow-sm sticky top-0 z-10">
      <div class="flex items-center gap-4">
         <button onclick={() => goto('/')} class="text-muted-foreground hover:text-foreground" aria-label="Go back to dashboard">
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
                    <span class="text-xs font-normal px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">Sealed</span>
                    <button onclick={startEditName} class="ml-2 text-muted-foreground hover:text-primary opacity-50 hover:opacity-100" title="Edit Name" aria-label="Rename Case">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>
                    </button>
                </h1>
            {/if}
         </div>
      </div>
      <div class="flex gap-2">
         <button onclick={exportHunt} class="text-sm border border-border bg-background hover:bg-accent px-3 py-1.5 rounded flex items-center gap-2">
             <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
             Export .osb
         </button>
      </div>
   </header>

   <!-- Navigation Tabs -->
   <div class="border-b bg-card/60 px-8">
      <div class="flex gap-6">
          <button 
            onclick={() => activeTab = "overview"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'overview' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Overview
          </button>
          <button 
            onclick={() => activeTab = "timeline"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'timeline' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Fraud Timeline
          </button>
          <button 
            onclick={() => activeTab = "evidence"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'evidence' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Evidence Locker
          </button>
          <button 
            onclick={() => activeTab = "parties"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'parties' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Witness Directory
          </button>
          <button 
            onclick={() => activeTab = "pleading"}
            class="py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'pleading' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-muted-foreground hover:text-foreground'}"
          >
            Sealed Pleading Builder
          </button>
      </div>
   </div>

   <!-- Main Content Area -->
   <main class="flex-1 p-8 max-w-6xl mx-auto w-full">
      
      <!-- OVERVIEW TAB -->
      {#if activeTab === "overview"}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
              <div class="md:col-span-2 space-y-6">
                  <section class="bg-card border rounded-lg p-6 shadow-sm space-y-4">
                      <div class="flex justify-between items-center">
                          <h2 class="text-lg font-semibold">Investigation Summary</h2>
                          <span class="text-xs font-mono text-muted-foreground">LOCAL VAULT PATH: $HOME/.open-season/vaults/{huntId}</span>
                      </div>
                      <div class="bg-yellow-500/10 border border-yellow-500/20 p-4 rounded text-xs text-yellow-500 flex gap-2">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                          <p>
                              <strong>Zero-Trust Guardrail:</strong> This case is isolated on disk. Evidentiary files are secured using XChaCha20Poly1305. To share this securely with legal counsel, export the bundle as a encrypted `.osb` file.
                          </p>
                      </div>
                       <div class="space-y-2 border-t pt-4">
                           <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Qui Tam Triage Presets</span>
                           <p class="text-xs text-muted-foreground">Select a preset template to bootstrap your Case Narrative Memo and Pleading outline:</p>
                           <div class="flex flex-wrap gap-2 pt-1">
                               <button 
                                   onclick={() => loadPreset("medicare")} 
                                   class="text-xs border border-indigo-500/30 bg-indigo-500/5 hover:bg-indigo-500/10 text-indigo-400 px-3 py-1.5 rounded transition-all flex items-center gap-1.5 font-medium"
                               >
                                   <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/></svg>
                                   Medicare Fraud
                               </button>
                               <button 
                                   onclick={() => loadPreset("defense")} 
                                   class="text-xs border border-indigo-500/30 bg-indigo-500/5 hover:bg-indigo-500/10 text-indigo-400 px-3 py-1.5 rounded transition-all flex items-center gap-1.5 font-medium"
                               >
                                   <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                                   Defense Procurement
                               </button>
                               <button 
                                   onclick={() => loadPreset("sba")} 
                                   class="text-xs border border-indigo-500/30 bg-indigo-500/5 hover:bg-indigo-500/10 text-indigo-400 px-3 py-1.5 rounded transition-all flex items-center gap-1.5 font-medium"
                               >
                                   <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="2" x2="12" y2="22"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>
                                   SBA Grant Diversion
                               </button>
                           </div>
                       </div>
                       <div class="space-y-2">
                          <label for="narrative-memo" class="text-sm font-medium text-muted-foreground">Case Narrative Memo</label>
                          <textarea 
                              id="narrative-memo"
                              class="w-full h-64 bg-muted/10 border border-border rounded p-3 text-sm resize-none focus:ring-1 focus:ring-primary focus:outline-none" 
                              placeholder="Describe the fraud scheme in detail. Who authorized the billing irregularities? Which federal program was defrauded?..."
                              bind:value={complaintSections.introduction}
                          ></textarea>
                      </div>
                      <div class="flex justify-between items-center">
                          <small class="text-xs text-muted-foreground">Saves directly into SQLite metadata.db</small>
                          <button onclick={() => saveComplaintSection("introduction")} class="text-xs bg-primary text-primary-foreground px-4 py-1.5 rounded hover:bg-primary/90">Save Memo</button>
                      </div>
                  </section>
              </div>

              <!-- Sidebar Panel -->
              <div class="space-y-6">
                  <div class="bg-card border rounded-lg p-6 space-y-4 shadow-sm">
                      <h3 class="font-medium text-xs text-muted-foreground uppercase tracking-wider">Scope Parameters</h3>
                      
                      <div class="space-y-2">
                          <label for="est-value" class="text-xs text-muted-foreground">Estimated Contract Value ($)</label>
                          <input 
                              id="est-value"
                              type="number" 
                              bind:value={estimatedValue}
                              class="w-full bg-background border border-border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500" 
                              placeholder="e.g. 1500000"
                              onchange={updateContractValue}
                          />
                      </div>

                      <div class="pt-4 border-t space-y-2">
                          <strong class="text-xs text-muted-foreground uppercase tracking-wide">Qui Tam Statistics</strong>
                          <div class="grid grid-cols-2 gap-4 text-center pt-2">
                              <div class="p-3 bg-muted/20 border border-border rounded">
                                  <span class="block text-2xl font-bold font-mono text-indigo-400">{events.length}</span>
                                  <span class="text-[10px] text-muted-foreground uppercase">Events Logged</span>
                              </div>
                              <div class="p-3 bg-muted/20 border border-border rounded">
                                  <span class="block text-2xl font-bold font-mono text-indigo-400">{evidenceList.length}</span>
                                  <span class="text-[10px] text-muted-foreground uppercase">Exhibits Vault</span>
                              </div>
                          </div>
                      </div>
                  </div>

                  <div class="bg-card border rounded-lg p-6 space-y-4 shadow-sm">
                      <h3 class="font-medium text-xs text-muted-foreground uppercase tracking-wider">Quick Actions</h3>
                      <button onclick={generateReport} class="w-full text-left text-sm py-2.5 px-3 bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-400 border border-indigo-500/20 rounded flex items-center gap-2 font-medium transition-all">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
                          Compile Disclosure Package
                      </button>
                      <button onclick={deleteHunt} class="w-full text-left text-sm py-2 px-3 hover:bg-muted rounded flex items-center gap-2 text-muted-foreground hover:text-red-500 transition-colors">
                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
                          Purge Vault File Cache
                      </button>
                  </div>
              </div>
          </div>

      <!-- TIMELINE TAB -->
      {:else if activeTab === "timeline"}
          <div class="bg-card border rounded-lg shadow-sm">
              <div class="p-4 border-b flex justify-between items-center bg-muted/10">
                  <div>
                      <h2 class="font-semibold">Fraud Chronology Timeline</h2>
                      <p class="text-xs text-muted-foreground mt-0.5">Chronological record of false billings or certifications.</p>
                  </div>
                  <button onclick={() => showAddEventModal = true} class="bg-indigo-600 text-white px-4 py-2 rounded text-sm hover:bg-indigo-700 flex items-center gap-2 font-medium">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                      Log Event
                  </button>
              </div>

              {#if showAddEventModal}
                  <div class="p-6 border-b bg-muted/5 space-y-4">
                      <h3 class="text-sm font-semibold uppercase text-indigo-400">Add Log Record</h3>
                      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                          <input type="text" bind:value={newEventTitle} placeholder="Event Title (e.g. Falsified Invoice #401)" class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                          <input type="date" bind:value={newEventDate} class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                          <select bind:value={newEventType} class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary">
                              <option value="billing">Billing Fraud</option>
                              <option value="certification">False Certification</option>
                              <option value="instruction">Internal Directive</option>
                              <option value="other">Other</option>
                          </select>
                      </div>
                      <textarea bind:value={newEventDesc} placeholder="Provide details (witnesses, documents referencing this incident, programs affected)..." class="w-full h-20 bg-background border rounded p-2 text-sm resize-none focus:outline-none focus:ring-1 focus:ring-primary"></textarea>
                      <div class="flex justify-end gap-2">
                          <button onclick={() => showAddEventModal = false} class="text-xs bg-muted px-3 py-1.5 rounded text-muted-foreground">Cancel</button>
                          <button onclick={addTimelineEvent} class="text-xs bg-primary text-primary-foreground px-4 py-1.5 rounded font-medium hover:bg-primary/95">Save Event</button>
                      </div>
                  </div>
              {/if}

              <div class="p-0">
                  {#if events.length === 0}
                      <div class="p-12 text-center text-muted-foreground">
                          <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mx-auto text-muted-foreground/30 mb-3"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
                          <p>No timeline events logged.</p>
                          <p class="text-xs opacity-70 mt-1">Log timeline occurrences to build chronological proof.</p>
                      </div>
                  {:else}
                      <div class="relative pl-6 border-l-2 border-indigo-500/20 m-6 space-y-6">
                          {#each events as ev}
                              <div class="relative">
                                  <div class="absolute -left-[31px] top-1 h-4 w-4 rounded-full border-2 border-indigo-500 bg-background flex items-center justify-center">
                                      <div class="h-1.5 w-1.5 bg-indigo-500 rounded-full"></div>
                                  </div>
                                  <div class="bg-muted/10 border border-border p-4 rounded-lg hover:border-indigo-500/30 transition-all flex justify-between items-start">
                                      <div class="space-y-1">
                                          <div class="flex items-center gap-2">
                                              <span class="text-xs font-mono bg-indigo-500/10 text-indigo-400 px-2 py-0.5 rounded border border-indigo-500/20">{ev.event_date}</span>
                                              <strong class="text-sm font-semibold">{ev.title}</strong>
                                              <span class="text-[10px] uppercase font-mono text-muted-foreground px-1 bg-muted rounded">{ev.event_type}</span>
                                          </div>
                                          <p class="text-xs text-muted-foreground">{ev.description || "_No details added._"}</p>
                                      </div>
                                      <button onclick={() => deleteEvent(ev.id)} class="text-muted-foreground hover:text-red-400 p-1 transition-colors" aria-label="Delete event">
                                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                      </button>
                                  </div>
                              </div>
                          {/each}
                      </div>
                  {/if}
              </div>
          </div>

      <!-- EVIDENCE TAB -->
      {:else if activeTab === "evidence"}
          <div class="bg-card border rounded-lg shadow-sm">
              <div class="p-4 border-b flex justify-between items-center bg-muted/10">
                  <div>
                      <h2 class="font-semibold">Evidence Locker</h2>
                      <p class="text-xs text-muted-foreground mt-0.5">Sensitive local assets encrypted via XChaCha20Poly1305.</p>
                  </div>
                  <button onclick={addEvidenceFile} class="bg-indigo-600 text-white px-4 py-2 rounded text-sm hover:bg-indigo-700 flex items-center gap-2 font-medium">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                      Lock Exhibit file
                  </button>
              </div>

              <!-- Drag and drop zone -->
              <div 
                  class="m-6 p-8 border-2 border-dashed border-border rounded-xl text-center transition-all bg-muted/5 hover:bg-muted/10 cursor-pointer relative"
                  ondragover={(e) => { e.preventDefault(); isDragging = true; }}
                  ondragleave={() => isDragging = false}
                  ondrop={(e) => { e.preventDefault(); isDragging = false; addEvidenceFile(); }}
              >
                  {#if isDragging}
                      <div class="absolute inset-0 bg-indigo-500/10 border-indigo-500 rounded-xl flex items-center justify-center font-bold text-indigo-400">
                          Drop to encrypt file...
                      </div>
                  {/if}
                  <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mx-auto text-indigo-400 mb-3"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
                  <p class="text-sm font-semibold">Drag & Drop exhibits here or click to import</p>
                  <p class="text-[10px] text-muted-foreground/80 mt-1 uppercase tracking-wide">Files are encrypted on disk instantly</p>
              </div>

              <div class="p-6 pt-0">
                  {#if evidenceList.length === 0}
                      <p class="text-center text-xs text-muted-foreground py-6">No exhibits stored. Encrypt document captures to list them in report.</p>
                  {:else}
                      <ul class="border rounded divide-y bg-background/50">
                          {#each evidenceList as file}
                              <li class="p-3.5 flex items-center justify-between hover:bg-muted/30">
                                  <div class="flex items-center gap-3">
                                      <div class="h-8 w-8 bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 rounded flex items-center justify-center">
                                          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
                                      </div>
                                      <div>
                                          <p class="font-medium text-xs text-foreground">{file.file_path}</p>
                                          <p class="text-[10px] text-muted-foreground font-mono">XChaCha20Poly1305 Locked • {file.description}</p>
                                          {#if file.sha256_hash}
                                              <p class="text-[9px] text-muted-foreground/60 font-mono mt-0.5">SHA-256: {file.sha256_hash}</p>
                                          {/if}
                                      </div>
                                  </div>
                                  <div class="flex items-center gap-2">
                                      <span class="text-[10px] text-green-500 border border-green-500/20 bg-green-500/10 px-2 py-0.5 rounded font-mono">CHAIN SECURE</span>
                                      <button onclick={() => deleteEvidence(file.id)} class="text-xs text-destructive hover:text-red-400 p-1.5 hover:bg-destructive/10 rounded transition-colors" aria-label="Delete Exhibit">
                                          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
                                      </button>
                                  </div>
                              </li>
                          {/each}
                      </ul>
                  {/if}
              </div>
          </div>

      <!-- PARTIES TAB -->
      {:else if activeTab === "parties"}
          <div class="bg-card border rounded-lg shadow-sm">
              <div class="p-4 border-b flex justify-between items-center bg-muted/10">
                  <div>
                      <h2 class="font-semibold">Witness Directory & Parties</h2>
                      <p class="text-xs text-muted-foreground mt-0.5">Manage contacts, key organization employees, or federal program auditors.</p>
                  </div>
                  <button onclick={() => showAddPartyModal = true} class="bg-indigo-600 text-white px-4 py-2 rounded text-sm hover:bg-indigo-700 flex items-center gap-2 font-medium">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
                      Add Person
                  </button>
              </div>

              {#if showAddPartyModal}
                  <div class="p-6 border-b bg-muted/5 space-y-4">
                      <h3 class="text-sm font-semibold uppercase text-indigo-400">Register Party</h3>
                      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                          <input type="text" bind:value={newPartyName} placeholder="Full Name" class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                          <select bind:value={newPartyRole} class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary">
                              <option value="Witness">Witness / Auditor</option>
                              <option value="Defendant Employee">Target Co. Employee</option>
                              <option value="Relator">Relator / Whistleblower</option>
                              <option value="Government Counsel">Federal Attorney</option>
                          </select>
                          <input type="email" bind:value={newPartyEmail} placeholder="Email (optional)" class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                      </div>
                      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <input type="text" bind:value={newPartyPhone} placeholder="Phone Number" class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                          <input type="text" bind:value={newPartyNotes} placeholder="Brief Notes (e.g. has billing system logs)" class="w-full bg-background border rounded px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary" />
                      </div>
                      <div class="flex justify-end gap-2">
                          <button onclick={() => showAddPartyModal = false} class="text-xs bg-muted px-3 py-1.5 rounded text-muted-foreground">Cancel</button>
                          <button onclick={addPartyRecord} class="text-xs bg-primary text-primary-foreground px-4 py-1.5 rounded font-medium hover:bg-primary/95">Save Party</button>
                      </div>
                  </div>
              {/if}

              <div class="p-6">
                  {#if parties.length === 0}
                      <p class="text-center text-xs text-muted-foreground py-6">No individuals logged. Add co-conspirators or witnesses to organize details.</p>
                  {:else}
                      <div class="grid gap-4 md:grid-cols-2">
                          {#each parties as p}
                              <div class="p-4 bg-background border border-border rounded-lg flex justify-between items-start">
                                  <div class="space-y-1">
                                      <div class="flex items-center gap-2">
                                          <strong class="text-sm font-semibold">{p.name}</strong>
                                          <span class="text-[10px] bg-indigo-500/10 text-indigo-400 border border-indigo-500/10 px-1.5 py-0.5 rounded font-mono">{p.role}</span>
                                      </div>
                                      {#if p.email || p.phone}
                                          <p class="text-xs text-muted-foreground font-mono">{p.email || ""} {p.phone || ""}</p>
                                      {/if}
                                      <p class="text-xs text-muted-foreground italic">{p.notes || "_No notes logged._"}</p>
                                  </div>
                                  <button onclick={() => deletePartyRecord(p.id)} class="text-muted-foreground hover:text-red-400 p-1" aria-label="Delete witness">
                                      <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
                                  </button>
                              </div>
                          {/each}
                      </div>
                  {/if}
              </div>
          </div>

      <!-- PLEADING BUILDER TAB -->
      {:else if activeTab === "pleading"}
          <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
              <!-- Sidebar Section selector -->
              <div class="space-y-2 col-span-1">
                  <h3 class="text-xs font-semibold uppercase text-muted-foreground px-2 mb-2 tracking-wider">Complaint Sections</h3>
                  {#each [
                      { id: "introduction", label: "I. Introduction" },
                      { id: "jurisdiction", label: "II. Jurisdiction & Venue" },
                      { id: "parties", label: "III. Parties Involved" },
                      { id: "facts", label: "IV. Statement of Facts" },
                      { id: "violations", label: "V. Legal Violations" }
                  ] as sec}
                      <button 
                          onclick={() => activeSection = sec.id}
                          class="w-full text-left text-xs py-2 px-3 rounded font-medium border border-transparent transition-all {activeSection === sec.id ? 'bg-indigo-500/10 text-indigo-400 border-indigo-500/20 font-bold' : 'hover:bg-muted text-muted-foreground'}"
                      >
                          {sec.label}
                      </button>
                  {/each}
              </div>

              <!-- Main drafting pane -->
              <div class="md:col-span-3 bg-card border rounded-lg p-6 shadow-sm space-y-4">
                  <div class="flex justify-between items-center border-b pb-3">
                      <div>
                          <h2 class="text-lg font-bold">Drafting: {activeSection.toUpperCase()}</h2>
                          <p class="text-xs text-muted-foreground">Draft your sealed False Claims Act Complaint sections locally.</p>
                      </div>
                      <button onclick={() => saveComplaintSection(activeSection)} class="bg-indigo-600 text-white px-4 py-1.5 rounded text-xs hover:bg-indigo-700 font-medium">Save Section</button>
                  </div>

                  <div class="space-y-4">
                      {#if activeSection === "introduction"}
                          <div class="bg-muted/20 border p-3.5 rounded text-xs text-muted-foreground space-y-1">
                              <strong>Drafting Guidance (Introduction):</strong>
                              <p>Identify the Relator (Plaintiff) bringing the action on behalf of the United States. State that the action is brought under 31 U.S.C. § 3729 to recover government money lost to fraudulent billing.</p>
                          </div>
                      {:else if activeSection === "jurisdiction"}
                          <div class="bg-muted/20 border p-3.5 rounded text-xs text-muted-foreground space-y-1">
                              <strong>Drafting Guidance (Jurisdiction):</strong>
                              <p>Establish standing. Cite federal question jurisdiction (28 U.S.C. § 1331) and False Claims Act venue guidelines (31 U.S.C. § 3732(a) - where the defendant resides or transact business).</p>
                          </div>
                      {:else if activeSection === "parties"}
                          <div class="bg-muted/20 border p-3.5 rounded text-xs text-muted-foreground space-y-1">
                              <strong>Drafting Guidance (Parties):</strong>
                              <p>State the names, locations, and roles of the Relator and the Target Co. (Defendants) who received federal program funding.</p>
                          </div>
                      {:else if activeSection === "facts"}
                          <div class="bg-muted/20 border p-3.5 rounded text-xs text-muted-foreground space-y-1">
                              <strong>Drafting Guidance (Statement of Facts):</strong>
                              <p>Be specific. Detail the fraudulent scheme, what billing records were falsified, who was involved, and which federal grants/contracts were overbilled.</p>
                          </div>
                      {:else if activeSection === "violations"}
                          <div class="bg-muted/20 border p-3.5 rounded text-xs text-muted-foreground space-y-1">
                              <strong>Drafting Guidance (Claims for Relief):</strong>
                              <p>Enumerate the counts. Cite specific subsections violated (e.g. 31 U.S.C. § 3729(a)(1)(A) - presenting false claims, or (B) - making false records material to a false claim).</p>
                          </div>
                      {/if}

                      <textarea 
                          class="w-full h-80 bg-background border border-border rounded p-3 text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500 font-mono"
                          bind:value={complaintSections[activeSection]}
                          placeholder="Type factual allegations..."
                      ></textarea>
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
     onCancel={handleWizardComplete}
   />
{/if}
