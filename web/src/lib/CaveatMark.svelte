<script lang="ts">
  import type { Mark } from './core/core'
  import Sources from './Sources.svelte'
  import { tip } from './tip'
  import { t } from './text'

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
      {#if caveat.support}<p class="support">{t.why} {caveat.support}.</p>{/if}
      <Sources sources={caveat.sources} />
    {/each}
  </div>
{:else}
  <span class="mark" class:warning={mark.kind === 'MayCostMore'}>{mark.text}</span>
{/if}

<style>
  .mark {
    display: inline-block;
    margin-inline-start: 8px;
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
    background: var(--accent-tint);
    color: var(--text);
  }
  .warning {
    border-color: var(--warning-edge);
    color: var(--warning);
  }
  .warning::before {
    content: '⚠ ';
  }
  .support {
    color: var(--weak);
  }
</style>
