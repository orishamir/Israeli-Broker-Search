<script lang="ts">
  import { lang, t } from './text'

  /** What a term is called in the other language. In English: its Hebrew
   * name, the usual one first, then the others. In Hebrew: its English name,
   * and the other Hebrew names it goes by, apart from `main`, the one shown. */
  let { names, english, main }: { names: string[]; english?: string; main?: string } = $props()
  const [first, ...rest] = $derived(names)
  const others = $derived(lang === 'he' ? names.filter((name) => name !== main) : rest)
</script>

{#if lang === 'he'}
  {#if english || others.length > 0}
    <dl>
      {#if english && english !== main}
        <dt>{t.inEnglish}</dt>
        <dd><bdi lang="en">{english}</bdi></dd>
      {/if}
      {#if others.length > 0}
        <dt>{t.alsoCalled}</dt>
        <dd class="others">
          {#each others as other (other)}<bdi lang="he">{other}</bdi>{/each}
        </dd>
      {/if}
    </dl>
  {/if}
{:else if first}
  <dl>
    <dt>{t.inHebrew}</dt>
    <dd><bdi lang="he">{first}</bdi></dd>
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
