<script module lang="ts">
  import type { Plan } from './app.svelte'

  /** A plan under the mouse, and where to show its preview: beside its row,
   * going down from it, or up from it when it's low in the window. */
  export type Preview = { plan: Plan; left: number; top?: number; bottom?: number }

  export function previewBeside(plan: Plan, row: HTMLElement): Preview {
    const { top, bottom, right } = row.getBoundingClientRect()
    const left = right + 16
    return top < innerHeight / 2
      ? { plan, left, top: top - 8 }
      : { plan, left, bottom: innerHeight - bottom - 8 }
  }
</script>

<script lang="ts">
  import { fade } from 'svelte/transition'
  import type { AppState } from './app.svelte'
  import FeesForInputs from './FeesForInputs.svelte'
  import { duration } from './motion'

  let { app, preview }: { app: AppState; preview: Preview | null } = $props()
</script>

<!-- Shown after a moment of hovering; it glides from plan to plan. Not on
     touch screens, where there's no hovering. -->
{#if preview && !app.details}
  {@const plan = preview.plan}
  <div
    class="popover preview"
    style:top={preview.top === undefined ? undefined : `${preview.top}px`}
    style:bottom={preview.bottom === undefined ? undefined : `${preview.bottom}px`}
    style:left="{preview.left}px"
    style:--plan-color={plan.color}
    in:fade={{ delay: 350, duration: duration(150) }}
    out:fade={{ duration: duration(100) }}
  >
    <strong>{plan.info.name}</strong>
    <span class="broker-of">{plan.subtitle}</span>
    {#if plan.info.description}<p>{plan.info.description}</p>{/if}
    <FeesForInputs {app} {plan} tips={false} />
    <p class="hint">
      {plan.yours
        ? '✎ changes its fees'
        : 'ℹ shows the full details and caveats · ✎ changes a copy of its fees'}
    </p>
  </div>
{/if}

<style>
  .preview {
    position: fixed;
    z-index: 10;
    width: 360px;
    max-width: 360px;
    border-left: 3px solid var(--plan-color);
    pointer-events: none;
    transition:
      top 180ms ease-out,
      bottom 180ms ease-out;
  }
  @media (hover: none) {
    .preview {
      display: none;
    }
  }
  .broker-of {
    margin-left: 6px;
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
