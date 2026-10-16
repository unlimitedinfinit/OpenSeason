<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";

  type Manifest = {
    never_files?: boolean;
    never_serves?: boolean;
    notice?: string;
    case_folder?: string;
    mcp_launch?: { command: string; args: string[]; cwd: string };
    tools?: { name: string; cli: string; mcp: string; summary: string }[];
    starter_prompt?: string;
  };

  let manifest = $state<Manifest | null>(null);
  let error = $state("");
  let copied = $state("");

  onMount(async () => {
    try {
      manifest = await ipc<Manifest>("agent_manifest_cmd");
    } catch (e) {
      error = String(e);
    }
  });

  async function copy(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      copied = label;
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

<main class="container mx-auto p-8 space-y-6 max-w-4xl">
  <div>
    <p class="text-xs uppercase tracking-widest text-muted-foreground">Local AI contract</p>
    <h1 class="text-3xl font-bold">For your AI</h1>
    <p class="text-muted-foreground mt-2">
      OpenSeason is a formatting engine. An AI on this computer can call the CLI or MCP server. It must never invent facts, citations, or deadlines, and it cannot file or serve.
    </p>
  </div>

  <ReviewNotice />

  {#if error}
    <p class="text-sm text-destructive">{error}</p>
  {/if}

  {#if copied}
    <p class="text-sm text-muted-foreground">{copied} copied.</p>
  {/if}

  {#if manifest}
    <section class="border border-border rounded-lg p-4 space-y-2">
      <h2 class="font-semibold">Case folder</h2>
      <p class="text-sm font-mono">{manifest.case_folder}</p>
      <p class="text-xs text-muted-foreground">{manifest.notice}</p>
    </section>

    <section class="border border-border rounded-lg p-4 space-y-2">
      <div class="flex justify-between items-center">
        <h2 class="font-semibold">MCP launch</h2>
        <button class="text-xs border border-border px-2 py-1 rounded-md" onclick={() => copy(mcpConfig, "MCP config")}>Copy config</button>
      </div>
      <pre class="text-xs bg-background border border-border rounded-md p-3 overflow-x-auto">{mcpConfig}</pre>
      <p class="text-xs text-muted-foreground">Accepts Content-Length framing or newline-delimited JSON-RPC. Never files.</p>
    </section>

    <section class="space-y-2">
      <h2 class="font-semibold">Tools</h2>
      <ul class="space-y-2">
        {#each manifest.tools || [] as tool}
          <li class="border border-border rounded-md p-3">
            <p class="font-medium">{tool.name}</p>
            <p class="text-sm text-muted-foreground">{tool.summary}</p>
            <p class="text-[11px] font-mono text-muted-foreground mt-1">CLI: openseason --json {tool.cli} · MCP: {tool.mcp}</p>
          </li>
        {/each}
      </ul>
    </section>

    <section class="border border-border rounded-lg p-4 space-y-2">
      <div class="flex justify-between items-center">
        <h2 class="font-semibold">Starter prompt for Gemini or Claude</h2>
        <button class="text-xs border border-border px-2 py-1 rounded-md" onclick={() => copy(manifest?.starter_prompt || "", "Starter prompt")}>Copy prompt</button>
      </div>
      <pre class="text-xs bg-background border border-border rounded-md p-3 whitespace-pre-wrap">{manifest.starter_prompt}</pre>
    </section>
  {/if}
</main>
