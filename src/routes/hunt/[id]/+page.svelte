<script lang="ts">
  import { page } from '$app/stores';
  import HuntWizard from "$lib/components/HuntWizard.svelte";
  
  let { id } = $props(); // Svelte 5 params? Or use page store
  // SvelteKit +page.svelte gets data/form props. 
  // For static/SPA mode in Tauri, we rely on page parameters if adapted.
  // Actually, simplified: Just show "Hunt Details" here.
  
  let huntId = $state($page.params.id);
</script>

<div class="h-screen bg-background">
  <HuntWizard 
    huntId={huntId} 
    onComplete={() => alert("Setup Complete! (Redirect to dashboard or detail view)")} 
    onCancel={() => history.back()} 
  />
</div>
