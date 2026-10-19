<script lang="ts">
  // A stub for W5 to fill (REDESIGN.md §2.7). "כמה מהרווח נשאר אצלכם": `app.answer.gain`, shown only when it is there.
  import type { AppState } from './app.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="GainSplit" hidden>{app.answer?.gain?.lead}</div>
