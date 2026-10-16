<script lang="ts">
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

  let { manifest }: { manifest: Manifest | null } = $props();
  let copied = $state("");

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
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-base font-semibold text-foreground">External Agent Connection (MCP & CLI)</h3>
      <p class="text-xs text-muted-foreground mt-0.5">
        Connect Claude Desktop, Cursor, or Gemini CLI to this local engine via stdio JSON-RPC.
      </p>
    </div>
    {#if copied}
      <span class="text-xs text-primary font-medium animate-pulse">{copied} copied!</span>
    {/if}
  </div>

  <ReviewNotice />

  {#if manifest}
    <div class="grid gap-4 md:grid-cols-2">
      <div class="rounded-xl border border-border/60 bg-card/60 backdrop-blur-md p-4 space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Claude Desktop / Cursor MCP Config</span>
          <button class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50 text-foreground" onclick={() => copy(mcpConfig, "MCP config")}>
            Copy JSON
          </button>
        </div>
        <pre class="text-[11px] font-mono bg-background/80 border border-border/40 rounded-lg p-3 overflow-x-auto text-foreground/90 max-h-48">{mcpConfig}</pre>
        <p class="text-[10px] text-muted-foreground">Supports Content-Length framing and newline-delimited JSON-RPC.</p>
      </div>

      <div class="rounded-xl border border-border/60 bg-card/60 backdrop-blur-md p-4 space-y-2">
        <div class="flex justify-between items-center">
          <span class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Agent Rule Starter Prompt</span>
          <button class="text-xs border border-border px-2.5 py-1 rounded hover:border-primary/50 text-foreground" onclick={() => copy(manifest?.starter_prompt || "", "Starter prompt")}>
            Copy Prompt
          </button>
        </div>
        <pre class="text-[11px] font-mono bg-background/80 border border-border/40 rounded-lg p-3 overflow-y-auto text-foreground/90 max-h-48 whitespace-pre-wrap">{manifest.starter_prompt}</pre>
        <p class="text-[10px] text-muted-foreground">Enforces single-engine rules: never invent cites, never file without consent.</p>
      </div>
    </div>

    <div class="rounded-xl border border-border/60 bg-card/60 backdrop-blur-md p-4 space-y-3">
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
  {/if}
</div>
