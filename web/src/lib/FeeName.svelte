<script lang="ts">
  import type { Choice, FeeKind } from './core/core'
  import HebrewNames from './HebrewNames.svelte'
  import Tip from './Tip.svelte'

  /** A fee's name, and its explanation and Hebrew in a tip. */
  let { kind }: { kind: Choice<FeeKind> } = $props()
</script>

{kind.name}<Tip about={kind.name}>
  <HebrewNames names={kind.hebrewNames} />
  <p>{kind.explanation}</p>
</Tip>
