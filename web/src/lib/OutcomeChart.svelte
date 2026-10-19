<script lang="ts">
  // A stub for W3 to fill (REDESIGN.md §2.6). "מה יהיה לכם ב‑…": one row per option on a shared ₪ axis, the deposits and the current account as reference lines. Reads `app.answer.outcome`; a row's tap calls `app.focusRow(row.id)`.
  import type { AppState } from './app.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="OutcomeChart" hidden>{app.answer?.outcome.title}</div>
