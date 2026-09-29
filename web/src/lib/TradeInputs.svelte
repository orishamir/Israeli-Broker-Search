<script lang="ts">
  import * as core from './core/core'
  import type { PriceKind, TradeFields } from './core/core'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  /** A trade fee's fields, one per line. `full` adds the maximum, which only
   * matters for very large trades. */
  let {
    fields,
    currency,
    name,
    full = false,
    onchange,
  }: {
    fields: TradeFields
    currency: string
    name: string
    full?: boolean
    onchange: (fields: TradeFields) => void
  } = $props()

  const kinds = core.priceKinds()
  const id = $props.id()
  const percent = $derived(fields.kind === 'Percent' || fields.kind === 'PercentPlusPerShare')
</script>

<div class="fields">
  <label class="label" for="{id}-amount">{t.price}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-amount"
        label="{name}: price"
        prefix={percent ? '' : currency}
        suffix={percent ? '%' : ''}
        step={percent ? 0.01 : 1}
        bind:value={
          () => fields.amount ?? null, (amount) => onchange({ ...fields, amount: amount ?? undefined })
        }
      />
    </div>
    <select
      aria-label="{name}: unit"
      value={fields.kind}
      onchange={(event) =>
        onchange({
          ...fields,
          kind: event.currentTarget.value as PriceKind,
          amount: undefined,
          perShare: undefined,
        })}
    >
      {#each kinds as kind (kind.value)}
        <option value={kind.value}>{kind.name}</option>
      {/each}
    </select>
  </div>
  {#if fields.kind === 'PercentPlusPerShare'}
    <label class="label" for="{id}-per-share">{t.plus}</label>
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-per-share"
          label="{name}: per share"
          prefix={currency}
          step={0.01}
          bind:value={
            () => fields.perShare ?? null,
            (perShare) => onchange({ ...fields, perShare: perShare ?? undefined })
          }
        />
      </div>
      <span>{t.perShare}</span>
    </div>
  {/if}
  {#if fields.kind !== 'PerOrder'}
    <label class="label" for="{id}-min">{t.min}</label>
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-min"
          label="{name}: min"
          prefix={currency}
          bind:value={() => fields.min ?? null, (min) => onchange({ ...fields, min: min ?? undefined })}
        />
      </div>
    </div>
    {#if full}
      <label class="label" for="{id}-max">{t.max}</label>
      <div class="control">
        <div class="number">
          <NumberField
            id="{id}-max"
            label="{name}: max"
            prefix={currency}
            bind:value={() => fields.max ?? null, (max) => onchange({ ...fields, max: max ?? undefined })}
          />
        </div>
      </div>
    {/if}
  {/if}
</div>
