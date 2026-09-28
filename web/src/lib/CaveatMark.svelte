<script lang="ts">
  import type { Mark } from './core/core'
  import { tip } from './tip'

  /** How sure a fee's number is, in a word or two beside its price, with the
   * caveats that say why on hover or tap. Without `tips` (in the hover
   * preview, which can't be hovered into) it's plain text. */
  let { mark, tips = true }: { mark: Mark; tips?: boolean } = $props()

  const id = $props.id()
  let popover = $state<HTMLElement>()
</script>

{#if tips}
  <button
    type="button"
    class="mark"
    class:warning={mark.kind === 'MayCostMore'}
    aria-describedby={id}
    {@attach popover && tip(popover)}>{mark.text}</button
  >
  <div {id} class="popover" popover="manual" role="tooltip" bind:this={popover}>
    {#each mark.caveats as caveat (caveat.text)}
      <p>{caveat.text}</p>
      {#if caveat.support}<p class="support">Why: {caveat.support}.</p>{/if}
    {/each}
  </div>
{:else}
  <span class="mark" class:warning={mark.kind === 'MayCostMore'}>{mark.text}</span>
{/if}

<style>
  .mark {
    display: inline-block;
    margin-left: 8px;
    padding: 0 7px;
    border: 1px solid var(--strong-border);
    border-radius: 999px;
    background: transparent;
    color: var(--weak);
    font-size: 0.75rem;
    font-style: normal;
    line-height: 1.5;
    vertical-align: 1px;
  }
  button.mark:hover,
  button.mark:global([aria-expanded='true']) {
    border-color: var(--accent);
    background: rgb(123 155 255 / 0.15);
    color: var(--text);
  }
  .warning {
    border-color: color-mix(in srgb, var(--warning) 50%, transparent);
    color: var(--warning);
  }
  .warning::before {
    content: '⚠ ';
  }
  .support {
    color: var(--weak);
  }
</style>
