<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";

  interface Manifest {
    never_files?: boolean;
    never_serves?: boolean;
    notice?: string;
    case_folder?: string;
    mcp_launch?: { command: string; args: string[]; cwd: string };
    tools?: { name: string; cli: string; mcp: string; summary: string }[];
    starter_prompt?: string;
  }

  interface DoctorReport {
    cli_version: string;
    cases_root: string;
    cases_root_exists: boolean;
    packs_root: string;
    packs_root_exists: boolean;
    rule_packs_available: number;
    offline_crypto_ready: boolean;
    airgap_status: string;
    eo14395_quitam_engine_ready: boolean;
    all_systems_operational: boolean;
  }

  let { manifest }: { manifest: Manifest | null } = $props();
  let copied = $state("");
  let doctorReport = $state<DoctorReport | null>(null);
  let checkingDoctor = $state(false);
  let showAdvancedDev = $state(false);

  onMount(async () => {
    await runDoctorCheck();
  });

  async function runDoctorCheck() {
    checkingDoctor = true;
    try {
      doctorReport = await ipc<DoctorReport>("doctor_diagnostic_cmd").catch(() => null);
    } catch {
      doctorReport = null;
    } finally {
      checkingDoctor = false;
    }
  }

  async function copy(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = label;
      setTimeout(() => (copied = ""), 3000);
    } catch {
      copied = "Could not copy";
    }
  }

  const mcpConfig = $derived(
    JSON.stringify(
      {
        mcpServers: {
          openseason: {
            command: manifest?.mcp_launch?.command || "openseason",
            args: manifest?.mcp_launch?.args || ["mcp"],
          },
        },
      },
      null,
      2,
    ),
  );
</script>

