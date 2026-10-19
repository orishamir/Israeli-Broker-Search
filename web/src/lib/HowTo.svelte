<script lang="ts">
  // A stub for W4 to fill (REDESIGN.md §2.1 item 4, §2.8). "איך עושים את זה": the How of `app.focusedRow` (identity line, search, steps or why it isn't offered), with `WhereToBuy` under it; an empty state until a row is chosen.
  import type { AppState } from './app.svelte'
  import WhereToBuy from './WhereToBuy.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="HowTo" hidden><WhereToBuy {app} /></div>
