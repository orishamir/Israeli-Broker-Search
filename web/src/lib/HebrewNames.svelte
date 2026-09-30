<script lang="ts">
  import { t } from './text'

  /** What a term is called besides `main`, the Hebrew name shown: its
   * English name, and the other Hebrew names it goes by. */
  let { names, english, main }: { names: string[]; english?: string; main?: string } = $props()
  const others = $derived(names.filter((name) => name !== main))
</script>

{#if english || others.length > 0}
  <dl>
    {#if english && english !== main}
      <dt>{t.inEnglish}</dt>
      <dd><bdi lang="en">{english}</bdi></dd>
    {/if}
    {#if others.length > 0}
      <dt>{t.alsoCalled}</dt>
      <!-- One per line: side by side, Hebrew names read in the wrong order. -->
      <dd class="others">
        {#each others as other (other)}<bdi lang="he">{other}</bdi>{/each}
      </dd>
    {/if}
  </dl>
{/if}

<style>
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 8px;
    margin: 0;
    font-size: 0.8rem;
  }
  dt {
    color: var(--weak);
  }
  dd {
    margin: 0;
  }
  .others bdi {
    display: block;
  }
</style>
