"""The typing and search scenarios: key-to-photon latency of a focused text input above a static
list (typing), the same with a list that filters on every key (search), and the latency until the
list shows the filtered rows or the full list again (filter).
Usage: python3 plots/typing_latency.py [run dir]
"""

import sys

import common as c


def typing(report):
    """(p50, p50, p95) of the key-to-photon latency over the iterations: the dot is the median
    key, the whisker reaches the 95th percentile key."""
    p50 = c.spread(report, "typing", "latency_p50")
    p95 = c.spread(report, "typing", "latency_p95")
    if not p50:
        return None
    return (p50[0], p50[0], p95[0] if p95 else p50[0])


PANELS = [
    ("typing", "typing", typing, "Typing, static list",
     "dot: median key · whisker: to the 95th percentile key"),
    ("search", "typing", typing, "Typing, list filters on every key",
     "dot: median key · whisker: to the 95th percentile key"),
    ("search", "filter", lambda r: c.latency_spread(r),
     "Key to the filtered list", "z hides every row, BackSpace shows them again"),
]


def plot(run):
    sizes = sorted({cell.rows for cell in run.cells if cell.scenario in ("typing", "search")})
    if not sizes:
        return None
    n = len(run.frameworks())
    width = 4.6 * len(sizes) + 1.8
    height, adjust = c.panel_grid(len(PANELS), len(sizes), 0.3 * n + 0.25, width, wspace=0.16)
    fig = c.figure(width, height, "Typing and search",
                   "Key press to the first frame that shows it, 120 words per minute, 60 Hz. "
                   "Filter: dot median · thick bar middle half · whisker 5th to 95th percentile")
    grid = fig.subplots(len(PANELS), len(sizes), squeeze=False)
    fig.subplots_adjust(**adjust)
    for axes, (scenario, report, value, title, how) in zip(grid, PANELS):
        for j, (ax, rows) in enumerate(zip(axes, sizes)):
            cells = run.of(scenario, rows=rows)
            c.dots(ax, cells, lambda cell, r=report, v=value: v(cell.report(r)), "{:.1f}")
            c.hstyle(ax, title, "milliseconds", note=f"{rows:,} rows · {how}")
            if j:
                ax.set_yticklabels([])
    c.legend(fig, run)
    return c.save(fig, run, "typing", "typing")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
