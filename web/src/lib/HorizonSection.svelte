<script lang="ts">
  // A stub for W5 to fill (REDESIGN.md §2.9). "ואם התאריך יזוז?": `app.answer.horizons`, a row per horizon (two where the date's answer is open and they differ).
  import type { AppState } from './app.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="HorizonSection" hidden>{app.answer?.horizons.length}</div>
