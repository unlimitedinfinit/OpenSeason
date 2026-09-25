<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import ReviewNotice from "$lib/components/ReviewNotice.svelte";
  import PartyEditor from "$lib/components/PartyEditor.svelte";
  import { ipc } from "$lib/ipc";
  import { session } from "$lib/session.svelte";
  import { emptyProfile, type CaseProfile, type Party } from "$lib/types/case";

  const steps = [
    "Mode",
    "Court",
    "Docket",
    "Caption",
    "Parties",
    "Filer",
    "Document",
  ];

  let step = $state(0);
  let error = $state("");
  let busy = $state(false);
  let kind = $derived($page.url.searchParams.get("kind") || "notice_of_appeal");
  let draft = $state<CaseProfile>(emptyProfile("", "notice_of_appeal"));
  let parties = $state<Party[]>([]);
  let addressText = $state("");
  let mode = $state<"standard" | "confidential">("standard");

  $effect(() => {
    draft.document_kind = kind;
  });

  function applyParties() {
    const left = parties.filter((p) =>
      ["Plaintiff", "Appellant", "Petitioner"].includes(p.role),
    );
    const right = parties.filter((p) =>
      ["Defendant", "Appellee", "Respondent"].includes(p.role),
    );
    const leftovers = parties.filter((p) => !left.includes(p) && !right.includes(p));
    draft.caption.plaintiffs = left.length ? left : leftovers;
    draft.caption.appellants = left;
    draft.caption.defendants = right;
    draft.caption.appellees = right;
    draft.filer.address_lines = addressText.split("\n").map((l) => l.trim()).filter(Boolean);
    draft.mode = mode;
  }

  function next() {
    error = "";
    if (step === 0 && mode === "confidential" && !session.unlocked) {
      error = "Unlock Confidential mode first. Use Unlock Confidential on the home screen, then return here.";
      return;
    }
    if (step === 1 && !draft.court.name.trim()) {
      error = "Enter the court name.";
      return;
    }
    if (step === 2 && !draft.docket_number.trim()) {
      error = "Enter the case or docket number.";
      return;
    }
    if (step === 3 && !draft.title.trim()) {
      error = "Enter a caption or case title, such as Example v. Sample County Clerk.";
      return;
    }
    if (step === 4 && parties.filter((p) => p.name.trim()).length < 2) {
      error = "Add at least two named parties (for example one plaintiff and one defendant).";
      return;
    }
    if (step === 5 && (!draft.filer.name.trim() || !draft.filer.signature_name.trim())) {
      error = "Filer name and signature name are required for the signature block.";
      return;
    }
    if (step < steps.length - 1) {
      step += 1;
    }
  }

  function back() {
    error = "";
    if (step === 0) {
      goto("/");
      return;
    }
    step -= 1;
  }

  async function finish() {
    applyParties();
    if (!draft.title.trim()) {
      error = "Give the case a title.";
      return;
    }
    busy = true;
    error = "";
    try {
      const created = await ipc<CaseProfile>("create_case", {
        title: draft.title.trim(),
        documentKind: draft.document_kind,
        mode,
      });
      draft.id = created.id;
      draft.created_at = created.created_at;
      const saved = await ipc<CaseProfile>("save_case", { caseId: created.id, profile: draft });
      goto(`/case/${saved.id}`);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<main class="container mx-auto max-w-3xl p-8 space-y-6">
  <button onclick={back} class="text-sm text-muted-foreground hover:text-foreground">
    {step === 0 ? "Back to home" : "Back"}
  </button>
  <div>
    <p class="text-xs uppercase tracking-widest text-muted-foreground">Guided setup · step {step + 1} of {steps.length}</p>
    <h1 class="text-2xl font-bold">{steps[step]}</h1>
  </div>
  <div class="flex gap-1">
    {#each steps as label, i}
      <div class="h-1 flex-1 rounded {i <= step ? 'bg-indigo-500' : 'bg-border'}" title={label}></div>
    {/each}
  </div>
  <ReviewNotice />

  {#if step === 0}
    <section class="space-y-3">
      <p class="text-sm text-muted-foreground">Standard cases stay in Documents. Confidential cases also create a vault hunt and need the Airlock plus password.</p>
      <label class="flex gap-3 items-start border rounded-md p-4">
        <input type="radio" bind:group={mode} value="standard" />
        <span>
          <strong class="block">Standard</strong>
          <span class="text-sm text-muted-foreground">Portable folder. You can copy it to a USB drive.</span>
        </span>
      </label>
      <label class="flex gap-3 items-start border rounded-md p-4">
        <input type="radio" bind:group={mode} value="confidential" />
        <span>
          <strong class="block">Confidential</strong>
          <span class="text-sm text-muted-foreground">Vault encryption for exhibits. Sync is refused. Unlock first if you choose this.</span>
        </span>
      </label>
    </section>
  {:else if step === 1}
    <label class="block space-y-1">
      <span class="text-sm font-medium">Court name</span>
      <input bind:value={draft.court.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="United States Court of Appeals for the Sample Circuit" />
    </label>
    <label class="block space-y-1">
      <span class="text-sm font-medium">Division or jurisdiction (optional)</span>
      <input bind:value={draft.court.division} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Civil division" />
    </label>
  {:else if step === 2}
    <label class="block space-y-1">
      <span class="text-sm font-medium">Case or docket number</span>
      <input bind:value={draft.docket_number} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="24-01000" />
    </label>
  {:else if step === 3}
    <label class="block space-y-1">
      <span class="text-sm font-medium">Caption / case title</span>
      <input bind:value={draft.title} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Example v. Sample County Clerk" />
    </label>
  {:else if step === 4}
    <PartyEditor bind:parties />
  {:else if step === 5}
    <div class="grid md:grid-cols-2 gap-3">
      <label class="block space-y-1">
        <span class="text-sm font-medium">Filer name</span>
        <input bind:value={draft.filer.name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
      <label class="block space-y-1">
        <span class="text-sm font-medium">Signature name</span>
        <input bind:value={draft.filer.signature_name} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
    </div>
    <label class="block space-y-1">
      <span class="text-sm font-medium">Filer role</span>
      <input bind:value={draft.filer.role} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" placeholder="Appellant, proceeding without an attorney" />
    </label>
    <label class="block space-y-1">
      <span class="text-sm font-medium">Address (one line per row)</span>
      <textarea bind:value={addressText} class="w-full h-20 rounded-md border border-input bg-background px-3 py-2 text-sm"></textarea>
    </label>
    <div class="grid md:grid-cols-2 gap-3">
      <label class="block space-y-1">
        <span class="text-sm font-medium">Phone</span>
        <input bind:value={draft.filer.phone} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
      <label class="block space-y-1">
        <span class="text-sm font-medium">Email</span>
        <input bind:value={draft.filer.email} class="w-full h-10 rounded-md border border-input bg-background px-3 text-sm" />
      </label>
    </div>
  {:else}
    <p class="text-sm text-muted-foreground">Choose the first document. You can still export the others later from the case dashboard. Complaint and motion are basic templates.</p>
    <label class="flex gap-3 items-start border rounded-md p-4">
      <input type="radio" bind:group={draft.document_kind} value="notice_of_appeal" />
      <span><strong>Notice of Appeal</strong> <span class="text-sm text-muted-foreground">Ready Word and PDF export.</span></span>
    </label>
    <label class="flex gap-3 items-start border rounded-md p-4">
      <input type="radio" bind:group={draft.document_kind} value="complaint" />
      <span><strong>Complaint</strong> <span class="text-sm text-muted-foreground">Basic template.</span></span>
    </label>
    <label class="flex gap-3 items-start border rounded-md p-4">
      <input type="radio" bind:group={draft.document_kind} value="motion" />
      <span><strong>Motion</strong> <span class="text-sm text-muted-foreground">Basic template.</span></span>
    </label>
  {/if}

  {#if error}
    <p class="text-sm text-destructive">{error}</p>
  {/if}

  <div class="flex gap-2">
    {#if step < steps.length - 1}
      <button onclick={next} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm">Continue</button>
    {:else}
      <button onclick={finish} disabled={busy} class="bg-primary text-primary-foreground px-4 py-2 rounded-md text-sm disabled:opacity-50">
        {busy ? "Creating..." : "Create case"}
      </button>
    {/if}
  </div>
</main>
