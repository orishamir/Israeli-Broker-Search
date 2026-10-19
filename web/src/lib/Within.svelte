<script lang="ts">
  // STUB (package K fills it): "איפה הכי זול לקנות אותה": one list for both
  // arenas, its columns named by the core (`recommendation.columns`): the top
  // three of `recommendation.plans` with a link to all of them; with the
  // switch on, the full table (the fixed name-column share and the
  // whole-column scroll steps of `ResultsTable`, which this replaces; delete
  // `ResultsTable.svelte` once nothing imports it). Screens s2, s4, s7.
  import type { AppState } from './app.svelte'
  import { shekels } from './format'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

{#if app.recommendation}
  <section class="card within" data-stub="Within">
    <h2>{t.withinTitle}</h2>
    <ul>
      {#each app.recommendation.plans.slice(0, 3) as plan (plan.name)}
        <li>{plan.name} · {shekels(plan.left.today)}</li>
      {/each}
    </ul>
    <p>{t.allPlans(app.recommendation.plans.length)}</p>
  </section>
{/if}
