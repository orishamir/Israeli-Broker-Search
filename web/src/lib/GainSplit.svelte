<script lang="ts">
  // "כמה מהרווח נשאר אצלכם" (REDESIGN.md §2.7): shown only where fees or tax
  // decide something on the page (`app.answer.gain`). The lead gives the
  // deposits, which the bars don't draw; one bar per option on one ₪ scale,
  // the gain only, every part named with its amount under the bar; then
  // the takeaway. All from the core, in today's money.
  import type { AppState } from './app.svelte'
  import { shekels } from './format'
  import { GAIN_COLORS, gainBars, type GainPartId } from './gain'
  import { HeldWhileTyping } from './held.svelte'
  import { glideHeight } from './motion'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const held = new HeldWhileTyping(() => app.answer?.gain)
  const gain = $derived(held.current)
  const bars = $derived(gain ? gainBars(gain.rows) : [])

  const partName: Record<GainPartId, string> = {
    kept: t.gainKept,
    tax: t.gainTax,
    fees: t.gainFees,
    product: t.gainProduct,
  }
</script>

{#if gain}
  <section
    class="card gain"
    class:waiting={!app.answerFresh}
    aria-labelledby="gain-title"
    {@attach glideHeight()}
  >
    <div>
      <h2 id="gain-title">{t.gainTitle}</h2>
      <p class="lead">{gain.lead}</p>
      <ul class="bars">
        {#each bars as bar, at (at)}
          <li>
            <span class="name">{bar.name}</span>
            <span
              class="bar"
              role="img"
              aria-label={t.gainBarLabel(
                bar.name,
                bar.parts.map((part) => `${partName[part.id]} ${shekels(part.value)}`).join(', '),
              )}
            >
              {#each bar.segments as segment (segment.id)}
                <span
                  class="segment {segment.id}"
                  style:--color={GAIN_COLORS[segment.id]}
                  style:inline-size="{segment.width * 100}%"
                ></span>
              {/each}
            </span>
            <span class="parts" aria-hidden="true">
              {#each bar.parts as part (part.id)}
                <span class="part">
                  <span class="swatch {part.id}" style:--color={GAIN_COLORS[part.id]}></span>
                  {partName[part.id]}
                  <bdi class="amount" dir="ltr">{shekels(part.value)}</bdi>
                </span>
              {/each}
            </span>
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

  .waiting {
    opacity: 0.6;
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
    gap: 16px;
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

  .bar {
    display: flex;
    gap: 2px;
    block-size: 20px;
  }

  .segment {
    min-inline-size: 2px;
    background: var(--color);
    transition: inline-size var(--settle);
  }

  .segment:first-child {
    border-start-start-radius: 4px;
    border-end-start-radius: 4px;
  }

  .segment:last-child {
    border-start-end-radius: 4px;
    border-end-end-radius: 4px;
  }

  /* The fund's own cost is a fee too: the fees' hue, hatched. */
  .product {
    background: repeating-linear-gradient(
      45deg,
      var(--color) 0 3px,
      color-mix(in srgb, var(--color) 35%, var(--surface)) 3px 6px
    );
  }

  .parts {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 16px;
    color: var(--weak);
    font-size: 0.85rem;
  }

  .part {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .swatch {
    flex: none;
    inline-size: 12px;
    block-size: 12px;
    border-radius: 3px;
    background: var(--color);
  }

  .amount {
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .takeaway {
    margin: 16px 0 0;
  }

  @media (prefers-reduced-motion: reduce) {
    .segment {
      transition: none;
    }
  }
</style>
