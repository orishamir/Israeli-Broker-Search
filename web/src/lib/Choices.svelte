<script lang="ts" generics="T extends string">
  import type { Attachment } from 'svelte/attachments'
  import type { Choice } from './core/core'
  import HebrewNames from './HebrewNames.svelte'
  import { tip } from './tip'

  /** Radio buttons shown as a segmented control. Hovering a choice explains
   * it. Not tapping, which picks it: the page shows the picked one's
   * explanation where it matters. */
  let { label, options, value = $bindable() }: { label: string; options: Choice<T>[]; value: T } = $props()

  const popovers = $state<HTMLElement[]>([])

  /** Keeps the highlight under the chosen button (see `.highlight` in
   * app.css). It glides there when the choice changes, and jumps when the
   * buttons move, as when they wrap onto two rows. */
  const highlight: Attachment<HTMLElement> = (group) => {
    const place = () => {
      const chosen = group.querySelector<HTMLElement>('label:has(input:checked)')
      if (!chosen) return
      group.style.setProperty('--x', `${chosen.offsetLeft}px`)
      group.style.setProperty('--y', `${chosen.offsetTop}px`)
      group.style.setProperty('--w', `${chosen.offsetWidth}px`)
      group.style.setProperty('--h', `${chosen.offsetHeight}px`)
    }
    $effect(() => {
      void value // Once the chosen button is in the DOM.
      place()
    })
    const resizing = new ResizeObserver(() => {
      group.style.setProperty('--glide', '0ms')
      place()
      // From the next frame on (and after the first placement), it glides.
      requestAnimationFrame(() => group.style.setProperty('--glide', '220ms'))
    })
    resizing.observe(group)
    return () => resizing.disconnect()
  }
</script>

<div class="choices" role="radiogroup" aria-label={label} {@attach highlight}>
  <div class="highlight" aria-hidden="true"></div>
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
