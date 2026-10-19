<script lang="ts">
  // STUB (package K fills it): the workbench, shown while `app.expert`: what
  // to buy (`app.thing`/`chooseThing`), the ways (`app.wayChoices`,
  // `selectedWays`, `toggleWay`), the levers (security, exchange, rates, unit
  // price, expectations, buy every, growing deposits, keep/sell, pension),
  // "מה משווים" with `PlanSheet` (`app.planSheetOpen`) and the editor, the
  // charts' tabs (`app.chartView`; growth, lost to fees, breakdown, by
  // deposit; the existing chart components take `app.comparison` /
  // `app.waysComparison` as before). Screens s4, s7.
  import type { AppState } from './app.svelte'
  import { t } from './text'

  let { app }: { app: AppState } = $props()
</script>

<section class="card workbench" data-stub="Workbench">
  <h2>{t.whatToBuy}</h2>
  <p>
    {#each [['sp500', t.thingSp500], ['other', t.thingOther], ['bonds', t.thingBonds], ['short', t.thingShort]] as const as [thing, name] (thing)}
      <button class:chosen={app.thing === thing} onclick={() => app.chooseThing(thing)}>{name}</button>
    {/each}
  </p>
</section>

<style>
  .chosen {
    border-color: var(--accent);
  }
</style>
