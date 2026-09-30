<script lang="ts">
  import type { ShortTermOutcomeData } from './core/core'
  import { chart } from './echarts.svelte'
  import { growthOption, NO_FEES, type Line } from './growth-chart'
  import { linkLines } from './linked.svelte'
  import { touchScreen } from './pointer'
  import type { PlaceResult, ShortTermState } from './short-term.svelte'
  import { t } from './text'

  /** What the money is worth in each place, month by month, beside the
   * same deposits at the Bank of Israel's rate. */
  let { short, results, atTheRate }: { short: ShortTermState; results: PlaceResult[]; atTheRate: number[] } =
    $props()

  const option = () =>
    growthOption({
      lines: results
        .filter(
          (result): result is PlaceResult & { outcome: ShortTermOutcomeData; rank: number } =>
            result.outcome !== undefined,
        )
        .map(({ place, outcome, rank }): Line => ({
          id: place.id,
          label: place.label,
          color: place.color,
          rank,
          dotted: place.yours !== undefined,
          values: outcome.valueByMonth,
        })),
      noFees: atTheRate,
      noFeesName: t.atTheRate,
      lost: false,
      pinned: short.pinned,
      touch: touchScreen,
      months: true,
    })
</script>

<div class="chart" {@attach chart(option, linkLines(short, NO_FEES))}></div>

<style>
  .chart {
    width: 100%;
    height: 420px;
  }
</style>
