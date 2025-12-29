<script lang="ts">
  import "../app.css";
  let { children } = $props();
</script>

<div class="dark min-h-screen bg-background text-foreground font-sans antialiased">
  {@render children()}
</div>
