<script lang="ts">
  /** What the tariff leaves unclear, and how it was read. A note about
   * other choices than the user's is titled with what it's about. */
  let { notes }: { notes: (string | { text: string; covers: string })[] } = $props()
</script>

{#if notes.length > 0}
  <ul>
    {#each notes as note (typeof note === 'string' ? note : note.text)}
      {#if typeof note === 'string'}
        <li>{note}</li>
      {:else}
        <li><strong class="covers">{note.covers}</strong>{note.text}</li>
      {/if}
    {/each}
  </ul>
{/if}

<style>
  ul {
    display: grid;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    padding: 8px 10px 8px 32px;
    border-radius: 8px;
    background: rgb(242 193 78 / 0.07);
    color: var(--weak);
    font-size: 0.9rem;
    position: relative;
  }
  .covers {
    display: block;
    color: var(--text);
    font-weight: 600;
  }
  li::before {
    content: '⚠';
    position: absolute;
    left: 10px;
    color: var(--warning);
  }
</style>
