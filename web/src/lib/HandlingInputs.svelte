<script lang="ts">
  import type { HandlingFields } from './core/core'
  import NumberField from './NumberField.svelte'

  /** The handling fee's fields, one per line. `full` adds whether a month's
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
  <label class="label" for="{id}-amount">Fee</label>
  <div class="control">
    <div class="number wide">
      <NumberField
        id="{id}-amount"
        label="Handling fee: a month"
        prefix="₪"
        placeholder="none"
        bind:value={
          () => fields.perMonth ?? null,
          (perMonth) => onchange({ ...fields, perMonth: perMonth ?? undefined })
        }
      />
    </div>
    <span>a month</span>
  </div>
  {#if fields.perMonth !== undefined}
    <label class="label" for="{id}-free">Free for</label>
    <div class="control">
      <div class="number">
        <NumberField
          id="{id}-free"
          label="Handling fee: free months"
          suffix="months"
          bind:value={
            () => fields.freeMonths,
            // Whole months: the core counts them.
            (months) => onchange({ ...fields, freeMonths: Math.max(0, Math.round(months ?? 0)) })
          }
        />
      </div>
      <span>after opening</span>
    </div>
    {#if full}
      <label class="check">
        <input
          type="checkbox"
          checked={fields.lessTradeFees}
          onchange={(event) => onchange({ ...fields, lessTradeFees: event.currentTarget.checked })}
        />
        Less that month's trade fees
      </label>
    {/if}
  {/if}
</div>
