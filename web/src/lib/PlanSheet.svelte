<script lang="ts">
  import type { AppState } from './app.svelte'
  import BrokerPicker from './BrokerPicker.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'
  import YourPlans from './YourPlans.svelte'

  /** Every plan there is to compare, to tick: the brokers', the funds' and
   * your own. It covers the whole screen on a phone; on a wider one it
   * covers the inputs only, so the table beside it changes as plans are
   * ticked. */
  let { app, open = $bindable(false) }: { app: AppState; open: boolean } = $props()

  let dialog: HTMLDialogElement
  $effect(() => {
    if (open && !dialog.open) dialog.showModal()
    else if (!open && dialog.open) dialog.close()
  })
</script>

<!-- Clicking beside it closes it, as Esc and "Done" do. -->
<dialog
  bind:this={dialog}
  aria-labelledby="plans-title"
  onclose={() => {
    open = false
    // The plan under the mouse went with it.
    app.hovered = null
  }}
  onclick={(event) => {
    if (event.target === dialog) dialog.close()
  }}
>
  <div class="sheet">
    <header>
      <h2 id="plans-title">{t.whatsCompared}</h2>
      <button class="close" aria-label={t.close} onclick={() => dialog.close()}>✕</button>
    </header>
    <div class="groups">
      <section>
        <div class="heading">
          <h3>{t.banksAndHouses}</h3>
          <Tip about={t.banksAndHouses}>
            <p>
              את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. לכל אחד יש כמה מסלולים,
              ולכל מסלול עמלות משלו: ללאומי, למשל, יש מסלול ללאומי טרייד, מסלול לקבוצת 18+ ומסלול לאפליקציית
              פפר. לחצו על שם כדי לראות את המסלולים שלו, וסמנו את אלה שתרצו להשוות.
            </p>
          </Tip>
        </div>
        <BrokerPicker {app} />
      </section>
      <section>
        <div class="heading">
          <h3>{t.fundsAndPolicies}</h3>
          <Tip about={t.fundsAndPolicies}><p>{t.fundsAndPoliciesTip}</p></Tip>
        </div>
        <BrokerPicker {app} funds />
      </section>
      <section>
        <div class="heading">
          <h3>{t.yourPlans}</h3>
          <Tip about={t.yourPlans}>
            <p>
              בנקים ובתי השקעות נותנים לא פעם הנחה בעמלות למי שמתמקח: למשל 0.06% במקום 0.07%, או מינימום נמוך
              יותר. לחצו על ✎ ליד מסלול כדי להעתיק אותו ולשנות את העמלות למה שאתם חושבים שתוכלו לקבל, או
              הוסיפו מסלול שלא ברשימה עם ״+ מסלול חדש״.
            </p>
          </Tip>
        </div>
        <YourPlans {app} />
      </section>
    </div>
    <footer>
      <button class="done" onclick={() => dialog.close()}>{t.doneComparing(app.compared.length)}</button>
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
  section + section {
    margin-top: 20px;
  }
  .heading {
    display: flex;
    align-items: center;
    margin-bottom: 4px;
  }
  h3 {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--weak);
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
