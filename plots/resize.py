"""The resize scenarios, of the rows app and of the dashboard: a resize storm (640x480 to 1200x800 in a second, a configure every
refresh) and a scale switch between 1 and 2. Configure to present is how long the app takes to show
a new size; stale frames are frames at an old size; the memory panels compare scale 2 with scale
1.
Usage: python3 plots/resize.py [run dir]
"""

import sys

import common as c


def at_scale(report, name, index, scale=1.0):
    """(median, low, high) over the iterations at one scale: even iterations run at scale 2, odd
    ones at scale 1."""
    sec = c.section(report, "resources")
    if not sec:
        return None
    m = next((m for m in sec["metrics"] if m["name"] == name and not m.get("variant")), None)
    values = [v * scale for v in (m or {}).get("values") or []][index::2]
    if not values:
        return None
    return (c.percentile(values, 0.5), min(values), max(values))


PANELS = [
    ("resize", lambda r: c.spread(r, "resize", "configure_to_present_p50"),
     "Resize: configure to present", "milliseconds", "{:.1f}"),
    ("resize", lambda r: c.spread(r, "resize", "mismatched_frames"),
     "Resize: frames at a stale size", "frames per storm", "{:.0f}"),
    ("resize", lambda r: c.spread(r, "frame_pacing", "fps"),
     "Resize: delivered frames", "frames per second", "{:.0f}"),
    ("resize", lambda r: c.spread(r, "resources", "cpu_ms_per_s"),
     "Resize: app CPU time", "CPU ms per second", "{:.0f}"),
    ("scale", lambda r: c.spread(r, "resize", "configure_to_present_p50"),
     "Scale switch: configure to present", "milliseconds", "{:.1f}"),
    ("scale", lambda r: at_scale(r, "gpu_proc_mem_max", 0, 1 / c.MIB),
     "VRAM at scale 2", "MiB", "{:.0f}"),
    ("scale", lambda r: at_scale(r, "gpu_proc_mem_max", 1, 1 / c.MIB),
     "VRAM at scale 1", "MiB", "{:.0f}"),
    ("scale", lambda r: at_scale(r, "pss_max", 0, 1 / c.MIB), "PSS at scale 2", "MiB", "{:.0f}"),
    ("scale", lambda r: at_scale(r, "pss_max", 1, 1 / c.MIB), "PSS at scale 1", "MiB", "{:.0f}"),
]


WHAT = {"resize": ("Rows app", "rows"), "dashboard-resize": ("Dashboard", "orders")}


def plot(run, scenario="resize"):
    sizes = sorted({cell.rows for cell in run.of(scenario)})
    if not sizes:
        return None
    app, unit_rows = WHAT[scenario]
    n = len(run.frameworks(scenario))
    cols = 3
    rows_n = (len(PANELS) + cols - 1) // cols
    width = 17.0
    height, adjust = c.panel_grid(rows_n * len(sizes), cols, 0.3 * n + 0.25, width, wspace=0.16)
    fig = c.figure(width, height, "Resize and scale switch",
                   f"{app}, 60 Hz. Dot: median iteration · whisker: best to worst iteration")
    grid = fig.subplots(rows_n * len(sizes), cols, squeeze=False)
    fig.subplots_adjust(**adjust)
    for s, rows in enumerate(sizes):
        for i, (report, value, title, unit, fmt) in enumerate(PANELS):
            ax = grid[s * rows_n + i // cols][i % cols]
            c.dots(ax, run.of(scenario, rows=rows),
                   lambda cell, r=report, v=value: v(cell.report(r)), fmt)
            c.hstyle(ax, title, unit, note=f"{rows:,} {unit_rows}")
            if i % cols:
                ax.set_yticklabels([])
        for j in range(len(PANELS), rows_n * cols):
            grid[s * rows_n + j // cols][j % cols].set_visible(False)
    c.legend(fig, run, scenario=scenario)
    return c.save(fig, run, "resize", scenario)


if __name__ == "__main__":
    loaded = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for name in WHAT:
        print(plot(loaded, name))
