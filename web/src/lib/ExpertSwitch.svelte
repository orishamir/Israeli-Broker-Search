<script lang="ts">
  // כל האפשרויות, in the header. The state remembers it and puts it in links
  // (`app.expert`); turning it on changes only what is shown, never the
  // answer.
  import type { AppState } from './app.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'

  let { app }: { app: AppState } = $props()
</script>

<span class="expert">
  <label class="expert-switch">
    <input type="checkbox" role="switch" bind:checked={app.expert} />
    <span class="track" aria-hidden="true"><span class="knob"></span></span>
    {t.allOptions}
  </label><Tip about={t.allOptions}>{t.allOptionsTip}</Tip>
</span>

<style>
  .expert {
    display: inline-flex;
    align-items: center;
  }
  .expert-switch {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 33px;
    cursor: pointer;
    user-select: none;
  }
  /* The checkbox stays for the keyboard and screen readers; the track is
     what's seen. */
  input {
    position: absolute;
    opacity: 0;
    width: 1px;
    height: 1px;
  }
  .track {
    position: relative;
    flex: none;
    box-sizing: border-box;
    width: 36px;
    height: 20px;
    border: 1px solid var(--strong-border);
    border-radius: 999px;
    background: var(--raised);
    transition:
      background var(--quick),
      border-color var(--quick);
  }
  .knob {
    position: absolute;
    inset-block-start: 2px;
    inset-inline-start: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--weak);
    transition:
      translate var(--quick),
      background var(--quick);
  }
  input:checked + .track {
    border-color: var(--accent);
    background: var(--accent-tint);
  }
  /* Sideways: right to left, the knob starts at the right and moves left. */
  input:checked + .track .knob {
    translate: -16px 0;
    background: var(--accent);
  }
  :global([dir='ltr']) input:checked + .track .knob {
    translate: 16px 0;
  }
  input:focus-visible + .track {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .track,
    .knob {
      transition: none;
    }
  }
</style>
