"""The list scenario: the order rows in the framework's virtual list at each row count. The
response to a hover and to a sort of the data, the slowest frames and the missed refreshes of a
fling, and the memory.

Usage: python3 plots/virtual_list.py [run dir]
"""

import sys

import common as c
import smoothness


def plot(run):
    hz = run.index["hz"]["main"]
    cells = [cell for cell in run.of("list", hz=hz) if not cell.stopped]
    if not cells:
        return None
    panels = [
        ("Hover a row", "ms from the pointer to the finished frame",
         lambda cell: c.median_ci(c.response_samples(cell.report("hover-row"))), "{:.1f}"),
        ("Sort the data", "ms from the click to the finished frame",
         lambda cell: c.median_ci(c.response_samples(cell.report("click-sort"))), "{:.1f}"),
        ("Fling: slowest frames", "ms, 99th percentile between two frames",
         lambda cell: c.spread(cell.report("scroll-fling"), "frame_pacing", "interval_p99"),
         "{:.1f}"),
        ("Fling: refreshes without a new frame", "% of the refreshes of the fling",
         lambda cell: smoothness.shares(cell.report("scroll-fling")), "{:.1f}"),
        ("Memory", "MiB, proportional set size at rest",
         lambda cell: c.spread(cell.report("idle"), "resources", "pss_max", 1 / c.MIB), "{:.0f}"),
    ]
    width = 19.0
    axes_h = 0.34 * len(cells) + 0.3
    height, adjust = c.panel_grid(1, len(panels), axes_h, width, wspace=0.32, left=1.9)
    fig = c.figure(width, height, "A long list",
                   c.subtitle(run, "The order rows in the framework's virtual list. Dot: "
                                   "median; whisker: range of the iterations, or the 95 % "
                                   "interval of the median for the responses. Lower is better"))
    fig._legend_row = True
    fig.legend(handles=[c.hand_built()], loc="upper left", fontsize=9.5, labelcolor=c.TEXT,
               bbox_to_anchor=(0.4 / width - 0.006, 1 - fig._header - 0.02 / height),
               handletextpad=0.3, borderaxespad=0)
    axes = fig.subplots(1, len(panels), squeeze=False)[0]
    fig.subplots_adjust(**adjust)
    for at, (ax, (title, note, value, fmt)) in enumerate(zip(axes, panels)):
        c.hstyle(ax, title, note=note)
        c.dots(ax, cells, value, fmt,
               label=lambda cell: f"{cell.framework}  {c.compact(cell.rows)} rows",
               hollow_of=lambda cell: run.flagged(cell.framework, "list") is not None)
        if at:
            ax.set_yticklabels([])
    return c.save(fig, run, "list")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
