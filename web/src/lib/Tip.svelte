<script lang="ts">
  import { getContext, type Snippet } from 'svelte'
  import { OPEN_CARD, type OpenCard } from './cards'
  import type { Term } from './core/core'
  import { tip } from './tip'
  import { t } from './text'

  /** A small "?" that explains `about`, on hover, focus or tap. `card`: the
   * term's learn card, if the core says it has one, which "עוד על זה ›"
   * opens. */
  let { about, card, children }: { about: string; card?: Term; children: Snippet } = $props()
  const openCard = getContext<OpenCard | undefined>(OPEN_CARD)

  const id = $props.id()
  let popover = $state<HTMLElement>()
</script>

<button
  type="button"
  class="tip-button"
  aria-label={t.whatMeans(about)}
  aria-describedby={id}
  {@attach popover && tip(popover)}>?</button
>
<div {id} class="popover" popover="manual" role="tooltip" bind:this={popover}>
  {@render children()}
  {#if card && openCard}
    <!-- A button, since it acts on the page; tip.ts keeps the tip open while the pointer goes to it. -->
    <button type="button" class="more" onclick={() => openCard(card)}>{t.learnMore}</button>
  {/if}
</div>

<style>
  .tip-button {
    display: inline-grid;
    place-items: center;
    width: 16px;
    height: 16px;
    margin-inline-start: 5px;
    padding: 0;
    border: 1px solid var(--strong-border);
    border-radius: 50%;
    background: transparent;
    color: var(--weak);
    font-size: 0.65rem;
    font-weight: 700;
    line-height: 1;
    vertical-align: 1px;
    /* A bigger target for fingers, without taking more room. */
    position: relative;
  }
  .tip-button::after {
    content: '';
    position: absolute;
    inset: -10px;
  }
  .tip-button:hover,
  .tip-button:global([aria-expanded='true']) {
    border-color: var(--accent);
    background: var(--accent-tint);
    color: var(--text);
  }
  .more {
    display: block;
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    font-size: inherit;
  }
  .more:hover {
    background: none;
    text-decoration: underline;
  }
</style>
