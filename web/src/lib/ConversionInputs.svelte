<script lang="ts">
  import type { ConversionFields, MarkupFields } from './core/core'
  import { feeKind } from './editor'
  import HebrewNames from './HebrewNames.svelte'
  import NumberField from './NumberField.svelte'
  import Tip from './Tip.svelte'
  import { t } from './text'

  /** Conversion's fields, one per line: the listed fee, and the markup, the
   * other half of what converting costs. `full` adds the maximum. A second
   * conversion fee has no markup of its own: it shares the first's. `name`
   * starts each field's accessible name: "המרת מט״ח: מינימום". */
  let {
    fields,
    markup,
    currency,
    name = feeKind('Conversion').name,
    full = false,
    onchange,
    onmarkupchange,
  }: {
    fields: ConversionFields
    markup?: MarkupFields
    currency: string
    name?: string
    full?: boolean
    onchange: (fields: ConversionFields) => void
    onmarkupchange?: (fields: MarkupFields) => void
  } = $props()

  const markupKind = feeKind('Markup')
  const id = $props.id()
</script>

<div class="fields">
  <label class="label" for="{id}-percent">{t.fee}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-percent"
        label={t.fieldOf(name, t.fee)}
        suffix="%"
        step={0.01}
        bind:value={
          () => fields.percent ?? null, (percent) => onchange({ ...fields, percent: percent ?? undefined })
        }
      />
    </div>
  </div>
  <label class="label" for="{id}-min">{t.min}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-min"
        label={t.fieldOf(name, t.min)}
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
          label={t.fieldOf(name, t.max)}
          prefix={currency}
          bind:value={() => fields.max ?? null, (max) => onchange({ ...fields, max: max ?? undefined })}
        />
      </div>
    </div>
  {/if}
  {#if markup && onmarkupchange}
    <span class="label"
      ><label for="{id}-markup">{t.markup}</label><Tip about={markupKind.name}>
        <HebrewNames names={markupKind.hebrewNames} />
        <p>{markupKind.explanation}</p>
      </Tip></span
    >
    <!-- A percentage, or shekels for every dollar (Excellence's 2 agorot). -->
    {@const perDollar = markup.perDollar !== undefined}
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-markup"
          label={markupKind.name}
          prefix={perDollar ? '₪' : ''}
          suffix={perDollar ? '' : '%'}
          step={0.01}
          placeholder={t.notPublished}
          bind:value={
            () => (perDollar ? markup.perDollar : markup.percent) ?? null,
            (amount) =>
              onmarkupchange(
                perDollar
                  ? { percent: undefined, perDollar: amount ?? undefined }
                  : { percent: amount ?? undefined },
              )
          }
        />
      </div>
      <select
        aria-label={t.fieldOf(markupKind.name, t.unit)}
        value={perDollar ? 'perDollar' : 'percent'}
        onchange={(event) =>
          onmarkupchange(
            event.currentTarget.value === 'perDollar'
              ? { percent: undefined, perDollar: 0 }
              : { percent: undefined },
          )}
      >
        <option value="percent">{t.markupPercent}</option>
        <option value="perDollar">{t.markupPerDollar}</option>
      </select>
      {#if !perDollar && markup.percent === undefined}<span>{t.countedAs0}</span>{/if}
    </div>
  {/if}
</div>
