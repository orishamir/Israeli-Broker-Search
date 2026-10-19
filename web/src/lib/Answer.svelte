<script lang="ts">
  // STUB (package I fills it): "מה שמתאים לכסף כזה": the what-sentence with
  // "מה זה? ›" (`app.openCard(kind.card)`) and "לשנות ›" (`app.chooseKind`),
  // the amount (today's money; the nominal figure as a note), the
  // current-account line, the reasons, the unlock line, the two-part cushion
  // plan, the bank chips of the existing-portfolio situation. Screens s2, s3,
  // s6. While `app.stale` the last answer stays, faded, under the message.
  import type { AppState } from './app.svelte'
  import { shekels } from './format'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
  const recommendation = $derived(app.recommendation)
</script>

<section class="card answer" data-stub="Answer">
  <h2>{t.answerTitle}</h2>
  {#if recommendation}
    <p>
      <strong>{recommendation.kind.name}</strong>
      · {recommendation.choice.provider}
      · {shekels(recommendation.left.today)}
    </p>
    <ul>
      {#each recommendation.reasons as reason (reason.text)}
        <li>{reason.text}</li>
      {/each}
    </ul>
  {/if}
</section>
