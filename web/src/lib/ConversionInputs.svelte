<script lang="ts">
  import type { ConversionFields, MarkupFields } from './core/core'
  import * as core from './core/core'
  import HebrewNames from './HebrewNames.svelte'
  import NumberField from './NumberField.svelte'
  import Tip from './Tip.svelte'

  /** Conversion's fields, one per line: the listed fee, and the markup, the
   * other half of what converting costs. `full` adds the maximum. */
  let {
    fields,
    markup,
    currency,
    full = false,
    onchange,
    onmarkupchange,
  }: {
    fields: ConversionFields
    markup: MarkupFields
    currency: string
    full?: boolean
    onchange: (fields: ConversionFields) => void
    onmarkupchange: (fields: MarkupFields) => void
  } = $props()

  const markupKind = core.feeKinds().find((kind) => kind.value === 'Markup')!
  const id = $props.id()
</script>

<div class="fields">
  <label class="label" for="{id}-percent">Fee</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-percent"
        label="Conversion: fee"
        suffix="%"
        step={0.01}
        bind:value={
          () => fields.percent ?? null, (percent) => onchange({ ...fields, percent: percent ?? undefined })
        }
      />
    </div>
  </div>
  <label class="label" for="{id}-min">Min</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-min"
        label="Conversion: min"
        prefix={currency}
        bind:value={() => fields.min ?? null, (min) => onchange({ ...fields, min: min ?? undefined })}
      />
    </div>
  </div>
  {#if full}
    <label class="label" for="{id}-max">Max</label>
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-max"
          label="Conversion: max"
          prefix={currency}
          bind:value={() => fields.max ?? null, (max) => onchange({ ...fields, max: max ?? undefined })}
        />
      </div>
    </div>
  {/if}
  <span class="label"
    ><label for="{id}-markup">Markup</label><Tip about={markupKind.name}>
      <HebrewNames names={markupKind.hebrewNames} />
      <p>{markupKind.explanation}</p>
    </Tip></span
  >
  <div class="control">
    <div class="number wide">
      <NumberField
        id="{id}-markup"
        label="Conversion: markup"
        suffix="%"
        step={0.01}
        placeholder="not published"
        bind:value={
          () => markup.percent ?? null, (percent) => onmarkupchange({ percent: percent ?? undefined })
        }
      />
    </div>
    {#if markup.percent === undefined}<span>counted as 0</span>{/if}
  </div>
</div>
