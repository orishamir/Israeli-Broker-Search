<script lang="ts">
  // "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7, amended by §4.7): shown only
  // where fees or tax decide something on the page (`app.answer.gain`). The
  // lead gives the deposits, which the bars don't draw; one bar per option
  // on one ₪ scale, the gain only, every part named with its amount on it or
  // beside it (`gain.ts`), "הקרן עצמה" in a colour of its own; then the
  // sentence saying where the difference mostly comes from. All from the
  // core, in today's money.
  import type { AppState } from './app.svelte'
  import { GAIN_COLORS, GAIN_LANE, gainBars } from './gain'
  import { HeldWhileTyping } from './held.svelte'
  import { glideHeight } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.answer?.gain)
  const gain = $derived(held.current)

  /** The bars' width, px. */
  let width = $state(0)
  let list: HTMLElement | null = $state(null)
  $effect(() => {
    if (!list) return
    const observer = new ResizeObserver(([entry]) => {
      const next = Math.round(entry.contentRect.width)
      if (next > 0 && next !== width) width = next
    })
    observer.observe(list)
    return () => observer.disconnect()
  })

  const FONT_PX = 12
  let context: CanvasRenderingContext2D | null = null
  const measure = (text: string): number => {
    context ??= document.createElement('canvas').getContext('2d')
    if (!context) return text.length * 7
    context.font = `600 ${FONT_PX}px ${getComputedStyle(document.documentElement).fontFamily}`
    return Math.ceil(context.measureText(text).width) + 6
  }

  const bars = $derived(gain ? gainBars(gain.rows, width || 400, measure) : [])
  const BAR = 22
  const share = (fraction: number) => `${(fraction * 100).toFixed(3)}%`
</script>

{#if gain}
  <section class="card gain" aria-labelledby="gain-title" {@attach glideHeight()}>
    <div>
      <h2 id="gain-title">{t.gainTitle}</h2>
      <p class="lead">{gain.lead}</p>
      <ul class="bars" bind:this={list}>
        {#each bars as bar, at (at)}
          <li>
            <span class="name"
              >{bar.name}{#if bar.note}&nbsp;<span class="note">{bar.note}</span>{/if}</span
            >
            <span
              class="track"
              role="img"
              aria-label={t.gainBarLabel(
                bar.name,
                [...bar.segments.map(({ text }) => text), ...bar.undrawn].join(', '),
              )}
              style:block-size="{(bar.above + bar.below) * GAIN_LANE + BAR}px"
            >
              {#each bar.segments as segment (segment.which)}
                <span
                  class="segment"
                  aria-hidden="true"
                  style:--color={GAIN_COLORS[segment.which]}
                  style:inset-block-start="{bar.above * GAIN_LANE}px"
                  style:inset-inline-start={share(segment.from)}
                  style:inline-size={share(segment.width)}
                  >{#if segment.inside}<span class="words">{segment.text}</span>{/if}</span
                >
              {/each}
              {#each bar.beside as label (label.key)}
                <span
                  class="beside"
                  aria-hidden="true"
                  style:--color={GAIN_COLORS[label.which]}
                  style:inset-block-start="{label.lane === 0
                    ? 0
                    : bar.above * GAIN_LANE + BAR + (label.lane - 1) * GAIN_LANE + 2}px"
                  style:inset-inline-start="{label.x}px"
                  style:inline-size="{label.width}px">{label.text}</span
                >
              {/each}
            </span>
            {#if bar.undrawn.length > 0}
              <span class="undrawn">{bar.undrawn.join(' · ')}</span>
            {/if}
          </li>
        {/each}
      </ul>
      <p class="takeaway">{gain.takeaway}</p>
    </div>
  </section>
{/if}

<style>
  .gain {
    transition: opacity var(--quick);
  }

  h2 {
    margin-block-end: 4px;
  }

  .lead {
    margin: 0 0 12px;
    color: var(--weak);
  }

  .bars {
    display: grid;
    gap: 14px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    gap: 6px;
  }

  .name {
    font-weight: 600;
  }

  .note {
    color: var(--weak);
    font-size: 0.85rem;
    font-weight: 400;
  }

  .track {
    position: relative;
    display: block;
  }

  /* Parts are 2 px apart: each loses a pixel on either side. */
  .segment {
    position: absolute;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    block-size: 22px;
    min-inline-size: 2px;
    border-inline: 1px solid var(--surface);
    background: var(--color);
    transition:
      inline-size var(--settle),
      inset-inline-start var(--settle);
  }

  .segment:first-child {
    border-start-start-radius: 4px;
    border-end-start-radius: 4px;
  }

  .words {
    color: #0d0f14;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
  }

  /* A part too narrow for its words: they sit beside it, with a short
     stroke of its colour under them, so they are read as its. */
  .beside {
    position: absolute;
    box-sizing: border-box;
    color: var(--text);
    font-size: 12px;
    font-weight: 600;
    line-height: 16px;
    text-align: center;
    white-space: nowrap;
    border-block-end: 2px solid var(--color);
    transition: inset-inline-start var(--settle);
  }

  .undrawn {
    color: var(--weak);
    font-size: 0.85rem;
  }

  .takeaway {
    margin: 16px 0 0;
  }

  @media (prefers-reduced-motion: reduce) {
    .segment,
    .beside {
      transition: none;
    }
  }
</style>
