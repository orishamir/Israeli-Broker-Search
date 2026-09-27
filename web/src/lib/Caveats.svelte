<script lang="ts">
  /** What the tariff leaves unclear, and how it was read. */
  let { notes }: { notes: string[] } = $props()
</script>

{#if notes.length > 0}
  <ul>
    {#each notes as note (note)}
      <li>{note}</li>
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
  li::before {
    content: '⚠';
    position: absolute;
    left: 10px;
    color: var(--warning);
  }
</style>
