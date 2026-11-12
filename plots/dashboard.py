"""The dashboard scenario: a realistic app with cards, shadows, controls and an orders table of 50 or
1000 rows. One figure per refresh rate, one block of panels per table size: the hover and click
latencies, the scroll glide over the orders, and the startup and idle footprint.
Usage: python3 plots/dashboard.py [run dir]
"""

import sys

import common as c

LATENCY = "milliseconds · dot: median · bar: middle half · whisker: 5th to 95th percentile"


def startup(report):
    v = (report or {}).get("first_present_ms")
    return None if v is None else (v, v, v)


# (report, value, title, unit, format)
PANELS = [
    ("hover-nav", c.latency_spread, "Hover a nav item", LATENCY, "{:.1f}"),
    ("hover-row", c.latency_spread, "Hover an order row", LATENCY, "{:.1f}"),
    ("click-compare", c.latency_spread, "Click the compare switch: chart redraw", LATENCY,
     "{:.1f}"),
    ("click-filter", c.latency_spread, "Click a status filter: table update", LATENCY, "{:.1f}"),
    ("scroll", lambda r: c.spread(r, "scroll_smoothness", "smoothness_score"),
     "Scroll: smoothness", "score, 0 to 100 · higher is better", "{:.0f}"),
    ("scroll", lambda r: c.spread(r, "frame_pacing", "fps"), "Scroll: delivered frames",
     "frames per second", "{:.0f}"),
    ("scroll", lambda r: c.spread(r, "scroll_smoothness", "missed_vblanks"),
     "Scroll: missed refreshes", "per 1 s glide · lower is better", "{:.0f}"),
    ("scroll", lambda r: c.spread(r, "resources", "cpu_ms_per_s"), "Scroll: app CPU time",
     "CPU ms per second", "{:.0f}"),
    ("startup", startup, "Startup: first frame", "milliseconds from spawn", "{:,.0f}"),
    ("idle", lambda r: c.spread(r, "resources", "pss_max", 1 / c.MIB), "Idle: memory (PSS)",
     "MiB", "{:.0f}"),
    ("idle", lambda r: c.spread(r, "resources", "gpu_proc_mem_max", 1 / c.MIB), "Idle: VRAM",
     "MiB", "{:.0f}"),
    ("idle", lambda r: c.spread(r, "resources", "cpu_ms_per_s"), "Idle: app CPU time",
     "CPU ms per second", "{:.1f}"),
]


def plot(run, hz):
    sizes = sorted({cell.rows for cell in run.of("dashboard", hz=hz)})
    if not sizes:
        return None
    n = len(run.frameworks("dashboard"))
    cols = 4
    rows_n = (len(PANELS) + cols - 1) // cols
    width = 21.0
    height, adjust = c.panel_grid(rows_n * len(sizes), cols, 0.3 * n + 0.25, width, wspace=0.16)
    fig = c.figure(width, height, f"Dashboard · {hz:g} Hz",
                   "An analytics dashboard with a sidebar, cards with shadows, a chart, controls "
                   "and an orders table; every order row is a real element")
    grid = fig.subplots(rows_n * len(sizes), cols, squeeze=False)
    fig.subplots_adjust(**adjust)
    for s, rows in enumerate(sizes):
        for i, (report, value, title, unit, fmt) in enumerate(PANELS):
            ax = grid[s * rows_n + i // cols][i % cols]
            c.dots(ax, run.of("dashboard", hz=hz, rows=rows),
                   lambda cell, r=report, v=value: v(cell.report(r)), fmt)
            c.hstyle(ax, title, unit, note=f"{rows:,} orders")
            if i % cols:
                ax.set_yticklabels([])
        for j in range(len(PANELS), rows_n * cols):
            grid[s * rows_n + j // cols][j % cols].set_visible(False)
    c.legend(fig, run, scenario="dashboard")
    return c.save(fig, run, f"dashboard_{hz:g}hz", "dashboard")


def plots(run):
    for hz in run.rates("dashboard"):
        yield plot(run, hz)


if __name__ == "__main__":
    for path in plots(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")):
        print(path)
