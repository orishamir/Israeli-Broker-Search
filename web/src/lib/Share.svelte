<script lang="ts">
  import type { AppState } from './app.svelte'
  import { touchScreen } from './pointer'
  import { t } from './text'
  import { tip } from './tip'

  /** A button that copies a link to the comparison as it is: on a phone,
   * through its share sheet. */
  let { app }: { app: AppState } = $props()

  let status = $state<'idle' | 'copied' | 'failed'>('idle')
  let link = $state('')
  let popover = $state<HTMLElement>()
  let timer: ReturnType<typeof setTimeout> | undefined

  async function share() {
    link = app.shareLink()
    if (touchScreen && 'share' in navigator) {
      try {
        await navigator.share({ url: link, title: document.title })
      } catch {
        // Closed without sharing.
      }
      return
    }
    try {
      await navigator.clipboard.writeText(link)
      status = 'copied'
      clearTimeout(timer)
      timer = setTimeout(() => (status = 'idle'), 2500)
    } catch {
      // No clipboard (an insecure page): the link itself, to copy by hand.
      status = 'failed'
    }
  }
</script>

<div class="share">
  <span class="status" role="status">{status === 'copied' ? t.linkCopied : ''}</span>
  {#if status === 'failed'}
    <input
      class="link"
      type="text"
      readonly
      value={link}
      aria-label={t.linkToComparison}
      onfocus={(event) => event.currentTarget.select()}
    />
  {/if}
  <button type="button" onclick={share} {@attach popover && tip(popover, { onClick: false })}>
    <svg aria-hidden="true" viewBox="0 0 16 16" width="14" height="14">
      <path
        d="M6.5 9.5l3-3M5 11a2.5 2.5 0 0 1-3.5-3.5l2-2A2.5 2.5 0 0 1 7 5M11 5a2.5 2.5 0 0 1 3.5 3.5l-2 2A2.5 2.5 0 0 1 9 11"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      />
    </svg>
    {t.share}
  </button>
</div>
<div class="popover" popover="manual" role="tooltip" bind:this={popover}>
  <p>{t.shareTip}</p>
</div>

<style>
  /* At the right, even when the title's row wraps it onto its own. */
  .share {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: end;
    gap: 8px;
    margin-inline-start: auto;
  }
  button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .status {
    font-size: 0.85rem;
    color: var(--weak);
  }
  .link {
    width: 16rem;
    max-width: 100%;
    font-size: 1rem;
  }
</style>
