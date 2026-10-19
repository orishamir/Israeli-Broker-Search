<script lang="ts">
  // STUB (package I fills it): "איך עושים את זה": `recommendation.steps`,
  // numbered, each a title and a text from the core. Screens s2, s3.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

{#if app.recommendation}
  <section class="card steps" data-stub="Steps">
    <h2>{t.stepsTitle}</h2>
    <ol>
      {#each app.recommendation.steps as step (step.title)}
        <li><strong>{step.title}</strong> {step.text}</li>
      {/each}
    </ol>
  </section>
{/if}
