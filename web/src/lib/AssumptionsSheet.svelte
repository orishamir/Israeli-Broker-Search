<script lang="ts">
  import type { AppState } from './app.svelte'
  import NumberField from './NumberField.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  /** The three assumptions behind every number, the same state as the
   * workbench's fields: a change here shows there, and the other way. */
  let { app, open = $bindable(false) }: { app: AppState; open: boolean } = $props()
</script>

<Sheet bind:open title={t.assumptionsTitle} done={t.done}>
  <section>
    <div class="fields">
      <span class="name">
        <label for="assume-return">{t.yearlyReturn}</label><Tip about={t.yearlyReturn}
          >{t.yearlyReturnTip}</Tip
        >
      </span>
      <NumberField id="assume-return" suffix="%" step={0.5} bind:value={app.yearlyReturnPercent} />

      <span class="name">
        <label for="assume-inflation">{t.inflation}</label><Tip about={t.inflation}>{t.inflationTip}</Tip>
      </span>
      <NumberField
        id="assume-inflation"
        suffix={t.percentAYear}
        step={0.5}
        bind:value={app.inflationPercent}
      />

      <span class="name">
        <label for="assume-boi">{t.boiRate}</label><Tip about={t.boiRate}>{t.boiRateTip}</Tip>
      </span>
      <NumberField id="assume-boi" suffix={t.percentAYear} step={0.25} bind:value={app.boiRatePercent} />
    </div>
  </section>
</Sheet>

<style>
  .fields {
    display: grid;
    grid-template-columns: minmax(0, 1fr) var(--input-field);
    gap: 10px 12px;
    align-items: center;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
