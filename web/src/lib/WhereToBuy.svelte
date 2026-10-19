<script lang="ts">
  // A stub for W4 to fill (REDESIGN.md §2.9). Where to buy, from the visitor's bank: the providers (ties as ties), the banks' range until a bank is named, and the bank's question where `asksBank`. Calls `app.setBank(bank)`.
  import type { AppState } from './app.svelte'

  let { app }: { app: AppState } = $props()
</script>

<div class="stub" data-stub="WhereToBuy" hidden>{app.answer?.how.length}</div>
