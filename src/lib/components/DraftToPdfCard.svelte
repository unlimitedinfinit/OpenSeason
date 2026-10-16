<script lang="ts">
  import { ipc } from "$lib/ipc";
  import type { CourtPreview, ExportResult } from "$lib/types/case";

  let {
    caseId = null,
    markdown = $bindable(""),
  }: {
    caseId?: string | null;
    markdown?: string;
  } = $props();

  let preview = $state<CourtPreview | null>(null);
  let exported = $state<ExportResult | null>(null);
  let busy = $state(false);
  let error = $state("");
  let status = $state("");

  const NJ_SAMPLE = `---
court: Superior Court of New Jersey, Appellate Division
court_address: Richard J. Hughes Justice Complex, 25 W. Market Street | P.O. Box 006, Trenton, NJ 08625-0006
plaintiff: Jordan Example
plaintiff_label: Appellant
defendant: Sample County Clerk
defendant_label: Respondent
case_no: A-01000-24
trial_court: Superior Court of New Jersey, Law Division, Sample County
trial_docket: L-01000-24
doc_title: Motion for reconsideration
governing_state: NJ
caption_layout: boxed
filer_email: jordan.example@example.test
filer_phone: 555-0100
filer_address: 100 Sample Street | Example City, ST 00000
---

# Preliminary statement

This is a fake draft for tests. The trial court entered an order dismissing the complaint.

Relief is sought under R. 2:11-6. Time may be enlarged under R. 1:3-4(a). A timely motion may toll the appeal period under R. 2:4-3(e).

The user, not the software, must pin each fact to a record exhibit before sign-off.

::: signature
/s/ Jordan Example
Jordan Example
Appellant, proceeding without an attorney
100 Sample Street
Example City, ST 00000
jordan.example@example.test
555-0100
:::

::: certificate
I will serve a copy of this paper myself. This software did not serve anyone.
Recipient: Sample County Clerk
Method: electronic mail
Date: October 1, 2026
:::
`;

  const CO_SAMPLE = `---
court: Colorado Court of Appeals
court_address: 2 East 14th Avenue | Denver, CO 80203
plaintiff: Jordan Example
plaintiff_label: Appellant
defendant: Sample County Clerk
defendant_label: Appellee
case_no: 24CA1000
trial_court: Denver District Court
trial_docket: 23CV1000
doc_title: Motion for reconsideration
governing_state: CO
caption_layout: boxed
filer_email: jordan.example@example.test
filer_phone: 555-0100
filer_address: 100 Sample Street | Example City, ST 00000
conferral_citation: C.R.C.P. 121 § 1-15(8)
conferral_status: needs_verification
declaration_citation: C.R.S. § 13-27-106
declaration_status: needs_verification
---

# Conferral

I conferred with the opposing party on October 1, 2026. The opposing party takes no position.

# Preliminary statement

This is a fake Colorado draft for tests. The Denver District Court entered an order dismissing the complaint. Relief is sought under C.A.R. 40. The user, not the software, must pin each fact to a record exhibit before sign-off.

::: declaration
I declare under penalty of perjury under the law of Colorado that the foregoing is true and correct. I signed this on October 2, 2026, at Example City, ST.
:::

::: signature
/s/ Jordan Example
Jordan Example
Appellant, proceeding without an attorney
100 Sample Street
Example City, ST 00000
jordan.example@example.test
555-0100
:::

::: certificate
I will serve a copy of this paper myself. This software did not serve anyone.
Recipient: Sample County Clerk
Method: electronic mail
Date: October 1, 2026
:::
`;

  async function doPreview() {
    error = "";
    status = "";
    exported = null;
    if (!markdown.trim()) {
      error = "Paste your Markdown draft first. OpenSeason will not invent the body.";
      return;
    }
    busy = true;
    try {
      preview = await ipc<CourtPreview>("preview_markdown_cmd", {
        markdown,
        caseId: caseId || null,
      });
      status = "Preview only. Nothing was filed. Check the caption, then export.";
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function doExport() {
    error = "";
    if (!markdown.trim()) {
      error = "Paste your Markdown draft first.";
      return;
    }
    busy = true;
    try {
      exported = await ipc<ExportResult>("export_markdown_draft_cmd", {
        markdown,
        caseId: caseId || null,
      });
      status = `Wrote Word and PDF. You still file and serve.\n${exported.pdf}`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function onFile(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      markdown = String(reader.result || "");
      preview = null;
      exported = null;
    };
    reader.readAsText(file);
  }
</script>

<section class="space-y-4 bg-card border border-border rounded-lg p-6" aria-labelledby="draft-to-pdf-heading">
  <div>
    <p class="text-xs uppercase tracking-widest text-muted-foreground">Three clicks</p>
    <h2 id="draft-to-pdf-heading" class="text-xl font-semibold">Turn your Markdown into a court PDF</h2>
    <p class="text-sm text-muted-foreground mt-1">
      Paste a draft you wrote. OpenSeason formats a boxed caption, signature, and certificate of service. It never files or serves.
    </p>
  </div>

  <ol class="text-sm space-y-1 list-decimal pl-5 text-muted-foreground">
    <li>Paste or choose your <code class="text-xs">.md</code> file</li>
    <li>Click <strong class="text-foreground">Preview court paper</strong></li>
    <li>Click <strong class="text-foreground">Export Word and PDF</strong></li>
  </ol>

  <div class="flex flex-wrap gap-2">
    <label class="text-xs border border-border px-3 py-1.5 rounded-md cursor-pointer">
      Choose a .md file
      <input type="file" accept=".md,text/markdown,text/plain" class="sr-only" onchange={onFile} aria-label="Choose a Markdown draft" />
    </label>
    <button type="button" class="text-xs border border-border px-3 py-1.5 rounded-md" onclick={() => (markdown = NJ_SAMPLE)}>
      NJ sample (Jordan Example)
    </button>
    <button type="button" class="text-xs border border-border px-3 py-1.5 rounded-md" onclick={() => (markdown = CO_SAMPLE)}>
      CO sample (Jordan Example)
    </button>
  </div>

  <textarea
    bind:value={markdown}
    class="w-full h-48 rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
    placeholder={"---\ncourt: ...\nplaintiff: ...\ndefendant: ...\ncase_no: ...\n---\n"}
    aria-label="Markdown draft"
  ></textarea>

  <div class="flex flex-wrap gap-2">
    <button
      type="button"
      onclick={doPreview}
      disabled={busy}
      class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm"
      aria-label="Preview court paper"
    >
      Preview court paper
    </button>
    <button
      type="button"
      onclick={doExport}
      disabled={busy}
      class="border border-border px-4 py-2 rounded-md text-sm"
      aria-label="Export Word and PDF"
    >
      Export Word and PDF
    </button>
  </div>

  {#if error}
    <p class="text-sm text-destructive" role="alert">{error}</p>
  {/if}
  {#if status}
    <p class="text-sm text-muted-foreground whitespace-pre-wrap">{status}</p>
  {/if}

  {#if preview}
    <div class="border border-border rounded-md p-4 space-y-2 bg-background" aria-label="Draft preview">
      <p class="text-xs uppercase tracking-wide text-muted-foreground">
        {preview.caption_kind}
        {#if preview.markup_stripped} · markup stripped{/if}
        {#if preview.has_signature} · signature{/if}
        {#if preview.has_certificate} · certificate of service{/if}
      </p>
      <p class="text-sm font-semibold">{preview.title}</p>
      <pre class="text-xs whitespace-pre-wrap font-serif leading-relaxed max-h-80 overflow-auto">{preview.lines.join("\n")}</pre>
    </div>
  {/if}

  {#if exported}
    <div class="border border-border rounded-md p-3 text-sm space-y-1">
      <p>PDF: <span class="font-mono text-xs">{exported.pdf}</span></p>
      <p>Word: <span class="font-mono text-xs">{exported.docx}</span></p>
      <p class="text-xs text-muted-foreground">Review every line before you sign and file. This app did not file or serve.</p>
    </div>
  {/if}
</section>
