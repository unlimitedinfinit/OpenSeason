<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseArtifact, PdfOperationResult, RedactOperationResult } from "$lib/types/case";

  interface Props {
    open?: boolean;
    caseId?: string;
    onclose: () => void;
  }

  let { open = $bindable(false), caseId = "", onclose }: Props = $props();

  type ToolMode = "merge" | "split" | "rotate" | "redact";
  let activeMode = $state<ToolMode>("merge");

  // Case Artifacts for quick picking
  let availableArtifacts = $state<CaseArtifact[]>([]);

  // Tool 1: Merge State
  let mergeInputs = $state<string[]>([]);
  let customMergePath = $state("");
  let mergeOutput = $state("");
  let mergeLoading = $state(false);
  let mergeResult = $state<PdfOperationResult | null>(null);

  // Tool 2: Split / Extract State
  let splitInput = $state("");
  let splitPages = $state("1-3");
  let splitOutput = $state("");
  let splitLoading = $state(false);
  let splitResult = $state<PdfOperationResult | null>(null);

  // Tool 3: Rotate State
  let rotateInput = $state("");
  let rotateDegrees = $state<number>(90);
  let rotatePages = $state(""); // blank means all
  let rotateOutput = $state("");
  let rotateLoading = $state(false);
  let rotateResult = $state<PdfOperationResult | null>(null);

  // Tool 4: Redact State
  let redactInput = $state("");
  let redactSsn = $state(true);
  let redactPhone = $state(true);
  let redactDob = $state(true);
  let customPatternsText = $state("");
  let redactOutput = $state("");
  let redactLoading = $state(false);
  let redactResult = $state<RedactOperationResult | null>(null);

  let errorMessage = $state("");
  let successMessage = $state("");

  onMount(loadArtifacts);

  async function loadArtifacts() {
    if (!caseId) return;
    try {
      const list = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => []);
      availableArtifacts = list.filter((a) => a.path.toLowerCase().endsWith(".pdf"));
    } catch {
      availableArtifacts = [];
    }
  }

  function addMergeFile(path: string) {
    if (!path.trim()) return;
    if (!mergeInputs.includes(path.trim())) {
      mergeInputs = [...mergeInputs, path.trim()];
    }
    customMergePath = "";
  }

  function removeMergeFile(index: number) {
    mergeInputs = mergeInputs.filter((_, i) => i !== index);
  }

  function moveMergeFile(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= mergeInputs.length) return;
    const copy = [...mergeInputs];
    const temp = copy[index];
    copy[index] = copy[target];
    copy[target] = temp;
    mergeInputs = copy;
  }

  async function runMerge() {
    if (mergeInputs.length < 2) {
      errorMessage = "Please provide at least 2 PDF files to merge.";
      return;
    }
    const out = mergeOutput.trim() || (mergeInputs[0].replace(/\.pdf$/i, "") + "-merged.pdf");
    mergeLoading = true;
    errorMessage = "";
    successMessage = "";
    mergeResult = null;
    try {
      const res = await ipc<PdfOperationResult>("pdf_merge_cmd", {
        inputPaths: mergeInputs,
        outputPath: out,
      });
      mergeResult = res;
      successMessage = `Merged ${mergeInputs.length} PDFs into a single ${res.page_count}-page court document!`;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      mergeLoading = false;
    }
  }

  function parsePageRange(spec: string): number[] {
    const pages: Set<number> = new Set();
    const parts = spec.split(/[,;\s]+/);
    for (const part of parts) {
      if (!part.trim()) continue;
      if (part.includes("-")) {
        const [startStr, endStr] = part.split("-");
        const s = parseInt(startStr, 10);
        const e = parseInt(endStr, 10);
        if (!isNaN(s) && !isNaN(e)) {
          for (let p = Math.min(s, e); p <= Math.max(s, e); p++) {
            if (p > 0) pages.add(p);
          }
        }
      } else {
        const p = parseInt(part, 10);
        if (!isNaN(p) && p > 0) pages.add(p);
      }
    }
    return Array.from(pages).sort((a, b) => a - b);
  }

  async function runSplit() {
    if (!splitInput.trim()) {
      errorMessage = "Please specify an input PDF to split.";
      return;
    }
    const pageList = parsePageRange(splitPages);
    if (pageList.length === 0) {
      errorMessage = "Please specify valid page numbers (e.g. 1-3, 5).";
      return;
    }
    const out = splitOutput.trim() || (splitInput.replace(/\.pdf$/i, "") + `-extracted-p${pageList.join("-")}.pdf`);
    splitLoading = true;
    errorMessage = "";
    successMessage = "";
    splitResult = null;
    try {
      const res = await ipc<PdfOperationResult>("pdf_split_cmd", {
        inputPath: splitInput.trim(),
        pagesToKeep: pageList,
        outputPath: out,
      });
      splitResult = res;
      successMessage = `Successfully extracted ${res.page_count} page(s) to new PDF!`;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      splitLoading = false;
    }
  }

  async function runRotate() {
    if (!rotateInput.trim()) {
      errorMessage = "Please specify an input PDF to rotate.";
      return;
    }
    const pageNums = rotatePages.trim() ? parsePageRange(rotatePages) : null;
    const out = rotateOutput.trim() || (rotateInput.replace(/\.pdf$/i, "") + `-rotated-${rotateDegrees}deg.pdf`);
    rotateLoading = true;
    errorMessage = "";
    successMessage = "";
    rotateResult = null;
    try {
      const res = await ipc<PdfOperationResult>("pdf_rotate_cmd", {
        inputPath: rotateInput.trim(),
        degrees: rotateDegrees,
        pageNumbers: pageNums,
        outputPath: out,
      });
      rotateResult = res;
      successMessage = `Rotated PDF by ${rotateDegrees}° clockwise!`;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      rotateLoading = false;
    }
  }

  async function runRedact() {
    if (!redactInput.trim()) {
      errorMessage = "Please specify an input PDF to redact.";
      return;
    }
    const patterns: string[] = [];
    if (redactSsn) {
      patterns.push(String.raw`\b\d{3}-\d{2}-\d{4}\b`);
      patterns.push(String.raw`\b\d{3}\s+\d{2}\s+\d{4}\b`);
    }
    if (redactPhone) {
      patterns.push(String.raw`\b(?:\+?1[-. ]?)?\(?\d{3}\)?[-. ]?\d{3}[-. ]?\d{4}\b`);
    }
    if (redactDob) {
      patterns.push(String.raw`\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b`);
    }
    if (customPatternsText.trim()) {
      for (const line of customPatternsText.split("\n")) {
        if (line.trim()) patterns.push(line.trim());
      }
    }

    if (patterns.length === 0) {
      errorMessage = "Please select at least one redaction type or enter custom text patterns.";
      return;
    }

    const out = redactOutput.trim() || (redactInput.replace(/\.pdf$/i, "") + "-redacted.pdf");
    redactLoading = true;
    errorMessage = "";
    successMessage = "";
    redactResult = null;
    try {
      const res = await ipc<RedactOperationResult>("pdf_redact_cmd", {
        inputPath: redactInput.trim(),
        customPatterns: patterns,
        outputPath: out,
      });
      redactResult = res;
      successMessage = `Cleanse complete! Byte-scrubbed ${res.redactions_count} sensitive identifier(s).`;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      redactLoading = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 overflow-y-auto animate-fade-in">
    <div class="w-full max-w-4xl rounded-2xl border border-zinc-700 bg-zinc-900 shadow-2xl overflow-hidden my-6">
      <!-- Modal Header -->
      <div class="bg-gradient-to-r from-blue-950/80 via-indigo-950/70 to-purple-950/80 p-5 border-b border-zinc-800 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="h-12 w-12 rounded-xl bg-blue-500/20 border border-blue-400/30 flex items-center justify-center text-2xl shadow-inner">
            🛠️
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-white tracking-tight">Offline "Zero-Leakage" PDF Power Station</h2>
              <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 flex items-center gap-1">
                <span>🔒</span> 100% Local Airgap
              </span>
            </div>
            <p class="text-xs text-zinc-300 mt-0.5">
              Pure-Rust deterministic PDF processing. Zero cloud uploads, zero telemetry, zero surveillance.
            </p>
          </div>
        </div>

        <button
          onclick={onclose}
          class="h-8 w-8 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200 flex items-center justify-center transition-colors self-end sm:self-auto"
        >
          ✕
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="grid grid-cols-2 sm:grid-cols-4 border-b border-zinc-800 bg-zinc-950/50 p-1.5 gap-1.5">
        <button
          onclick={() => { activeMode = 'merge'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeMode === 'merge' ? 'bg-blue-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>📑</span> Merge Files
        </button>
        <button
          onclick={() => { activeMode = 'split'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeMode === 'split' ? 'bg-purple-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>✂️</span> Split / Extract
        </button>
        <button
          onclick={() => { activeMode = 'rotate'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeMode === 'rotate' ? 'bg-amber-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>🔄</span> Rotate Pages
        </button>
        <button
          onclick={() => { activeMode = 'redact'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeMode === 'redact' ? 'bg-rose-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>🕵️‍♂️</span> Byte Redact
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-5 max-h-[70vh] overflow-y-auto">
        <!-- Feedback Alerts -->
        {#if successMessage}
          <div class="rounded-xl border border-emerald-500/30 bg-emerald-950/30 p-3 text-xs text-emerald-300 flex items-center gap-2">
            <span>✅</span>
            <span>{successMessage}</span>
          </div>
        {/if}

        {#if errorMessage}
          <div class="rounded-xl border border-rose-500/30 bg-rose-950/30 p-3 text-xs text-rose-300 flex items-center gap-2">
            <span>⚠️</span>
            <span>{errorMessage}</span>
          </div>
        {/if}

        <!-- Quick Pick from Case Artifacts -->
        {#if availableArtifacts.length > 0}
          <div class="rounded-xl border border-zinc-800 bg-zinc-950/60 p-3.5 space-y-2">
            <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider flex items-center gap-1.5">
              <span>📁</span> Quick-Pick from Case Artifacts:
            </div>
            <div class="flex flex-wrap gap-1.5">
              {#each availableArtifacts as a}
                <button
                  type="button"
                  onclick={() => {
                    if (activeMode === 'merge') addMergeFile(a.path);
                    if (activeMode === 'split') splitInput = a.path;
                    if (activeMode === 'rotate') rotateInput = a.path;
                    if (activeMode === 'redact') redactInput = a.path;
                  }}
                  class="text-xs px-2.5 py-1 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-300 border border-zinc-700/80 flex items-center gap-1 transition-colors"
                >
                  <span class="text-blue-400">📄</span>
                  <span class="font-mono">{a.name}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- 1. MERGE TAB -->
        {#if activeMode === 'merge'}
          <div class="space-y-4">
            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">PDF Files to Combine (In Sequence)</label>
              <p class="text-xs text-zinc-400">Add documents in the order you want them stitched into your court packet.</p>
            </div>

            <!-- Merge Items List -->
            {#if mergeInputs.length === 0}
              <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-950/40 p-6 text-center text-xs text-zinc-500">
                No PDFs added to merge sequence yet. Type or pick a file path below.
              </div>
            {:else}
              <div class="space-y-2">
                {#each mergeInputs as item, idx}
                  <div class="flex items-center justify-between gap-3 p-2.5 rounded-xl border border-zinc-800 bg-zinc-950/60 text-xs font-mono">
                    <div class="flex items-center gap-2 overflow-hidden">
                      <span class="h-5 w-5 rounded bg-blue-600/20 text-blue-300 font-bold flex items-center justify-center text-[10px]">
                        {idx + 1}
                      </span>
                      <span class="text-zinc-200 truncate">{item}</span>
                    </div>
                    <div class="flex items-center gap-1 shrink-0">
                      <button
                        onclick={() => moveMergeFile(idx, -1)}
                        disabled={idx === 0}
                        class="px-1.5 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 disabled:opacity-30"
                        title="Move Up"
                      >▲</button>
                      <button
                        onclick={() => moveMergeFile(idx, 1)}
                        disabled={idx === mergeInputs.length - 1}
                        class="px-1.5 py-0.5 rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-300 disabled:opacity-30"
                        title="Move Down"
                      >▼</button>
                      <button
                        onclick={() => removeMergeFile(idx)}
                        class="px-2 py-0.5 rounded bg-rose-950/40 hover:bg-rose-900/50 text-rose-300 ml-1"
                        title="Remove"
                      >✕</button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Add File Input -->
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={customMergePath}
                placeholder="Paste full path to a PDF (e.g. C:\Users\...\file.pdf)"
                class="flex-1 rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 font-mono focus:border-blue-500 focus:outline-none"
              />
              <button
                type="button"
                onclick={() => addMergeFile(customMergePath)}
                class="px-3.5 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-white text-xs font-bold transition-colors"
              >
                + Add PDF
              </button>
            </div>

            <!-- Output Path -->
            <div class="space-y-1 pt-2">
              <label class="text-xs font-semibold text-zinc-300">Target Output File Path (Optional)</label>
              <input
                type="text"
                bind:value={mergeOutput}
                placeholder="Leave blank to save alongside the first file as '-merged.pdf'"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 placeholder-zinc-500 font-mono focus:border-blue-500 focus:outline-none"
              />
            </div>

            <!-- Submit Button -->
            <div class="pt-3 border-t border-zinc-800 flex justify-end">
              <button
                onclick={runMerge}
                disabled={mergeLoading || mergeInputs.length < 2}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg transition-all active:scale-95"
              >
                <span>📑</span> {mergeLoading ? 'Merging Pure-Rust...' : 'Merge PDFs Now'}
              </button>
            </div>
          </div>
        {/if}

        <!-- 2. SPLIT / EXTRACT TAB -->
        {#if activeMode === 'split'}
          <div class="space-y-4">
            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Source PDF File Path</label>
              <input
                type="text"
                bind:value={splitInput}
                placeholder="e.g. C:\Court\Docket-Packet.pdf"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-purple-500 focus:outline-none"
              />
            </div>

            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Pages to Extract</label>
              <input
                type="text"
                bind:value={splitPages}
                placeholder="e.g. 1-3, 5, 8-12"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-purple-500 focus:outline-none"
              />
              <p class="text-[11px] text-zinc-400">Specify comma-separated page numbers or ranges (e.g. 1-5, 8).</p>
            </div>

            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-300">Target Output File Path (Optional)</label>
              <input
                type="text"
                bind:value={splitOutput}
                placeholder="Leave blank for automatic '-extracted.pdf' name"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-purple-500 focus:outline-none"
              />
            </div>

            <div class="pt-3 border-t border-zinc-800 flex justify-end">
              <button
                onclick={runSplit}
                disabled={splitLoading || !splitInput.trim()}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg transition-all active:scale-95"
              >
                <span>✂️</span> {splitLoading ? 'Extracting Pages...' : 'Extract Selected Pages'}
              </button>
            </div>
          </div>
        {/if}

        <!-- 3. ROTATE TAB -->
        {#if activeMode === 'rotate'}
          <div class="space-y-4">
            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Source PDF File Path</label>
              <input
                type="text"
                bind:value={rotateInput}
                placeholder="e.g. C:\Court\SidewaysScan.pdf"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-amber-500 focus:outline-none"
              />
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div class="space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Rotation Angle (Clockwise)</label>
                <div class="grid grid-cols-3 gap-2">
                  <button
                    type="button"
                    onclick={() => rotateDegrees = 90}
                    class="py-2 rounded-lg border text-xs font-bold transition-all {rotateDegrees === 90 ? 'bg-amber-600 text-white border-amber-500' : 'bg-zinc-800 text-zinc-300 border-zinc-700'}"
                  >
                    90° ↷
                  </button>
                  <button
                    type="button"
                    onclick={() => rotateDegrees = 180}
                    class="py-2 rounded-lg border text-xs font-bold transition-all {rotateDegrees === 180 ? 'bg-amber-600 text-white border-amber-500' : 'bg-zinc-800 text-zinc-300 border-zinc-700'}"
                  >
                    180° ⟲
                  </button>
                  <button
                    type="button"
                    onclick={() => rotateDegrees = 270}
                    class="py-2 rounded-lg border text-xs font-bold transition-all {rotateDegrees === 270 ? 'bg-amber-600 text-white border-amber-500' : 'bg-zinc-800 text-zinc-300 border-zinc-700'}"
                  >
                    270° ↶
                  </button>
                </div>
              </div>

              <div class="space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Specific Pages (Optional)</label>
                <input
                  type="text"
                  bind:value={rotatePages}
                  placeholder="Leave empty to rotate ALL pages, or e.g. 1, 3"
                  class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-amber-500 focus:outline-none"
                />
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-300">Target Output File Path (Optional)</label>
              <input
                type="text"
                bind:value={rotateOutput}
                placeholder="Leave blank for automatic '-rotated.pdf' name"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-amber-500 focus:outline-none"
              />
            </div>

            <div class="pt-3 border-t border-zinc-800 flex justify-end">
              <button
                onclick={runRotate}
                disabled={rotateLoading || !rotateInput.trim()}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg transition-all active:scale-95"
              >
                <span>🔄</span> {rotateLoading ? 'Rotating...' : `Rotate ${rotateDegrees}° Clockwise`}
              </button>
            </div>
          </div>
        {/if}

        <!-- 4. REDACT TAB -->
        {#if activeMode === 'redact'}
          <div class="space-y-4">
            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Source PDF File Path</label>
              <input
                type="text"
                bind:value={redactInput}
                placeholder="e.g. C:\Court\Sensitive-Exhibit.pdf"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-rose-500 focus:outline-none"
              />
            </div>

            <div class="space-y-2">
              <label class="text-xs font-semibold text-zinc-200">Standard PII Privacy Scrubbers</label>
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <label class="flex items-center gap-2 p-2.5 rounded-xl border border-zinc-800 bg-zinc-950/50 text-xs text-zinc-300 cursor-pointer">
                  <input type="checkbox" bind:checked={redactSsn} class="rounded text-rose-600" />
                  <span>🛡️ SSN (XXX-XX-XXXX)</span>
                </label>
                <label class="flex items-center gap-2 p-2.5 rounded-xl border border-zinc-800 bg-zinc-950/50 text-xs text-zinc-300 cursor-pointer">
                  <input type="checkbox" bind:checked={redactPhone} class="rounded text-rose-600" />
                  <span>📞 Phone Numbers</span>
                </label>
                <label class="flex items-center gap-2 p-2.5 rounded-xl border border-zinc-800 bg-zinc-950/50 text-xs text-zinc-300 cursor-pointer">
                  <input type="checkbox" bind:checked={redactDob} class="rounded text-rose-600" />
                  <span>🎂 Dates of Birth (MM/DD/YYYY)</span>
                </label>
              </div>
            </div>

            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Custom Names, Words, or Regex Patterns to Scrub</label>
              <textarea
                bind:value={customPatternsText}
                rows="3"
                placeholder="One custom word or regex pattern per line (e.g. John Doe, Acct# 12345)"
                class="w-full rounded-xl border border-zinc-700 bg-zinc-950/60 p-2.5 text-xs text-zinc-100 font-mono focus:border-rose-500 focus:outline-none"
              ></textarea>
            </div>

            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-300">Target Output File Path (Optional)</label>
              <input
                type="text"
                bind:value={redactOutput}
                placeholder="Leave blank for automatic '-redacted.pdf' name"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-rose-500 focus:outline-none"
              />
            </div>

            <div class="pt-3 border-t border-zinc-800 flex justify-end">
              <button
                onclick={runRedact}
                disabled={redactLoading || !redactInput.trim()}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-rose-600 to-pink-600 hover:from-rose-500 hover:to-pink-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg transition-all active:scale-95"
              >
                <span>🕵️‍♂️</span> {redactLoading ? 'Scrubbing Byte Streams...' : 'Execute Byte Redaction'}
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="p-4 bg-zinc-950 border-t border-zinc-800 flex items-center justify-between text-xs text-zinc-500">
        <div class="flex items-center gap-2">
          <span>🔒</span>
          <span>Zero cloud connection. Safe for confidential False Claims / sealed documents.</span>
        </div>
        <button
          onclick={onclose}
          class="px-4 py-1.5 rounded-lg border border-zinc-700 bg-zinc-900 hover:bg-zinc-800 text-zinc-300 font-medium"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
