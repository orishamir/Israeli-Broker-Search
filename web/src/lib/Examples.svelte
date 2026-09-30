<script lang="ts" generics="T extends { name: string; explanation: string }">
  import { tip } from './tip'

  /** Ready-made patterns: one click fills the inputs they set. */
  let { examples, apply }: { examples: T[]; apply: (example: T) => void } = $props()

  const popovers = $state<HTMLElement[]>([])
</script>

<div class="examples">
  {#each examples as example, index (example.name)}
    {@const popover = popovers[index]}
    <button
      type="button"
      class="chip"
      onclick={() => apply(example)}
      {@attach popover && tip(popover, { onClick: false })}>{example.name}</button
    >
  {/each}
</div>
{#each examples as example, index (example.name)}
  <div class="popover" popover="manual" role="tooltip" bind:this={popovers[index]}>
    <p><strong>{example.name}</strong></p>
    <p>{example.explanation}</p>
  </div>
{/each}

<style>
  .examples {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .chip {
    padding: 3px 10px;
    border-color: var(--strong-border);
    border-radius: 999px;
    background: transparent;
    color: var(--text);
    font-size: 0.85rem;
  }
  .chip:hover {
    border-color: var(--weak);
    background: var(--raised);
  }
</style>
