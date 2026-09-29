<script lang="ts">
  import * as core from './core/core'
  import type { CustodyFields, Period } from './core/core'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  /** The share-of-holdings (custody) fields, one per line. `full` lets the billing period change;
   * otherwise it's only named, after the minimum it applies to. */
  let {
    fields,
    currency,
    full = false,
    onchange,
  }: {
    fields: CustodyFields
    currency: string
    full?: boolean
    onchange: (fields: CustodyFields) => void
  } = $props()

  const periods = core.periods()
  const id = $props.id()
  const billed = $derived(periods.find((period) => period.value === fields.billed))
</script>

<div class="fields">
  <label class="label" for="{id}-percent">{t.rate}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-percent"
        label="Share of holdings: rate"
        suffix="%"
        step={0.01}
        bind:value={
          () => fields.percent ?? null, (percent) => onchange({ ...fields, percent: percent ?? undefined })
        }
      />
    </div>
    <select
      aria-label="Share of holdings: period"
      value={fields.per}
      onchange={(event) => onchange({ ...fields, per: event.currentTarget.value as Period })}
    >
      {#each periods as period (period.value)}
        <option value={period.value}>{period.each}</option>
      {/each}
    </select>
  </div>
  <label class="label" for="{id}-min">{t.min}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-min"
        label="Share of holdings: min"
        prefix={currency}
        bind:value={() => fields.min ?? null, (min) => onchange({ ...fields, min: min ?? undefined })}
      />
    </div>
    <!-- The minimum is per charge, so it's named with the billing period. -->
    <span>{billed?.each}</span>
  </div>
  {#if full}
    <label class="label" for="{id}-billed">{t.charged}</label>
    <div class="control">
      <select
        id="{id}-billed"
        value={fields.billed}
        onchange={(event) => onchange({ ...fields, billed: event.currentTarget.value as Period })}
      >
        {#each periods as period (period.value)}
          <option value={period.value}>{period.adverb}</option>
        {/each}
      </select>
    </div>
  {/if}
</div>
