// The label where a plan's line ends, the same in both line charts.

import type { LineSeriesOption } from 'echarts/charts'
import { readableOn } from './format'

/** The line's row number from the table, on a badge in its color, then its
 * value in that color. The number tells two plans apart where they share a
 * color (there are more plans than colors that differ enough), and links
 * the line to its row. */
export function endLabel(
  line: { rank: number; color: string },
  faded: boolean,
  value: (params: { value: unknown }) => string,
): LineSeriesOption['endLabel'] {
  return {
    show: true,
    // Counts up as the line draws itself in; and a label with no place yet
    // to glide from (see the chart attachment) isn't faded in instead.
    valueAnimation: true,
    formatter: (params) => `{rank|${line.rank}} ${value(params as { value: unknown })}`,
    color: faded ? undefined : line.color,
    opacity: faded ? 0.6 : 1,
    rich: {
      rank: {
        backgroundColor: line.color,
        color: readableOn(line.color),
        fontWeight: 'bold',
        fontSize: 11,
        // No taller than the value beside it: a taller label leaves less room
        // in the column of end labels, and more of them are hidden.
        lineHeight: 12,
        padding: [0, 3],
        borderRadius: 3,
      },
    },
  }
}

/** How a line's labels make room. The end labels, which share one column,
 * are moved apart (unless `moveEnds` is off: on a phone they're hidden
 * instead); the labels on its points only hide where they'd overlap, as
 * moving them too would stack them into that column (moving is up and down
 * only) and push the end labels out of the chart. The end label is the one
 * on the line itself, which isn't a point, so it has no index. */
export const labelLayout =
  (moveEnds: boolean): LineSeriesOption['labelLayout'] =>
  ({ dataIndex }) =>
    moveEnds && dataIndex === undefined ? { moveOverlap: 'shiftY', hideOverlap: true } : { hideOverlap: true }
