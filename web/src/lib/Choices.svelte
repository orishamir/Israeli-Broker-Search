<script lang="ts" generics="T extends string">
  import type { Choice } from './core/core'
  import HebrewNames from './HebrewNames.svelte'
  import { tip } from './tip'

  /** Radio buttons shown as a segmented control. Hovering a choice explains
   * it. Not tapping, which picks it: the page shows the picked one's
   * explanation where it matters. */
  let { label, options, value = $bindable() }: { label: string; options: Choice<T>[]; value: T } = $props()

  const popovers = $state<HTMLElement[]>([])
</script>

<div class="choices" role="radiogroup" aria-label={label}>
  {#each options as option, index (option.value)}
    {@const popover = popovers[index]}
    <label {@attach popover && tip(popover, { onClick: false })}>
      <input type="radio" bind:group={value} value={option.value} />{option.name}
    </label>
  {/each}
</div>
{#each options as option, index (option.value)}
  {#if option.explanation}
    <div class="popover" popover="manual" role="tooltip" bind:this={popovers[index]}>
      <p><strong>{option.name}</strong></p>
      <HebrewNames names={option.hebrewNames} />
      <p>{option.explanation}</p>
    </div>
  {/if}
{/each}
