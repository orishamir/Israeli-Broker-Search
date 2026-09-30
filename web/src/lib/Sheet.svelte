<script lang="ts">
  import type { Snippet } from 'svelte'
  import { t } from './text'

  /** A list to choose from, over the page, with a title and a button to
   * finish. It covers the whole screen on a phone; on a wider one it covers
   * the inputs only, so the results beside it change as things are ticked.
   * `done`: the finishing button's words. */
  let {
    open = $bindable(false),
    title,
    done,
    onclose,
    children,
  }: { open: boolean; title: string; done: string; onclose?: () => void; children: Snippet } = $props()

  const id = $props.id()
  let dialog: HTMLDialogElement
  $effect(() => {
    if (open && !dialog.open) dialog.showModal()
    else if (!open && dialog.open) dialog.close()
  })
</script>

<!-- Clicking beside it closes it, as Esc and the button do. -->
<dialog
  bind:this={dialog}
  aria-labelledby="{id}-title"
  onclose={() => {
    open = false
    onclose?.()
  }}
  onclick={(event) => {
    if (event.target === dialog) dialog.close()
  }}
>
  <div class="sheet">
    <header>
      <h2 id="{id}-title">{title}</h2>
      <button class="close" aria-label={t.close} onclick={() => dialog.close()}>✕</button>
    </header>
    <div class="groups">
      {@render children()}
    </div>
    <footer>
      <button class="done" onclick={() => dialog.close()}>{done}</button>
    </footer>
  </div>
</dialog>

<style>
  /* A phone's whole screen. It rises a little and fades in, on a curve that
     starts fast and settles; it leaves faster. At rest it has no transform,
     so the plan preview inside it (fixed) is placed by the window. */
  dialog {
    box-sizing: border-box;
    width: 100%;
    max-width: none;
    height: 100dvh;
    max-height: none;
    margin: 0;
    padding: 0;
    border: none;
    background: var(--surface);
    color: var(--text);
    opacity: 0;
    translate: 0 16px;
    transition:
      opacity 140ms var(--ease-in),
      translate 140ms var(--ease-in),
      overlay 140ms allow-discrete,
      display 140ms allow-discrete;
  }
  dialog[open] {
    opacity: 1;
    translate: none;
    transition:
      opacity 260ms var(--ease-out),
      translate 260ms var(--ease-out),
      overlay 260ms allow-discrete,
      display 260ms allow-discrete;
  }
  @starting-style {
    dialog[open] {
      opacity: 0;
      translate: 0 16px;
    }
  }
  /* The results stay in sight beside it, as they are. */
  dialog::backdrop {
    background: transparent;
  }
  /* Beside the results, over the inputs column: at the start of the line,
     from the top of the window to the bottom. */
  @media (width >= 800px) {
    dialog {
      width: 400px;
      inset-inline-end: auto;
      border-inline-end: 1px solid var(--strong-border);
      box-shadow: var(--shadow);
    }
  }

  .sheet {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: calc(14px + env(safe-area-inset-top)) 20px 12px;
    border-bottom: 1px solid var(--border);
  }
  h2 {
    margin: 0;
    font-size: 1.2rem;
  }
  .close {
    border: none;
    background: transparent;
    color: var(--weak);
    padding: 2px 8px;
  }
  .close:hover {
    color: var(--text);
    background: var(--raised);
  }
  .groups {
    flex: 1;
    overflow-y: auto;
    padding: 14px 20px 20px;
  }
  footer {
    padding: 12px 20px calc(12px + env(safe-area-inset-bottom));
    border-top: 1px solid var(--border);
  }
  .done {
    width: 100%;
    padding: 9px 12px;
    border-color: var(--accent);
    color: var(--accent);
    font-weight: 600;
  }
  .done:hover {
    border-color: var(--accent);
  }
</style>
