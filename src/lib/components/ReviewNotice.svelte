<script lang="ts">
  import { REVIEW_NOTICE } from "$lib/types/case";
</script>

<div class="rounded-lg border border-amber-500/30 bg-amber-500/10 p-4 text-xs text-amber-100/90 leading-relaxed space-y-2">
  <div class="flex items-center gap-2 text-amber-200 font-medium">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-amber-300 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
    <span>Important Administrative & Legal Advisory</span>
  </div>
  <p>{REVIEW_NOTICE}</p>
  <div class="pt-1 flex flex-wrap items-center gap-3 text-[11px] text-amber-200/80">
    <span>Need help with your case?</span>
    <a href="https://justlegal.me/resources" target="_blank" rel="noopener noreferrer" class="underline hover:text-amber-100 font-medium">
      Find your State Bar Lawyer Referral & Unbundled Legal Aid &rarr;
    </a>
  </div>
</div>
