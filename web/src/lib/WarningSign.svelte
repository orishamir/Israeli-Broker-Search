<script lang="ts">
  import { tip } from './tip'

  /** A ⚠ that says `text` on hover, focus or tap, beside the best plan's
   * amount: a line of its own changed the card's height, moving the table,
   * and the words are shown outright under the same plan in the table. */
  let { text }: { text: string } = $props()

  const id = $props.id()
  let popover = $state<HTMLElement>()
</script>

<button type="button" class="sign" aria-label={`⚠ ${text}`} {@attach popover && tip(popover)}>⚠</button>
<div {id} class="popover" popover="manual" role="tooltip" bind:this={popover}><p>{text}</p></div>

<style>
  .sign {
    margin-inline-start: 6px;
    padding: 0 5px;
    border: 1px solid color-mix(in srgb, var(--warning) 50%, transparent);
    border-radius: 999px;
    background: transparent;
    color: var(--warning);
    font-size: 0.8rem;
    font-weight: 400;
    line-height: 1.5;
    vertical-align: 0.3em;
  }
  .sign:hover,
  .sign:global([aria-expanded='true']) {
    border-color: var(--warning);
    background: color-mix(in srgb, var(--warning) 15%, transparent);
  }
</style>
