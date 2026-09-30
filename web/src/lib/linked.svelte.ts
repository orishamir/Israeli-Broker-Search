// What a table and its line chart share: the row under the mouse, and the
// rows pinned by a click on either.

import type { ECharts } from 'echarts/core'
import type { SvelteSet } from 'svelte/reactivity'
import { touchScreen } from './pointer'

/** A calculator's rows, as its table and charts link them. */
export interface Linked {
  /** The row under the mouse, in the table or a chart. */
  hovered: string | null
  /** Rows clicked in the table or a chart; they stay highlighted. */
  readonly pinned: SvelteSet<string>
  togglePin(id: string): void
}

/** A line chart's `setup` (see `chart`): hovering a line highlights its row
 * and clicking pins it; a row hovered anywhere stands out in the chart; R or
 * Home resets the zoom. `skip`: the id of a line that isn't a row. */
export function linkLines(linked: Linked, skip: string) {
  return (instance: ECharts) => {
    // Events say which series by position; this finds the row's id.
    const rowAt = (seriesIndex: number | undefined): string | undefined => {
      const series = instance.getOption().series as { id: string }[]
      const id = seriesIndex === undefined ? undefined : series[seriesIndex]?.id
      return id === skip ? undefined : id
    }
    instance.on('mouseover', 'series', ({ seriesIndex }) => {
      if (touchScreen) return
      linked.hovered = rowAt(seriesIndex) ?? null
    })
    instance.on('mouseout', 'series', () => {
      linked.hovered = null
    })
    instance.on('click', 'series', ({ seriesIndex }) => {
      const id = rowAt(seriesIndex)
      if (id) linked.togglePin(id)
    })

    // A row hovered anywhere (here or in the table) stands out, the rest fade.
    $effect(() => {
      instance.dispatchAction({ type: 'downplay' })
      if (linked.hovered) instance.dispatchAction({ type: 'highlight', seriesId: linked.hovered })
    })

    const resetZoom = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement
      if (!typing && (event.key === 'r' || event.key === 'Home')) {
        instance.dispatchAction({ type: 'dataZoom', start: 0, end: 100 })
      }
    }
    window.addEventListener('keydown', resetZoom)
    return () => window.removeEventListener('keydown', resetZoom)
  }
}
