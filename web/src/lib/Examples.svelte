<script lang="ts">
  // The "דוגמאות" button and its list (REDESIGN.md §4.2): each example's
  // name and numbers, the one on screen ticked. The list floats over the
  // fields, taking no room on the page; on a phone it is a sheet from the
  // bottom, "לראות דוגמה", whose numbers wait for the visitor. Picking one
  // keeps the visitor's own numbers (`app.pickExample`), and the page
  // doesn't move: the list closes and the focus goes back to the button.
  // Self-contained, so the phone's collapsed facts row can hold it too.
  import type { AppState } from './app.svelte'
  import { fieldsOf, numbersLine } from './examples'
  import { oneColumn } from './scroll'
  import { t } from './text'

  let { app }: { app: AppState } = $props()

  const id = $props.id()
  let dialog: HTMLDialogElement
  let button: HTMLButtonElement
  let open = $state(false)

  const items = $derived(
    app.examples.map((example) => ({ example, numbers: numbersLine(fieldsOf(example.answers)) })),
  )

  function show() {
    // On a phone, a sheet over a dimmed page; beside the fields, a list over them.
    if (oneColumn()) dialog.showModal()
    else dialog.show()
    open = true
    const current = dialog.querySelector<HTMLElement>('[aria-current="true"]')
    ;(current ?? dialog.querySelector<HTMLElement>('.item'))?.focus({ preventScroll: true })
  }

  function close() {
    if (dialog.open) dialog.close()
  }

  function pick(exampleId: string) {
    app.pickExample(exampleId)
    close()
  }

  // A list that isn't modal closes at a click or a tap beside it, and at Esc.
  $effect(() => {
    if (!open) return
    const outside = (event: PointerEvent) => {
      const target = event.target as Node
      if (!dialog.contains(target) && !button.contains(target)) close()
    }
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') close()
    }
    addEventListener('pointerdown', outside)
    addEventListener('keydown', escape)
    return () => {
      removeEventListener('pointerdown', outside)
      removeEventListener('keydown', escape)
    }
  })
</script>

<button
  bind:this={button}
  type="button"
  class="open"
  aria-expanded={open}
  aria-controls={id}
  onclick={() => (open ? close() : show())}
>
  {t.examplesButton}<svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"><path d="m6 9 6 6 6-6"></path></svg
  >
</button>

<dialog
  bind:this={dialog}
  {id}
  class="examples"
  aria-labelledby="{id}-title"
  onclose={() => {
    open = false
    button.focus({ preventScroll: true })
  }}
  onclick={(event) => {
    // The sheet's dimmed page around it.
    if (event.target === dialog) close()
  }}
>
  <div class="inside">
    <span class="grip" aria-hidden="true"></span>
    <header>
      <h3 id="{id}-title">{t.examplesTitle}</h3>
      <span class="wait">{t.examplesWait}</span>
      <button type="button" class="close" aria-label={t.close} onclick={close}>✕</button>
    </header>
    <ul>
      {#each items as { example, numbers } (example.id)}
        {@const current = app.exampleOnScreen?.id === example.id}
        <li>
          <button
            type="button"
            class="item"
            class:current
            aria-current={current ? 'true' : undefined}
            onclick={() => pick(example.id)}
          >
            <span class="words">
              <span class="name">{example.name}</span>
              <span class="numbers">{numbers}</span>
            </span>
            {#if current}
              <svg
                class="tick"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.6"
                stroke-linecap="round"
                stroke-linejoin="round"
                role="img"
                aria-label={t.exampleShownNow}><path d="M20 6 9 17l-5-5"></path></svg
              >
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  </div>
</dialog>

<style>
  .open {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 4px;
    min-height: 33px;
  }
  /* Beside the fields: a list over them, under the card's title, at its
     end. Not modal, so the page stays as it is around it. */
  dialog {
    position: absolute;
    z-index: 10;
    top: 48px;
    inset-inline: auto 12px;
    box-sizing: border-box;
    width: min(330px, calc(100% - 24px));
    max-height: min(70dvh, 560px);
    margin: 0;
    padding: 8px;
    overflow-y: auto;
    border: 1px solid var(--strong-border);
    border-radius: var(--radius);
    background: var(--raised);
    color: var(--text);
    box-shadow: var(--shadow);
  }
  .grip,
  .wait {
    display: none;
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 2px 6px 6px;
  }
  h3 {
    flex: 1;
    margin: 0;
    font-size: 1rem;
  }
  .close {
    flex: none;
    border: none;
    background: transparent;
    color: var(--weak);
  }
  .close:hover {
    color: var(--text);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    box-sizing: border-box;
    width: 100%;
    min-height: 44px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    text-align: start;
  }
  .item.current {
    background: var(--accent-tint);
  }
  .words {
    display: flex;
    flex-direction: column;
  }
  .name {
    font-size: 0.92rem;
    font-weight: 600;
  }
  .numbers {
    color: var(--weak);
    font-size: 0.8rem;
  }
  .tick {
    flex: none;
    color: var(--accent);
  }
  /* A phone: a sheet from the bottom of the screen, over the page dimmed. */
  @media (width < 800px) {
    dialog {
      position: fixed;
      inset: auto 0 0;
      width: 100%;
      max-width: none;
      max-height: 85dvh;
      padding: 8px 12px calc(14px + env(safe-area-inset-bottom));
      border-width: 1px 0 0;
      border-radius: 16px 16px 0 0;
    }
    dialog::backdrop {
      background: rgb(0 0 0 / 0.55);
    }
    .grip {
      display: block;
      width: 40px;
      height: 4px;
      margin: 0 auto 8px;
      border-radius: 2px;
      background: var(--strong-border);
    }
    .wait {
      display: inline;
      color: var(--weak);
      font-size: 0.82rem;
    }
    .item {
      border-top: 1px solid var(--border);
      border-radius: 0;
    }
  }
</style>
