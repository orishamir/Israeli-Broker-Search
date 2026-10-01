<script lang="ts">
  import type { HandlingFields } from './core/core'
  import NumberField from './NumberField.svelte'
  import { t } from './text'

  /** The fixed monthly amount's (handling fee's) fields, one per line. `full` adds whether a month's
   * trade fees are taken off it. */
  let {
    fields,
    full = false,
    onchange,
  }: {
    fields: HandlingFields
    full?: boolean
    onchange: (fields: HandlingFields) => void
  } = $props()

  const id = $props.id()
</script>

<div class="fields">
  <label class="label" for="{id}-amount">{t.fee}</label>
  <div class="control">
    <div class="number">
      <NumberField
        id="{id}-amount"
        label={t.fieldOf(t.handling, t.fee)}
        prefix="₪"
        placeholder={t.none}
        bind:value={
          () => fields.perMonth ?? null,
          (perMonth) => onchange({ ...fields, perMonth: perMonth ?? undefined })
        }
      />
    </div>
    <span>{t.aMonth}</span>
  </div>
  {#if fields.perMonth !== undefined}
    <label class="label" for="{id}-free">{t.freeFor}</label>
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-free"
          label={t.fieldOf(t.handling, t.freeMonths)}
          suffix={t.months}
          bind:value={
            () => fields.freeMonths,
            // Whole months: the core counts them.
            (months) => onchange({ ...fields, freeMonths: Math.max(0, Math.round(months ?? 0)) })
          }
        />
      </div>
      <span>{t.afterOpening}</span>
    </div>
    {#if full}
      <label class="check">
        <input
          type="checkbox"
          checked={fields.lessTradeFees}
          onchange={(event) => onchange({ ...fields, lessTradeFees: event.currentTarget.checked })}
        />
        {t.lessTradeFees}
      </label>
    {/if}
  {/if}
</div>
