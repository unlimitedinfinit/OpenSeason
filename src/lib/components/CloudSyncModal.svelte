<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "$lib/ipc";
  import type { CloudSession, PullReport, PushReport } from "$lib/types/case";

  interface Props {
    open: boolean;
    activeCaseId?: string | null;
    activeCaseTitle?: string | null;
    isConfidential?: boolean;
    onclose?: () => void;
    onsynced?: () => void;
  }

  let { open = $bindable(), activeCaseId = null, activeCaseTitle = null, isConfidential = false, onclose, onsynced }: Props = $props();

  function closeModal() {
    open = false;
    onclose?.();
  }

  let session = $state<CloudSession | null>(null);
  let email = $state("");
  let password = $state("");
  let loading = $state(false);
  let error = $state("");
  let successMsg = $state("");
  let syncReport = $state<PullReport | null>(null);
  let pushReport = $state<PushReport | null>(null);

  onMount(loadStatus);

  $effect(() => {
    if (open) {
      loadStatus();
      error = "";
      successMsg = "";
    }
  });

  async function loadStatus() {
    try {
      session = await ipc<CloudSession | null>("cloud_status_cmd");
    } catch (e) {
      console.warn("Could not load cloud status:", e);
    }
  }

  async function handleLogin(e: Event) {
    e.preventDefault();
    if (!email.trim() || !password) {
      error = "Please enter both email and password.";
      return;
    }
    loading = true;
    error = "";
    successMsg = "";
    try {
      session = await ipc<CloudSession>("cloud_login_cmd", { email: email.trim(), password });
      successMsg = `Connected to JustLegal Cloud as ${session.email}`;
      password = "";
      onsynced?.();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handleLogout() {
    loading = true;
    error = "";
    successMsg = "";
    try {
      await ipc("cloud_logout_cmd");
      session = null;
      successMsg = "Disconnected from JustLegal Cloud.";
      onsynced?.();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handlePullAll() {
    loading = true;
    error = "";
    successMsg = "";
    syncReport = null;
    try {
      const report = await ipc<PullReport>("cloud_pull_cmd", { caseId: null });
      syncReport = report;
      successMsg = `Pulled ${report.cases_pulled} cloud matter(s) and downloaded ${report.files_downloaded} files.`;
      onsynced?.();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  async function handlePushCurrent() {
    if (!activeCaseId) return;
    if (isConfidential) {
      error = "Confidential / sealed cases are strictly airgapped and cannot be pushed to cloud.";
      return;
    }
    loading = true;
    error = "";
    successMsg = "";
    pushReport = null;
    try {
      const report = await ipc<PushReport>("cloud_push_cmd", { caseId: activeCaseId });
      pushReport = report;
      successMsg = report.message;
      onsynced?.();
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
    <div class="w-full max-w-xl rounded-xl border border-zinc-700 bg-zinc-900 shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-zinc-800 px-6 py-4 bg-zinc-950/60">
        <div class="flex items-center gap-3">
          <div class="h-9 w-9 rounded-lg bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 00-9.78 2.096A4.001 4.001 0 003 15z" />
            </svg>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-zinc-100">JustLegal Cloud Sync Hub</h2>
            <p class="text-xs text-zinc-400">Sync matters, orders, and evidence with justlegal.me</p>
          </div>
        </div>
        <button
          onclick={closeModal}
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 transition-colors"
          aria-label="Close"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-5 overflow-y-auto">
        <!-- Privacy & Airgap Guarantee Banner -->
        <div class="rounded-lg border border-emerald-500/20 bg-emerald-950/30 p-3.5 text-xs text-emerald-300 space-y-1">
          <div class="flex items-center gap-2 font-medium text-emerald-200">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            Additive-Only Non-Destructive Guarantee
          </div>
          <p class="text-zinc-400">
            Local drafts, manual edits, and notes are never overwritten. Downloads route cleanly to <code class="text-zinc-300">orders/</code>, <code class="text-zinc-300">filings/</code>, and <code class="text-zinc-300">evidence/</code>. Confidential cases remain strictly airgapped.
          </p>
        </div>

        {#if error}
          <div class="rounded-lg border border-red-500/30 bg-red-950/40 p-3 text-xs text-red-300">
            {error}
          </div>
        {/if}

        {#if successMsg}
          <div class="rounded-lg border border-green-500/30 bg-green-950/40 p-3 text-xs text-green-300">
            {successMsg}
          </div>
        {/if}

        {#if session}
          <!-- Logged In State -->
          <div class="rounded-lg border border-zinc-800 bg-zinc-950/50 p-4 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-xs text-zinc-400">Connected Account</p>
                <p class="text-sm font-semibold text-zinc-200">{session.email}</p>
                <p class="text-[11px] text-zinc-500 font-mono">UID: {session.uid}</p>
              </div>
              <button
                onclick={handleLogout}
                disabled={loading}
                class="text-xs px-3 py-1.5 rounded-md border border-zinc-700 hover:bg-zinc-800 text-zinc-300 transition-colors"
              >
                Sign Out
              </button>
            </div>

            <!-- Sync Controls -->
            <div class="pt-3 border-t border-zinc-800 space-y-3">
              <div class="flex flex-col sm:flex-row gap-3">
                <button
                  onclick={handlePullAll}
                  disabled={loading}
                  class="flex-1 py-2 px-4 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
                >
                  {#if loading}
                    <span class="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-white border-t-transparent"></span>
                  {:else}
                    <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                    </svg>
                  {/if}
                  Sync All Matters from Cloud
                </button>

                {#if activeCaseId && !isConfidential}
                  <button
                    onclick={handlePushCurrent}
                    disabled={loading}
                    class="py-2 px-4 rounded-lg border border-zinc-700 hover:bg-zinc-800 text-zinc-200 text-xs font-medium transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
                  >
                    <svg class="h-4 w-4 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                    </svg>
                    Push "{activeCaseTitle || 'Current Case'}"
                  </button>
                {/if}
              </div>

              {#if isConfidential}
                <p class="text-xs text-amber-400/90 italic">
                  Note: The open case is marked Confidential / Sealed and is blocked from cloud sync by design.
                </p>
              {/if}
            </div>

            {#if syncReport}
              <div class="rounded-md bg-zinc-900 border border-zinc-800 p-3 text-xs space-y-1">
                <p class="font-medium text-zinc-300">Last Pull Summary:</p>
                <p class="text-zinc-400">• Matters Pulled: <span class="text-zinc-200 font-semibold">{syncReport.cases_pulled}</span></p>
                <p class="text-zinc-400">• Files Downloaded: <span class="text-zinc-200 font-semibold">{syncReport.files_downloaded}</span></p>
              </div>
            {/if}
          </div>
        {:else}
          <!-- Login Form -->
          <form onsubmit={handleLogin} class="space-y-4">
            <div>
              <label for="cloud-email" class="block text-xs font-medium text-zinc-300 mb-1">JustLegal Email Address</label>
              <input
                id="cloud-email"
                type="email"
                bind:value={email}
                required
                placeholder="you@example.com"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </div>

            <div>
              <label for="cloud-password" class="block text-xs font-medium text-zinc-300 mb-1">Password</label>
              <input
                id="cloud-password"
                type="password"
                bind:value={password}
                required
                placeholder="••••••••"
                class="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-200 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              class="w-full py-2.5 px-4 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
            >
              {#if loading}
                <span class="inline-block h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"></span>
                Connecting...
              {:else}
                Sign In & Connect Cloud
              {/if}
            </button>
          </form>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-6 py-3 border-t border-zinc-800 bg-zinc-950/40 flex justify-between items-center text-xs text-zinc-500">
        <span>JustLegal Cloud • Project: justlegalapp1</span>
        <button onclick={onclose} class="text-zinc-400 hover:text-zinc-200 transition-colors">
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
