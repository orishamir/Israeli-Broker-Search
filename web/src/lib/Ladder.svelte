<script lang="ts">
  // STUB (package J fills it): the rungs (`app.ladder`: the same money
  // needed after 1, 3, 5, 10, 20 years, or any day); tapping a rung sets the
  // horizon (`app.setHorizonMonths(rung.months)`). Screen s6.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

{#if app.ladder}
  <section class="card ladder" data-stub="Ladder">
    <h2>{t.ladderTitle}</h2>
    <ul>
      {#each app.ladder as rung (`${rung.months}-${rung.anyDay}`)}
        <li><button onclick={() => app.setHorizonMonths(rung.months)}>{rung.kind.name}</button></li>
      {/each}
    </ul>
  </section>
{/if}
