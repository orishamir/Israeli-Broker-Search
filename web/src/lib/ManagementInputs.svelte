<script lang="ts">
  import type { ManagementFields } from './core/core'
  import NumberField from './NumberField.svelte'
  import { feeKind } from './editor'
  import { t } from './text'

  /** A fund's or a policy's fee: a share of the balance a year, and a share
   * of each deposit, one per line. An empty one is none. */
  let {
    fields,
    onchange,
  }: {
    fields: ManagementFields
    onchange: (fields: ManagementFields) => void
  } = $props()

  const id = $props.id()
</script>

<div class="fields">
  <label class="label" for="{id}-balance">{t.ofTheBalance}</label>
  <div class="control">
    <div class="number wide">
      <NumberField
        id="{id}-balance"
        label={feeKind('Management').name}
        suffix="%"
        step={0.05}
        placeholder={t.none}
        bind:value={
          () => fields.ofBalance ?? null,
          (ofBalance) => onchange({ ...fields, ofBalance: ofBalance ?? undefined })
        }
      />
    </div>
    <span>{t.aYear}</span>
  </div>
  <label class="label" for="{id}-deposits">{t.ofEachDeposit}</label>
  <div class="control">
    <div class="number wide">
      <NumberField
        id="{id}-deposits"
        label={feeKind('DepositFee').name}
        suffix="%"
        step={0.5}
        placeholder={t.none}
        bind:value={
          () => fields.ofDeposits ?? null,
          (ofDeposits) => onchange({ ...fields, ofDeposits: ofDeposits ?? undefined })
        }
      />
    </div>
  </div>
</div>
