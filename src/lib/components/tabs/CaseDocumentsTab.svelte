<script lang="ts">
  import FieldHelp from "$lib/components/FieldHelp.svelte";
  import type {
    CaseProfile,
    GuidedDraft,
    CaseArtifact,
    FormInspection,
    FormFillResult,
  } from "$lib/types/case";

  interface Props {
    profile: CaseProfile;
    guided: GuidedDraft | null;
    artifacts: CaseArtifact[];
    busy: boolean;
    mdDraft: string;
    selectedFormPath: string;
    formInspection: FormInspection | null;
    formFillResult: FormFillResult | null;
    fillingForm: boolean;
    onfillsection: (id: string, text: string) => void;
    onexportkind: (kind: string) => void;
    onexportpacket: () => void;
    onimportmarkdown: () => void;
    oninspectform: (path: string) => void;
    onfillform: (path: string) => void;
  }

  let {
    profile = $bindable(),
    guided,
    artifacts,
    busy,
    mdDraft = $bindable(),
    selectedFormPath = $bindable(),
    formInspection = $bindable(),
    formFillResult = $bindable(),
    fillingForm,
    onfillsection,
    onexportkind,
    onexportpacket,
    onimportmarkdown,
    oninspectform,
    onfillform,
  }: Props = $props();
</script>

