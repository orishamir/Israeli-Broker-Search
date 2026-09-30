<script lang="ts">
  import type { AppState } from './app.svelte'
  import BrokerPicker from './BrokerPicker.svelte'
  import Sheet from './Sheet.svelte'
  import { t } from './text'
  import Tip from './Tip.svelte'
  import YourPlans from './YourPlans.svelte'

  /** Every plan there is to compare, to tick: the brokers', the funds' and
   * your own. */
  let { app, open = $bindable(false) }: { app: AppState; open: boolean } = $props()
</script>

<!-- The plan under the mouse goes with it when it closes. -->
<Sheet
  bind:open
  title={t.whatsCompared}
  done={t.doneComparing(app.compared.length)}
  onclose={() => (app.hovered = null)}
>
  <section>
    <div class="heading">
      <h3>{t.banksAndHouses}</h3>
      <Tip about={t.banksAndHouses}>
        <p>
          את אותו נייר ערך, באותה בורסה, אפשר לקנות דרך בנקים ובתי השקעות רבים. לכל אחד יש כמה מסלולים, ולכל
          מסלול עמלות משלו: ללאומי, למשל, יש מסלול ללאומי טרייד, מסלול לקבוצת 18+ ומסלול לאפליקציית פפר. לחצו
          על שם כדי לראות את המסלולים שלו, וסמנו את אלה שתרצו להשוות.
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
          יותר. לחצו על ✎ ליד מסלול כדי להעתיק אותו ולשנות את העמלות למה שאתם חושבים שתוכלו לקבל, או הוסיפו
          מסלול שלא ברשימה עם ״+ מסלול חדש״.
        </p>
      </Tip>
    </div>
    <YourPlans {app} />
  </section>
</Sheet>

<style>
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
</style>
