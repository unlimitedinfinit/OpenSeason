<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    const id = $page.params.id;
    if (id) {
      goto(`/case/${id}`);
    } else {
      goto("/");
    }
  });
</script>

<main class="container mx-auto p-8">
  <p class="text-sm text-muted-foreground">Opening this case on the unified dashboard...</p>
</main>
