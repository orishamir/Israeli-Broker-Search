<script lang="ts">
  // Screens s2, s3: "how to do it", the core's steps numbered, each a title
  // and what to do.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

{#if app.recommendation && app.recommendation.steps.length > 0}
  <section class="card steps">
    <h2>{t.stepsTitle}</h2>
    <ol>
      {#each app.recommendation.steps as step (step.title)}
        <li>
          <strong>{step.title}</strong>
          <span>{step.text}</span>
        </li>
      {/each}
    </ol>
  </section>
{/if}

<style>
  h2 {
    margin: 0 0 8px;
    font-size: 1.05rem;
  }
  ol {
    display: grid;
    gap: 10px;
    margin: 0;
    padding-inline-start: 22px;
  }
  li span {
    display: block;
    margin-top: 2px;
    color: var(--weak);
    font-size: 0.92rem;
  }
</style>