<div class="space-y-6">
  <!-- Top Headline -->
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-base font-bold text-foreground">Work with an AI Assistant</h3>
      <p class="text-xs text-muted-foreground mt-0.5">
        Connect JustLegal Assistant (on GrokBot, Claude, Cursor, or Gemini) to safely organize cases, decode orders, and triage claims.
      </p>
    </div>
    {#if copied}
      <span class="text-xs text-primary font-medium animate-pulse">{copied} copied!</span>
    {/if}
  </div>

  <ReviewNotice />

  <!-- Assistant Connection Status Card -->
  <div class="rounded-3xl border border-primary/30 bg-card/80 backdrop-blur-xl p-5 shadow-lg space-y-4">
    <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 border-b border-border/50 pb-3">
      <div class="flex items-center gap-2.5">
        <span class="w-2.5 h-2.5 rounded-full {doctorReport?.all_systems_operational ? 'bg-green-500 animate-pulse' : 'bg-amber-500'}"></span>
        <h4 class="text-sm font-semibold text-foreground">Local System Engine Status</h4>
        <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-secondary text-muted-foreground">
          v{doctorReport?.cli_version || "0.1.0"}
        </span>
      </div>
      <button
        type="button"
        onclick={runDoctorCheck}
        disabled={checkingDoctor}
        class="text-xs px-3 py-1.5 rounded-xl border border-border bg-secondary/60 hover:bg-secondary text-foreground transition-colors font-medium"
      >
        {checkingDoctor ? "Checking..." : "↻ Run Diagnostics"}
      </button>
    </div>

    <!-- Health Grid -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
      <div class="p-3 rounded-2xl bg-background/60 border border-border/60 space-y-1">
        <span class="text-[10px] uppercase font-bold text-muted-foreground">Workspace</span>
        <div class="font-medium text-foreground truncate">{doctorReport?.cases_root_exists ? "Initialized" : "Not Found"}</div>
        <div class="text-[10px] text-muted-foreground font-mono truncate">{doctorReport?.cases_root || "JustLegal/Cases"}</div>
      </div>

      <div class="p-3 rounded-2xl bg-background/60 border border-border/60 space-y-1">
        <span class="text-[10px] uppercase font-bold text-muted-foreground">State Rule Packs</span>
        <div class="font-medium text-foreground">{doctorReport?.rule_packs_available || 5} Bundled Packs</div>
        <div class="text-[10px] text-muted-foreground font-mono">NJ, CO, Federal, SCOTUS</div>
      </div>

      <div class="p-3 rounded-2xl bg-background/60 border border-border/60 space-y-1">
        <span class="text-[10px] uppercase font-bold text-muted-foreground">Pure-Rust Crypto</span>
        <div class="font-medium text-foreground">Argon2id + XChaCha20</div>
        <div class="text-[10px] text-green-400 font-semibold">100% Offline Ready</div>
      </div>

      <div class="p-3 rounded-2xl bg-background/60 border border-border/60 space-y-1">
        <span class="text-[10px] uppercase font-bold text-muted-foreground">EO 14395 Fraud Engine</span>
        <div class="font-medium text-foreground">NPI + Rule 9(b)</div>
        <div class="text-[10px] text-amber-400 font-semibold">Qui Tam Triage Active</div>
      </div>
    </div>

    <!-- Access Permissions Ledger -->
    <div class="p-3.5 rounded-2xl bg-secondary/30 border border-border/60 space-y-2 text-xs">
      <span class="text-[11px] font-bold uppercase tracking-wider text-muted-foreground">What Your Connected Assistant Can & Cannot Access:</span>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-2 text-[11px]">
        <div class="flex items-start gap-2 text-foreground">
          <span class="text-green-400 font-bold">🟢</span>
          <span><strong>Public Forms & Rules:</strong> 22,000+ official court forms, AcroForm catalogs, and deadline rules.</span>
        </div>
        <div class="flex items-start gap-2 text-foreground">
          <span class="text-amber-400 font-bold">🟡</span>
          <span><strong>Authorized Case Workspace:</strong> Reads and organizes active case folders only when you approve the local command.</span>
        </div>
        <div class="flex items-start gap-2 text-foreground">
          <span class="text-red-400 font-bold">🔴</span>
          <span><strong>Confidential / Qui Tam Vault:</strong> Excluded from external AI agents. Sealed evidence is never exposed to remote models.</span>
        </div>
      </div>
    </div>
  </div>

  <!-- Assistant Personas Grid -->
  <div class="grid gap-4 md:grid-cols-2">
    <!-- Persona 1: Fraud & Whistleblower Navigator -->
    <div class="rounded-3xl border border-amber-500/30 bg-gradient-to-b from-amber-500/5 to-card/70 p-5 space-y-3.5 shadow-sm">
      <div class="flex items-start justify-between gap-3">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-amber-500/20 text-amber-400 border border-amber-500/30 text-lg">
            🛡️
          </div>
          <div>
            <h4 class="font-bold text-sm text-foreground">JustLegal Fraud & Whistleblower Navigator</h4>
            <p class="text-[11px] font-mono text-amber-400/90">GrokBot Template · EO 14395 & Qui Tam</p>
          </div>
        </div>
        <button
          type="button"
          class="text-xs px-3 py-1.5 rounded-xl border border-amber-500/40 bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 font-medium transition-colors"
          onclick={() => copy("You are the JustLegal Fraud & Whistleblower Navigator sponsored by Just Liberty (501(c)(3)). Your role is to help citizens triage government fraud under EO 14395 and the False Claims Act (31 U.S.C. 3729). Mandatory 60-day seal rule under 3730(b)(2): never post evidence to social media or press before filing under seal.", "Fraud Prompt")}
        >
          Copy Bot Prompt
        </button>
      </div>

      <p class="text-xs text-muted-foreground leading-relaxed">
        Specialized for citizen investigators and whistleblowers. Evaluates suspect clinics across CMS NPI, calculates Rule 9(b) viability scores (0–100), and prepares the DOJ 60-day disclosure dossier.
      </p>

      <div class="pt-1 flex items-center justify-between text-[11px] text-muted-foreground border-t border-border/40">
        <span>Template: <code class="text-amber-300 font-mono">MCP_Grok/bot_fraud_recipe.json</code></span>
        <span class="text-green-400 font-semibold">✓ Airgap Protected</span>
      </div>
    </div>

    <!-- Persona 2: Civil Legal Navigator -->
    <div class="rounded-3xl border border-primary/30 bg-gradient-to-b from-primary/5 to-card/70 p-5 space-y-3.5 shadow-sm">
      <div class="flex items-start justify-between gap-3">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-primary/20 text-primary border border-primary/30 text-lg">
            ⚖️
          </div>
          <div>
            <h4 class="font-bold text-sm text-foreground">JustLegal Civil Navigator</h4>
            <p class="text-[11px] font-mono text-primary/90">GrokBot Template · Civil Self-Help</p>
          </div>
        </div>
        <button
          type="button"
          class="text-xs px-3 py-1.5 rounded-xl border border-primary/40 bg-primary/10 hover:bg-primary/20 text-primary font-medium transition-colors"
          onclick={() => copy("You are the JustLegal Civil Navigator sponsored by Just Liberty (501(c)(3)). Your role is to assist pro se litigants in understanding court orders, finding official court forms, and calculating deadlines without inventing citations or practicing law.", "Civil Prompt")}
        >
          Copy Bot Prompt
        </button>
      </div>

      <p class="text-xs text-muted-foreground leading-relaxed">
        Designed for everyday civil court navigation. Guides pro se litigants through plain-English intake, decodes court minute orders, finds verified state court forms, and prepares certificates of service.
      </p>

      <div class="pt-1 flex items-center justify-between text-[11px] text-muted-foreground border-t border-border/40">
        <span>Template: <code class="text-primary font-mono">MCP_Grok/system_prompt_civil.md</code></span>
        <span class="text-green-400 font-semibold">✓ Standalone Ready</span>
      </div>
    </div>
  </div>

  <!-- Collapsible Advanced Developer Section -->
  <div class="border-t border-border/60 pt-4">
    <button
      type="button"
      onclick={() => (showAdvancedDev = !showAdvancedDev)}
      class="text-xs text-muted-foreground hover:text-foreground font-semibold flex items-center gap-1.5 transition-colors"
    >
      <span>{showAdvancedDev ? "▼ Hide Advanced Developer Settings & Raw MCP Config" : "▶ Show Advanced Developer Settings & Raw MCP Config"}</span>
    </button>

    {#if showAdvancedDev && manifest}
      <div class="grid gap-4 md:grid-cols-2 mt-4 animate-in fade-in duration-150">
        <div class="rounded-2xl border border-border/60 bg-card/60 p-4 space-y-2">
          <div class="flex justify-between items-center">
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Claude Desktop / Cursor MCP Config</span>
            <button class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50 text-foreground" onclick={() => copy(mcpConfig, "MCP config")}>
              Copy JSON
            </button>
          </div>
          <pre class="text-[11px] font-mono bg-background/80 border border-border/40 rounded-lg p-3 overflow-x-auto text-foreground/90 max-h-48">{mcpConfig}</pre>
        </div>

        <div class="rounded-2xl border border-border/60 bg-card/60 p-4 space-y-2">
          <div class="flex justify-between items-center">
            <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Raw Agent Starter Prompt</span>
            <button class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50 text-foreground" onclick={() => copy(manifest?.starter_prompt || "", "Starter prompt")}>
              Copy Prompt
            </button>
          </div>
          <pre class="text-[11px] font-mono bg-background/80 border border-border/40 rounded-lg p-3 overflow-y-auto text-foreground/90 max-h-48 whitespace-pre-wrap">{manifest.starter_prompt}</pre>
        </div>

        <div class="md:col-span-2 rounded-2xl border border-border/60 bg-card/60 p-4 space-y-3">
          <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Exposed Tool Endpoints ({manifest.tools?.length || 0})</span>
          <div class="grid gap-2 sm:grid-cols-2 md:grid-cols-3 max-h-64 overflow-y-auto pr-1">
            {#each manifest.tools || [] as tool}
              <div class="border border-border/50 rounded-lg p-2.5 bg-background/50 text-left space-y-1">
                <p class="font-medium text-xs text-foreground truncate">{tool.name}</p>
                <p class="text-[11px] text-muted-foreground line-clamp-2">{tool.summary}</p>
                <p class="text-[10px] font-mono text-primary/80 truncate">CLI: --json {tool.cli}</p>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
