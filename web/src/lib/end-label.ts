// The label where a plan's line ends, the same in both line charts.

import type { LineSeriesOption } from 'echarts/charts'
import { readableOn } from './format'

/** How much of a line shows when others are pinned or one is hovered: "the
 * other lines fade only a little" (CLAUDE.md). At 0.4 a thin line in a dark
 * colour all but vanished on the dark page. The label at its end fades less. */
export const FADED_LINE = 0.7
export const FADED_LABEL = 0.85

/** What ECharts does to the other lines while one is hovered
 * (`emphasis.focus: 'series'`): by default it blurs them to 0.1, which is
 * nearly invisible; this is the same little fade as for pinned ones. */
export const BLUR: Pick<LineSeriesOption, 'blur'> = {
  blur: {
    lineStyle: { opacity: FADED_LINE },
    itemStyle: { opacity: FADED_LINE },
    label: { opacity: FADED_LABEL },
    endLabel: { opacity: FADED_LABEL },
    areaStyle: { opacity: FADED_LINE },
  },
}

/** The line's row number from the table, on a badge in its color, then its
 * value in that color. The number tells two plans apart where they share a
 * color (there are more plans than colors that differ enough), and links
 * the line to its row; a plan the table doesn't rank has "–", as its row. */
export function endLabel(
  line: { rank: number | undefined; color: string },
  faded: boolean,
  value: (params: { value: unknown }) => string,
): LineSeriesOption['endLabel'] {
  return {
    show: true,
    // Counts up as the line draws itself in; and a label with no place yet
    // to glide from (see the chart attachment) isn't faded in instead.
    valueAnimation: true,
    formatter: (params) => `{rank|${line.rank ?? '–'}} ${value(params as { value: unknown })}`,
    color: faded ? undefined : line.color,
    opacity: faded ? FADED_LABEL : 1,
    // Keeps the labels in the column apart, a few pixels clear of each other.
    minMargin: 3,
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