<div class="space-y-6">
  {#if guided}
    <section class="space-y-4 bg-card border rounded-lg p-6">
      <h2 class="font-semibold">Write this paper (guided)</h2>
      <p class="text-sm text-muted-foreground">Answer each prompt in your own words. Leave a box blank rather than guessing. The app will not invent facts or citations.</p>
      {#each guided.sections as section}
        <label class="block space-y-1">
          <span class="text-sm font-medium">{section.heading} <FieldHelp text={section.hint} /></span>
          <p class="text-xs text-muted-foreground">{section.hint}</p>
          <textarea
            class="w-full h-24 rounded-md border border-input bg-background px-3 py-2 text-sm"
            value={section.user_text}
            onblur={(e) => onfillsection(section.id, (e.currentTarget as HTMLTextAreaElement).value)}
          ></textarea>
        </label>
      {/each}
      <div class="flex flex-wrap gap-2">
        <button onclick={() => onexportkind(profile?.document_kind || "motion")} disabled={busy} class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm">Export this paper (Word and PDF)</button>
        <button onclick={onexportpacket} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export filing packet</button>
      </div>
    </section>
  {/if}

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <h2 class="font-semibold">Notice of Appeal</h2>
    <p class="text-xs text-muted-foreground">OpenSeason will not invent grounds, facts, or citations. Paste only text you wrote or researched.</p>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Trial court</span>
      <input bind:value={profile.notice_of_appeal.trial_court_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
    <div class="grid md:grid-cols-2 gap-3">
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Trial court docket</span>
        <input bind:value={profile.notice_of_appeal.trial_court_docket} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
      <label class="block space-y-1">
        <span class="text-xs text-muted-foreground">Judgment or order date</span>
        <input type="date" bind:value={profile.notice_of_appeal.judgment_date} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
    </div>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">What order or judgment is being appealed?</span>
      <input bind:value={profile.notice_of_appeal.judgment_description} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
    </label>
    <label class="block space-y-1">
      <span class="text-xs text-muted-foreground">Your notice text</span>
      <textarea bind:value={profile.notice_of_appeal.user_text} class="w-full h-36 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
    </label>
    <button onclick={() => onexportkind("notice_of_appeal")} disabled={busy} class="bg-indigo-600 text-white px-4 py-2 rounded-md text-sm">Export Notice of Appeal (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Complaint</h2>
      <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
    </div>
    <p class="text-xs text-muted-foreground">Caption, parties, and the signature block come from the profile. You write the claim. This is not a full complaint interview.</p>
    <input bind:value={profile.complaint.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Complaint title" />
    <textarea bind:value={profile.complaint.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your statement of the claim"></textarea>
    <button onclick={() => onexportkind("complaint")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export complaint (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Motion</h2>
      <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
    </div>
    <p class="text-xs text-muted-foreground">Same fill-once profile. You write the request. This is not a motion library.</p>
    <input bind:value={profile.motion.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Motion title" />
    <textarea bind:value={profile.motion.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your motion text"></textarea>
    <button onclick={() => onexportkind("motion")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export motion (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Appellate brief</h2>
      <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic scaffold</span>
    </div>
    <input bind:value={profile.brief.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Brief title" />
    <textarea bind:value={profile.brief.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your brief text. The scaffold will not invent argument."></textarea>
    <button onclick={() => onexportkind("appellate_brief")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export brief (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Motion in the appellate court</h2>
      <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic template</span>
    </div>
    <input bind:value={profile.appellate_motion.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Appellate motion title" />
    <textarea bind:value={profile.appellate_motion.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your R. 2:8-1 motion text"></textarea>
    <button onclick={() => onexportkind("appellate_motion")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export appellate motion (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Mandamus (Colorado)</h2>
      <span class="text-[10px] uppercase tracking-wide text-amber-300">Basic C.A.R. 21</span>
    </div>
    <input bind:value={profile.mandamus.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Petition title" />
    <textarea bind:value={profile.mandamus.user_text} class="w-full h-32 rounded-md border border-input bg-background px-3 py-2 text-sm" placeholder="Your petition text"></textarea>
    <button onclick={() => onexportkind("mandamus")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export mandamus (Word and PDF)</button>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between">
      <h2 class="font-semibold">Court Markdown</h2>
      <span class="text-[10px] uppercase tracking-wide text-muted-foreground">Same engine as the CLI</span>
    </div>
    <p class="text-xs text-muted-foreground">Paste an md2court subset draft (YAML front matter, ::: signature, ::: certificate). Import writes drafts/draft.md. Export uses the same renderer as `openseason export court-md`.</p>
    <textarea bind:value={mdDraft} class="w-full h-36 rounded-md border border-input bg-background px-3 py-2 text-sm font-mono" placeholder={"---\ncourt: ...\nplaintiff: ...\ndefendant: ...\ncase_no: ...\n---\n"}></textarea>
    <div class="flex flex-wrap gap-2">
      <button onclick={onimportmarkdown} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Import Markdown</button>
      <button onclick={() => onexportkind("court_md")} disabled={busy} class="border border-border px-4 py-2 rounded-md text-sm">Export court Markdown (Word and PDF)</button>
    </div>
  </section>

  <section class="space-y-4 bg-card border rounded-lg p-6">
    <div class="flex justify-between items-center">
      <div>
        <h2 class="font-semibold">Official Court Forms (AcroForms)</h2>
        <p class="text-xs text-muted-foreground">Inspect official court PDF forms, auto-map case profile data into form fields, and export filled PDFs.</p>
      </div>
    </div>

    {#if artifacts.filter((a) => a.kind === "form").length > 0}
      <div class="space-y-2">
        <span class="text-xs font-medium text-muted-foreground">Available Court Forms in Case:</span>
        <div class="grid gap-2">
          {#each artifacts.filter((a) => a.kind === "form") as form}
            <div class="flex items-center justify-between border rounded-md p-3 bg-muted/10">
              <div>
                <p class="text-sm font-semibold">{form.name}</p>
                <p class="text-xs font-mono text-muted-foreground">{form.path}</p>
              </div>
              <div class="flex gap-2">
                <button
                  onclick={() => oninspectform(form.path)}
                  class="text-xs border border-border px-3 py-1.5 rounded-md hover:bg-muted"
                >
                  Inspect Fields
                </button>
                <button
                  onclick={() => onfillform(form.path)}
                  disabled={fillingForm}
                  class="text-xs bg-indigo-600 hover:bg-indigo-700 text-white px-3 py-1.5 rounded-md font-medium disabled:opacity-50"
                >
                  {fillingForm && selectedFormPath === form.path ? "Filling..." : "Fill with Case Profile"}
                </button>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <p class="text-xs text-muted-foreground border border-dashed rounded-md p-3">
        No forms found in case `forms/` directory. Place court-issued PDFs (e.g. JDF 205, JDF 1911) into `forms/` or inspect by path below.
      </p>
    {/if}

    <div class="pt-2 border-t border-border space-y-2">
      <span class="text-xs text-muted-foreground">Custom PDF Form Path:</span>
      <div class="flex gap-2">
        <input
          bind:value={selectedFormPath}
          placeholder="C:\path\to\court-form.pdf"
          class="flex-1 h-9 rounded-md border border-input bg-background px-3 text-xs font-mono"
        />
        <button
          onclick={() => oninspectform(selectedFormPath)}
          disabled={!selectedFormPath.trim()}
          class="text-xs border border-border px-3 py-1.5 rounded-md hover:bg-muted disabled:opacity-50"
        >
          Inspect
        </button>
        <button
          onclick={() => onfillform(selectedFormPath)}
          disabled={!selectedFormPath.trim() || fillingForm}
          class="text-xs bg-indigo-600 hover:bg-indigo-700 text-white px-3 py-1.5 rounded-md font-medium disabled:opacity-50"
        >
          Fill Form
        </button>
      </div>
    </div>

    {#if formInspection}
      <div class="mt-4 p-4 border rounded-md bg-muted/20 space-y-2">
        <div class="flex justify-between items-center">
          <p class="text-xs font-semibold">{formInspection.detected_form_number || formInspection.form_name} — {formInspection.fields.length} Fields</p>
          <button onclick={() => formInspection = null} class="text-[10px] text-muted-foreground hover:text-foreground">Close</button>
        </div>
        <div class="max-h-48 overflow-y-auto space-y-1">
          {#each formInspection.fields as f}
            <div class="text-xs font-mono flex justify-between border-b border-border/40 py-1">
              <span class="text-foreground">{f.key} ({f.label})</span>
              <span class="text-muted-foreground text-[10px]">{f.field_type} {f.required ? '(required)' : ''}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    {#if formFillResult}
      <div class="mt-4 p-4 border border-emerald-500/30 bg-emerald-500/10 rounded-md space-y-2">
        <div class="flex justify-between items-center">
          <p class="text-sm font-semibold text-emerald-400">Form Filled ({formFillResult.mapped_fields} fields mapped)</p>
          <button onclick={() => formFillResult = null} class="text-[10px] text-muted-foreground hover:text-foreground">Dismiss</button>
        </div>
        <div class="text-xs font-mono space-y-1">
          <p><span class="text-muted-foreground">Filled PDF:</span> {formFillResult.output_pdf_path}</p>
          <p><span class="text-muted-foreground">Field Data JSON:</span> {formFillResult.output_json_path}</p>
        </div>
      </div>
    {/if}
  </section>
</div>
