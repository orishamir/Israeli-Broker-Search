<script module lang="ts">
  import type { Plan } from './app.svelte'

  const WIDTH = 360
  const GAP = 16

  /** A plan under the mouse, and its row, which the preview opens beside. */
  export type Preview = { plan: Plan; row: HTMLElement }

  /** None when the window has no room beside the row (one column): over the
   * list, it would hide the plans the mouse goes to next. */
  export function previewBeside(plan: Plan, row: HTMLElement): Preview | null {
    // Left of the row, where the results are on a right-to-left page.
    const room = row.getBoundingClientRect().left
    return room < GAP + WIDTH ? null : { plan, row }
  }
</script>

<script lang="ts">
  import { autoUpdate, computePosition, offset, shift } from '@floating-ui/dom'
  import type { Attachment } from 'svelte/attachments'
  import { fade } from 'svelte/transition'
  import type { AppState } from './app.svelte'
  import FeesForInputs from './FeesForInputs.svelte'
  import { duration } from './motion'
  import { t } from './text'

  let { app, preview }: { app: AppState; preview: Preview | null } = $props()

  // Beside the row, on the side of the results (left of it), its first line
  // level with the row's, or moved up just enough to stay in the window,
  // also as its fees change.
  const beside =
    (row: HTMLElement): Attachment<HTMLElement> =>
    (element) =>
      autoUpdate(row, element, () =>
        computePosition(row, element, {
          strategy: 'fixed',
          placement: 'left-start',
          middleware: [offset({ mainAxis: GAP, crossAxis: -8 }), shift({ padding: 8 })],
        }).then(({ x, y }) => {
          element.style.left = `${x}px`
          element.style.top = `${y}px`
        }),
      )
</script>

<!-- Shown after a moment of hovering; it glides from plan to plan. Not on
     touch screens, where there's no hovering, nor in one column
     (`previewBeside`). -->
{#if preview && !app.details}
  {@const plan = preview.plan}
  <div
    class="popover preview"
    style:width="{WIDTH}px"
    style:--plan-color={plan.color}
    {@attach beside(preview.row)}
    in:fade={{ delay: 350, duration: duration(150) }}
    out:fade={{ duration: duration(100) }}
  >
    <strong>{plan.info.name}</strong>
    <span class="broker-of">{plan.subtitle}</span>
    {#if plan.info.description}<p>{plan.info.description}</p>{/if}
    <FeesForInputs {app} {plan} tips={false} />
    <p class="hint">
      {plan.yours ? t.previewHintYours : t.previewHint}
    </p>
  </div>
{/if}

<style>
  .preview {
    position: fixed;
    z-index: 10;
    /* `WIDTH` wide, not a tip's width: `previewBeside` checks there's room. */
    max-width: none;
    border-inline-start: 3px solid var(--plan-color);
    pointer-events: none;
    transition: top 180ms ease-out;
  }
  @media (hover: none) {
    .preview {
      display: none;
    }
  }
  .broker-of {
    margin-inline-start: 6px;
    color: var(--weak);
    font-size: 0.8rem;
  }
  .preview p {
    margin: 6px 0 10px;
  }
  .preview .hint {
    margin: 10px 0 0;
    font-size: 0.8rem;
    color: var(--weak);
  }
</style>
