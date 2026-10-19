<script lang="ts">
  // STUB (package K fills it): כל האפשרויות, in the header, persisted
  // (`app.expert`). Turning it on changes only what is shown.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

<label class="expert-switch" data-stub="ExpertSwitch">
  <input type="checkbox" role="switch" bind:checked={app.expert} />
  {t.allOptions}
</label>
