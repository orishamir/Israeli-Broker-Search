<script lang="ts">
  import type { Snippet } from 'svelte'
  import { tip } from './tip'

  /** A small "?" that explains `about`, on hover, focus or tap. */
  let { about, children }: { about: string; children: Snippet } = $props()

  const id = $props.id()
  let popover = $state<HTMLElement>()
</script>

<button
  type="button"
  class="tip-button"
  aria-label="What “{about}” means"
  aria-describedby={id}
  {@attach popover && tip(popover)}>?</button
>
<div {id} class="popover" popover="manual" role="tooltip" bind:this={popover}>{@render children()}</div>

<style>
  .tip-button {
    display: inline-grid;
    place-items: center;
    width: 16px;
    height: 16px;
    margin-left: 5px;
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
    background: rgb(123 155 255 / 0.15);
    color: var(--text);
  }
</style>
