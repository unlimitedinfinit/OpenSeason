<script lang="ts">
  import { ipc } from "$lib/ipc";
  import type { CaseProfile } from "$lib/types/case";
  import type { AccommodationCategory, AdaRequestPayload, AdaPetitionReport } from "$lib/types/record";

  interface Props {
    open: boolean;
    caseId: string;
    profile?: CaseProfile | null;
    onclose: () => void;
  }

  let { open = $bindable(false), caseId, profile = null, onclose }: Props = $props();

  let requesterName = $state("");
  let requesterRole = $state("Plaintiff / Appellant (Self-Represented)");
  let contactPhone = $state("");
  let contactEmail = $state("");
  let upcomingDate = $state("");
  let specificNeeds = $state("");
  let selectedAccommodations = $state<AccommodationCategory[]>([
    "remote_telephonic_appearance",
    "extended_reading_time",
  ]);

  let generating = $state(false);
  let petitionReport = $state<AdaPetitionReport | null>(null);
  let copied = $state(false);
  let errorMessage = $state("");

  const ACCOMMODATION_OPTIONS: { id: AccommodationCategory; label: string; desc: string }[] = [
    {
      id: "remote_telephonic_appearance",
      label: "Remote Video / Telephonic Appearance",
      desc: "Appearance via Zoom/Teams or telephone due to medical, cognitive, or physical limitations.",
    },
    {
      id: "extended_reading_time",
      label: "Extended Time & Frequent Breaks",
      desc: "Additional time to review filings during court sessions and scheduled recess breaks.",
    },
    {
      id: "visual_hearing_assistance",
      label: "Assistive Listening & Real-Time Captioning (CART)",
      desc: "Audio amplification devices or certified real-time transcription display.",
    },
    {
      id: "physical_mobility_access",
      label: "Barrier-Free Physical Mobility Access",
      desc: "Wheelchair-accessible courtroom, first-floor assignment, or proximity seating.",
    },
    {
      id: "support_person_accompaniment",
      label: "Support Person / Caregiver Accompaniment",
      desc: "Permission for non-attorney support person or aide to sit at counsel table.",
    },
    {
      id: "document_magnification_accessible_pdf",
      label: "Accessible Digital PDF / Large Print",
      desc: "Electronic court orders and filings provided in text-searchable accessible format.",
    },
    {
      id: "communication_interpreter",
      label: "Sign Language or Specialized Interpreter",
      desc: "Certified ASL or tactile interpreter appointed by the judiciary.",
    },
  ];

  function toggleAccommodation(id: AccommodationCategory) {
    if (selectedAccommodations.includes(id)) {
      selectedAccommodations = selectedAccommodations.filter((a) => a !== id);
    } else {
      selectedAccommodations = [...selectedAccommodations, id];
    }
  }

  async function generatePetition() {
    if (!caseId || !requesterName.trim()) {
      errorMessage = "Requester name is required.";
      return;
    }
    if (selectedAccommodations.length === 0) {
      errorMessage = "Please select at least one accommodation.";
      return;
    }

    generating = true;
    errorMessage = "";
    try {
      const payload: AdaRequestPayload = {
        case_id: caseId,
        requester_name: requesterName.trim(),
        requester_role: requesterRole.trim(),
        contact_phone: contactPhone.trim(),
        contact_email: contactEmail.trim(),
        accommodations: selectedAccommodations,
        specific_needs_narrative: specificNeeds.trim(),
        upcoming_date: upcomingDate.trim() || null,
      };
      petitionReport = await ipc<AdaPetitionReport>("ada_petition_cmd", { payload });
    } catch (e: any) {
      errorMessage = typeof e === "string" ? e : e?.message || "Failed to generate ADA petition.";
    } finally {
      generating = false;
    }
  }

  function copyText() {
    if (petitionReport?.rendered_petition_text && navigator?.clipboard) {
      navigator.clipboard.writeText(petitionReport.rendered_petition_text);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    }
  }

  $effect(() => {
    if (open && profile) {
      if (!requesterName && profile.filer?.name) requesterName = profile.filer.name;
      if (!contactPhone && profile.filer?.phone) contactPhone = profile.filer.phone;
      if (!contactEmail && profile.filer?.email) contactEmail = profile.filer.email;
    }
  });
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-stone-950/60 backdrop-blur-xs overflow-y-auto">
    <div class="relative w-full max-w-2xl my-8 rounded-2xl border border-stone-200 dark:border-stone-800 bg-white dark:bg-stone-900 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Header -->
      <div class="p-5 border-b border-stone-100 dark:border-stone-800 flex items-center justify-between gap-3 bg-stone-50/70 dark:bg-stone-950/40">
        <div>
          <div class="flex items-center gap-2">
            <span class="inline-flex items-center justify-center w-6 h-6 rounded-md bg-teal-500/10 text-teal-600 dark:text-teal-400 font-bold text-xs">
              ADA
            </span>
            <h3 class="text-base font-semibold text-stone-900 dark:text-stone-100">
              Title II Equal Access & ADA Accommodation Petition
            </h3>
          </div>
          <p class="text-xs text-stone-500 dark:text-stone-400 mt-0.5">
            42 U.S.C. § 12132 · 28 C.F.R. Part 35 · NJ Directive #01-19 · CO CJD 06-01
          </p>
        </div>
        <button
          onclick={onclose}
          class="p-1 rounded-lg text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 hover:bg-stone-100 dark:hover:bg-stone-800 transition"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Confidentiality Alert -->
      <div class="px-5 py-3 bg-primary/10 border-b border-primary/20 text-xs flex items-start gap-2.5">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-primary shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-[11px] text-foreground/90 leading-relaxed">
          <strong>Confidential Administrative Submission:</strong> ADA accommodation petitions are transmitted directly to the Court ADA Coordinator / Trial Court Administrator and are exempt from public filing on the general docket.
        </p>
      </div>

      <!-- Form Body -->
      <div class="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
        {#if errorMessage}
          <div class="p-3 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-700 dark:text-rose-400">
            {errorMessage}
          </div>
        {/if}

        <!-- Requester Fields Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <label for="ada-req-name" class="font-semibold text-stone-700 dark:text-stone-300">Requester Legal Name *</label>
            <input
              id="ada-req-name"
              bind:value={requesterName}
              placeholder="e.g. Jordan Example"
              class="w-full px-3 py-1.5 rounded-lg border border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
            />
          </div>
          <div class="space-y-1">
            <label for="ada-req-role" class="font-semibold text-stone-700 dark:text-stone-300">Case Role / Status</label>
            <input
              id="ada-req-role"
              bind:value={requesterRole}
              placeholder="e.g. Appellant (Self-Represented)"
              class="w-full px-3 py-1.5 rounded-lg border border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
            />
          </div>
          <div class="space-y-1">
            <label for="ada-contact-phone" class="font-semibold text-stone-700 dark:text-stone-300">Contact Telephone</label>
            <input
              id="ada-contact-phone"
              bind:value={contactPhone}
              placeholder="e.g. (555) 010-0100"
              class="w-full px-3 py-1.5 rounded-lg border border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
            />
          </div>
          <div class="space-y-1">
            <label for="ada-upcoming-date" class="font-semibold text-stone-700 dark:text-stone-300">Upcoming Hearing / Oral Argument Date</label>
            <input
              id="ada-upcoming-date"
              type="date"
              bind:value={upcomingDate}
              class="w-full px-3 py-1.5 rounded-lg border border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
            />
          </div>
        </div>

        <!-- Accommodations Checkbox Grid -->
        <div class="space-y-2">
          <label class="font-semibold text-stone-700 dark:text-stone-300 block">Requested Accommodation Categories *</label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {#each ACCOMMODATION_OPTIONS as opt}
              {@const checked = selectedAccommodations.includes(opt.id)}
              <button
                type="button"
                onclick={() => toggleAccommodation(opt.id)}
                class="text-left p-2.5 rounded-lg border transition {checked ? 'border-teal-500 bg-teal-50/50 dark:bg-teal-950/20' : 'border-stone-200 dark:border-stone-800 hover:border-stone-300'}"
              >
                <div class="flex items-center gap-2">
                  <div class="w-4 h-4 rounded border flex items-center justify-center {checked ? 'bg-teal-600 border-teal-600 text-white' : 'border-stone-300 dark:border-stone-600'}">
                    {#if checked}
                      <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                      </svg>
                    {/if}
                  </div>
                  <span class="font-semibold text-stone-900 dark:text-stone-100 text-[11px]">{opt.label}</span>
                </div>
                <p class="text-[10px] text-stone-500 dark:text-stone-400 mt-1 pl-6">{opt.desc}</p>
              </button>
            {/each}
          </div>
        </div>

        <!-- Needs Narrative -->
        <div class="space-y-1">
          <label for="ada-specific-needs" class="font-semibold text-stone-700 dark:text-stone-300">Specific Functional Limitations / Notes (Optional)</label>
          <textarea
            id="ada-specific-needs"
            bind:value={specificNeeds}
            rows="2"
            placeholder="Detail any specifics (e.g. difficulty sitting for more than 45 minutes; need audio transcription due to bilateral hearing impairment)..."
            class="w-full p-2.5 rounded-lg border border-stone-200 dark:border-stone-700 bg-white dark:bg-stone-900 text-stone-800 dark:text-stone-200 focus:outline-none focus:ring-1 focus:ring-teal-500"
          ></textarea>
        </div>

        <!-- Generated Result -->
        {#if petitionReport}
          <div class="p-3.5 rounded-xl border border-teal-300 dark:border-teal-800 bg-teal-50/40 dark:bg-teal-950/30 space-y-2">
            <div class="flex items-center justify-between">
              <span class="font-bold text-teal-900 dark:text-teal-200">
                Petition Form Ready for {petitionReport.ada_coordinator_title}
              </span>
              <button
                onclick={copyText}
                class="px-2.5 py-1 text-xs rounded bg-teal-600 hover:bg-teal-700 text-white font-medium transition"
              >
                {copied ? "Copied Petition!" : "Copy Petition"}
              </button>
            </div>
            <p class="text-[11px] text-stone-600 dark:text-stone-300">{petitionReport.submission_guidelines}</p>
            <pre class="p-3 rounded bg-white dark:bg-stone-900 border border-teal-200 dark:border-teal-800 text-[11px] font-mono text-stone-800 dark:text-stone-200 overflow-x-auto max-h-40 whitespace-pre-wrap">{petitionReport.rendered_petition_text}</pre>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-stone-100 dark:border-stone-800 bg-stone-50/50 dark:bg-stone-950/40 flex items-center justify-between">
        <button
          onclick={onclose}
          class="px-3.5 py-1.5 text-xs font-medium rounded-lg border border-stone-300 dark:border-stone-700 text-stone-700 dark:text-stone-300 hover:bg-stone-100"
        >
          Close
        </button>
        <button
          onclick={generatePetition}
          disabled={generating}
          class="px-4 py-1.5 text-xs font-medium rounded-lg bg-teal-600 hover:bg-teal-700 text-white transition disabled:opacity-50"
        >
          {generating ? "Generating..." : "Generate Title II Petition"}
        </button>
      </div>
    </div>
  </div>
{/if}
