<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CaseArtifact, SignatureProfile, SignatureStampResult } from "$lib/types/case";

  interface Props {
    open?: boolean;
    caseId?: string;
    onclose: () => void;
  }

  let { open = $bindable(false), caseId = "", onclose }: Props = $props();

  type Tab = "profile" | "stamp";
  let activeTab = $state<Tab>("stamp");

  // Signature Profile
  let profile = $state<SignatureProfile>({
    full_name: "",
    initials: "",
    style: "script_formal",
    signature_data: null,
    court_s_block: "",
    updated_at: "",
  });

  // Stamping State
  let targetPdfPath = $state("");
  let pageNumber = $state<number>(1);
  let posX = $state<number>(350);
  let posY = $state<number>(120);
  let stamping = $state(false);
  let stampResult = $state<SignatureStampResult | null>(null);

  // Canvas Drawing State
  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let isDrawing = $state(false);

  // Case Artifacts for quick pick
  let availableArtifacts = $state<CaseArtifact[]>([]);

  let errorMessage = $state("");
  let successMessage = $state("");
  let savingProfile = $state(false);

  onMount(async () => {
    await loadProfile();
    await loadArtifacts();
  });

  async function loadProfile() {
    try {
      const p = await ipc<SignatureProfile>("get_signature_profile_cmd");
      if (p) profile = p;
      if (!profile.court_s_block && profile.full_name) {
        profile.court_s_block = `/s/ ${profile.full_name}`;
      }
    } catch {
      // Use defaults
    }
  }

  async function loadArtifacts() {
    if (!caseId) return;
    try {
      const list = await ipc<CaseArtifact[]>("list_case_artifacts", { caseId }).catch(() => []);
      availableArtifacts = list.filter((a) => a.path.toLowerCase().endsWith(".pdf"));
      if (availableArtifacts.length > 0 && !targetPdfPath) {
        targetPdfPath = availableArtifacts[0].path;
      }
    } catch {
      availableArtifacts = [];
    }
  }

  function handleNameChange() {
    if (profile.full_name) {
      profile.court_s_block = `/s/ ${profile.full_name}`;
      const parts = profile.full_name.trim().split(/\s+/);
      profile.initials = parts.map((p) => p[0]?.toUpperCase() || "").join("");
    }
  }

  // Canvas Drawing handlers
  function startDraw(e: MouseEvent | TouchEvent) {
    if (!canvasEl) return;
    isDrawing = true;
    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;
    ctx.lineWidth = 2.5;
    ctx.lineCap = "round";
    ctx.strokeStyle = "#38bdf8";
    const rect = canvasEl.getBoundingClientRect();
    const x = ("clientX" in e ? e.clientX : e.touches[0].clientX) - rect.left;
    const y = ("clientY" in e ? e.clientY : e.touches[0].clientY) - rect.top;
    ctx.beginPath();
    ctx.moveTo(x, y);
  }

  function drawMove(e: MouseEvent | TouchEvent) {
    if (!isDrawing || !canvasEl) return;
    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;
    const rect = canvasEl.getBoundingClientRect();
    const x = ("clientX" in e ? e.clientX : e.touches[0].clientX) - rect.left;
    const y = ("clientY" in e ? e.clientY : e.touches[0].clientY) - rect.top;
    ctx.lineTo(x, y);
    ctx.stroke();
  }

  function endDraw() {
    isDrawing = false;
    if (canvasEl) {
      profile.signature_data = canvasEl.toDataURL("image/png");
    }
  }

  function clearCanvas() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);
    profile.signature_data = null;
  }

  async function saveProfile() {
    savingProfile = true;
    errorMessage = "";
    successMessage = "";
    try {
      profile.updated_at = new Date().toISOString();
      await ipc("save_signature_profile_cmd", { profile });
      successMessage = "Signature profile saved locally!";
    } catch (e) {
      errorMessage = String(e);
    } finally {
      savingProfile = false;
    }
  }

  function applyPreset(preset: "bottom_right" | "bottom_left" | "cert_line") {
    if (preset === "bottom_right") {
      posX = 350;
      posY = 120;
    } else if (preset === "bottom_left") {
      posX = 72;
      posY = 120;
    } else if (preset === "cert_line") {
      posX = 350;
      posY = 180;
    }
  }

  async function stampSignature() {
    if (!targetPdfPath.trim()) {
      errorMessage = "Please choose a PDF file to stamp.";
      return;
    }
    const sigText = profile.court_s_block.trim() || `/s/ ${profile.full_name || 'Signer'}`;
    stamping = true;
    errorMessage = "";
    successMessage = "";
    stampResult = null;

    try {
      const res = await ipc<SignatureStampResult>("stamp_pdf_signature_cmd", {
        pdfPath: targetPdfPath.trim(),
        signatureText: sigText,
        pageNumber: pageNumber || 1,
        x: posX,
        y: posY,
      });

      stampResult = res;
      successMessage = `Document stamped successfully on page ${pageNumber}!`;
    } catch (e) {
      errorMessage = String(e);
    } finally {
      stamping = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 overflow-y-auto animate-fade-in">
    <div class="w-full max-w-3xl rounded-2xl border border-border bg-card shadow-2xl overflow-hidden my-6">
      <!-- Modal Header -->
      <div class="bg-card p-5 border-b border-border flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="h-12 w-12 rounded-xl bg-emerald-500/20 border border-emerald-400/30 flex items-center justify-center text-2xl shadow-inner">
            ✍️
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-base font-bold text-white tracking-tight">Visual Legal Signature & Stamp Studio</h2>
              <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                AcroForm & Direct Stamp
              </span>
            </div>
            <p class="text-xs text-zinc-300 mt-0.5">
              Apply court-compliant electronic `/s/` signatures, formal cursive, or hand-drawn stamps without cloud services.
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

      <!-- Navigation Tabs -->
      <div class="grid grid-cols-2 border-b border-zinc-800 bg-zinc-950/50 p-1.5 gap-1.5">
        <button
          onclick={() => { activeTab = 'stamp'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeTab === 'stamp' ? 'bg-emerald-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>📑</span> 1-Click Stamp PDF
        </button>
        <button
          onclick={() => { activeTab = 'profile'; errorMessage = ''; successMessage = ''; }}
          class="py-2.5 px-3 rounded-xl text-xs font-semibold flex items-center justify-center gap-2 transition-all {activeTab === 'profile' ? 'bg-teal-600 text-white shadow-md' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'}"
        >
          <span>⚙️</span> Signature Profile & Style
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-5 max-h-[70vh] overflow-y-auto">
        <!-- Alerts -->
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

        <!-- TAB 1: 1-CLICK STAMP PDF -->
        {#if activeTab === 'stamp'}
          <div class="space-y-4">
            <!-- Active Signature Preview Card -->
            <div class="rounded-xl border border-emerald-500/30 bg-emerald-950/20 p-4 flex items-center justify-between gap-4">
              <div>
                <span class="text-[10px] font-bold uppercase tracking-wider text-emerald-400">Active Signature Block</span>
                <div class="text-sm font-bold text-zinc-100 font-mono mt-0.5">
                  {profile.court_s_block || '/s/ Your Name'}
                </div>
                <div class="text-[11px] text-zinc-400">
                  Style: <span class="capitalize text-zinc-300">{profile.style.replace('_', ' ')}</span>
                </div>
              </div>
              <button
                onclick={() => activeTab = 'profile'}
                class="text-xs px-3 py-1.5 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 border border-zinc-700 transition-colors"
              >
                Customize
              </button>
            </div>

            <!-- Quick pick PDF from artifacts -->
            {#if availableArtifacts.length > 0}
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-zinc-300">Quick-Pick from Case Exports</label>
                <div class="flex flex-wrap gap-1.5">
                  {#each availableArtifacts as a}
                    <button
                      type="button"
                      onclick={() => targetPdfPath = a.path}
                      class="text-xs px-2.5 py-1 rounded-lg border flex items-center gap-1 transition-all {targetPdfPath === a.path ? 'bg-emerald-600 text-white border-emerald-500 shadow-sm' : 'bg-zinc-800 text-zinc-300 border-zinc-700 hover:bg-zinc-700'}"
                    >
                      <span>📄</span>
                      <span class="font-mono">{a.name}</span>
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Target PDF Input -->
            <div class="space-y-1">
              <label class="text-xs font-semibold text-zinc-200">Target PDF Path</label>
              <input
                type="text"
                bind:value={targetPdfPath}
                placeholder="e.g. C:\Users\...\exports\notice-of-appeal.pdf"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-emerald-500 focus:outline-none"
              />
            </div>

            <!-- Page Number & Position Presets -->
            <div class="grid grid-cols-1 sm:grid-cols-12 gap-3">
              <div class="sm:col-span-4 space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Target Page #</label>
                <input
                  type="number"
                  min="1"
                  bind:value={pageNumber}
                  class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-emerald-500 focus:outline-none"
                />
              </div>

              <div class="sm:col-span-8 space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Position Placement Presets</label>
                <div class="grid grid-cols-3 gap-2">
                  <button
                    type="button"
                    onclick={() => applyPreset('bottom_right')}
                    class="py-2 px-1.5 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-colors"
                  >
                    📍 Bottom Right
                  </button>
                  <button
                    type="button"
                    onclick={() => applyPreset('bottom_left')}
                    class="py-2 px-1.5 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-colors"
                  >
                    📍 Bottom Left
                  </button>
                  <button
                    type="button"
                    onclick={() => applyPreset('cert_line')}
                    class="py-2 px-1.5 rounded-lg border border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-colors"
                  >
                    📍 Service Line
                  </button>
                </div>
              </div>
            </div>

            <!-- Fine-tuning Sliders -->
            <div class="p-3.5 rounded-xl border border-zinc-800 bg-zinc-950/40 space-y-3">
              <div class="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider flex items-center justify-between">
                <span>Placement Fine Tuning (Points from Bottom-Left)</span>
                <span class="font-mono text-zinc-300">X: {posX}pt, Y: {posY}pt</span>
              </div>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div class="space-y-1">
                  <div class="flex justify-between text-xs text-zinc-400">
                    <span>Horizontal (X)</span>
                    <span>{posX}</span>
                  </div>
                  <input type="range" min="30" max="500" bind:value={posX} class="w-full accent-emerald-500" />
                </div>
                <div class="space-y-1">
                  <div class="flex justify-between text-xs text-zinc-400">
                    <span>Vertical (Y)</span>
                    <span>{posY}</span>
                  </div>
                  <input type="range" min="30" max="750" bind:value={posY} class="w-full accent-emerald-500" />
                </div>
              </div>
            </div>

            <!-- Stamp Action Button -->
            <div class="pt-3 border-t border-zinc-800 flex items-center justify-between">
              <span class="text-[11px] text-zinc-500">Creates a clean '-signed.pdf' without modifying original.</span>
              <button
                onclick={stampSignature}
                disabled={stamping || !targetPdfPath.trim()}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg transition-all active:scale-95"
              >
                <span>🔏</span> {stamping ? 'Stamping Document...' : 'Stamp Signature Now'}
              </button>
            </div>

            <!-- Stamped Result -->
            {#if stampResult}
              <div class="rounded-xl border border-emerald-500/40 bg-emerald-950/20 p-4 space-y-1">
                <div class="text-xs font-semibold text-emerald-300 flex items-center gap-1.5">
                  <span>✨</span> Signed Court Document:
                </div>
                <div class="text-xs font-mono text-zinc-200 break-all p-2 rounded bg-zinc-900 border border-zinc-800">
                  {stampResult.output_path}
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <!-- TAB 2: PROFILE & STYLE -->
        {#if activeTab === 'profile'}
          <div class="space-y-4">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div class="space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Full Legal Name</label>
                <input
                  type="text"
                  bind:value={profile.full_name}
                  oninput={handleNameChange}
                  placeholder="e.g. Joshua Abrams"
                  class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 focus:border-teal-500 focus:outline-none"
                />
              </div>

              <div class="space-y-1">
                <label class="text-xs font-semibold text-zinc-200">Legal `/s/` Signature Block</label>
                <input
                  type="text"
                  bind:value={profile.court_s_block}
                  placeholder="e.g. /s/ Joshua Abrams, Pro Se"
                  class="w-full rounded-lg border border-zinc-700 bg-zinc-950/60 px-3 py-2 text-xs text-zinc-100 font-mono focus:border-teal-500 focus:outline-none"
                />
              </div>
            </div>

            <!-- Style Selector -->
            <div class="space-y-2">
              <label class="text-xs font-semibold text-zinc-200">Signature Style Appearance</label>
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <button
                  type="button"
                  onclick={() => profile.style = 'script_formal'}
                  class="p-3 rounded-xl border text-left transition-all {profile.style === 'script_formal' ? 'border-teal-500 bg-teal-950/20 shadow-md ring-1 ring-teal-500' : 'border-zinc-800 bg-zinc-950/50 hover:bg-zinc-800'}"
                >
                  <div class="text-xs font-bold text-zinc-200">🖋️ Formal Cursive</div>
                  <div class="text-lg italic text-teal-300 font-serif mt-1">
                    {profile.full_name || 'Signature'}
                  </div>
                </button>

                <button
                  type="button"
                  onclick={() => profile.style = 'script_modern'}
                  class="p-3 rounded-xl border text-left transition-all {profile.style === 'script_modern' ? 'border-teal-500 bg-teal-950/20 shadow-md ring-1 ring-teal-500' : 'border-zinc-800 bg-zinc-950/50 hover:bg-zinc-800'}"
                >
                  <div class="text-xs font-bold text-zinc-200">✨ Modern Script</div>
                  <div class="text-lg italic text-primary font-mono mt-1">
                    {profile.full_name || 'Signature'}
                  </div>
                </button>

                <button
                  type="button"
                  onclick={() => profile.style = 'drawn'}
                  class="p-3 rounded-xl border text-left transition-all {profile.style === 'drawn' ? 'border-teal-500 bg-teal-950/20 shadow-md ring-1 ring-teal-500' : 'border-zinc-800 bg-zinc-950/50 hover:bg-zinc-800'}"
                >
                  <div class="text-xs font-bold text-zinc-200">🎨 Hand-Drawn / Stylus</div>
                  <div class="text-xs text-zinc-400 mt-1">
                    Draw with mouse, stylus, or touchpad
                  </div>
                </button>
              </div>
            </div>

            <!-- Interactive Drawing Pad if Drawn selected -->
            {#if profile.style === 'drawn'}
              <div class="space-y-2 p-4 rounded-xl border border-zinc-800 bg-zinc-950/60">
                <div class="flex items-center justify-between">
                  <span class="text-xs font-semibold text-zinc-300">Sign Inside the Box:</span>
                  <button
                    type="button"
                    onclick={clearCanvas}
                    class="text-xs text-rose-400 hover:text-rose-300 underline"
                  >
                    Clear Pad
                  </button>
                </div>
                <canvas
                  bind:this={canvasEl}
                  width="500"
                  height="120"
                  onmousedown={startDraw}
                  onmousemove={drawMove}
                  onmouseup={endDraw}
                  onmouseleave={endDraw}
                  ontouchstart={startDraw}
                  ontouchmove={drawMove}
                  ontouchend={endDraw}
                  class="w-full h-28 bg-zinc-900 border border-zinc-700 rounded-lg cursor-crosshair touch-none"
                ></canvas>
              </div>
            {/if}

            <div class="pt-3 border-t border-zinc-800 flex justify-end">
              <button
                onclick={saveProfile}
                disabled={savingProfile}
                class="px-5 py-2.5 rounded-xl bg-gradient-to-r from-teal-600 to-emerald-600 hover:from-teal-500 hover:to-emerald-500 text-white text-xs font-bold flex items-center gap-2 shadow-md transition-all active:scale-95"
              >
                <span>💾</span> {savingProfile ? 'Saving...' : 'Save Signature Profile'}
              </button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="p-4 bg-zinc-950 border-t border-zinc-800 flex items-center justify-between text-xs text-zinc-500">
        <div>
          Signatures are stored strictly in your local AppData / JustLegal configuration.
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
