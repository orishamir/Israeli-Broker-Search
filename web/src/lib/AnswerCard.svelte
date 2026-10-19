<script lang="ts">
  // A stub for W2 to fill (REDESIGN.md §2.2–§2.4). The answer card: the chip and its note, the title and body, the four lines (each stated, a decision, or waiting), and the plan or "N decisions left". Reads `app.answer` (confidence, title, body, lines, plan, openDecisions); a decision's option calls `app.choose(decision.id, option.pick)`.
  import type { AppState } from './app.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="AnswerCard" hidden>{app.answer?.title}</div>
