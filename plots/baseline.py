"""The cost of a small app: startup, memory, idle work, the size of the binary and the hover
response of every framework at the smallest size of the app scenario. The rows name the version
of every framework.

Usage: python3 plots/baseline.py [run dir]
"""

import sys

import common as c


def startup(cell):
    """Returns (median, low, high) over the launches of the time to the first frame that shows
    the content."""
    launches = (cell.report("startup") or {}).get("launches", [])
    values = [launch["content_ms"] for launch in launches
              if launch.get("content_ms") is not None]
    return (c.percentile(values, 0.5), min(values), max(values)) if values else None


def plot(run):
    cells = run.of("app", hz=run.index["hz"]["main"])
    cells = [cell for cell in cells if not cell.stopped]
    if not cells:
        return None
    rows = min(cell.rows for cell in cells)
    cells = [cell for cell in cells if cell.rows == rows]
    builds = run.index.get("builds", {})

    def binary(cell):
        size = builds.get(cell.framework, {}).get("binary_bytes")
        return None if size is None else (size / c.MIB,) * 3

    panels = [
        ("Startup", "ms from launch to the frame with the content", startup, "{:.0f}"),
        ("Memory", "MiB, proportional set size at rest",
         lambda cell: c.spread(cell.report("idle"), "resources", "pss_max", 1 / c.MIB), "{:.0f}"),
        ("Work at rest", "CPU ms per second with no input",
         lambda cell: c.spread(cell.report("idle"), "resources", "cpu_ms_per_s"), "{:.1f}"),
        ("Hover response", "ms from the pointer to the finished frame",
         lambda cell: c.median_ci(c.response_samples(cell.report("hover-row"))), "{:.1f}"),
        ("Binary", "MiB, release build with line tables", binary, "{:.0f}"),
    ]
    width = 19.0
    axes_h = 0.34 * len(cells) + 0.3
    height, adjust = c.panel_grid(1, len(panels), axes_h, width, legend=False, wspace=0.32,
                                  left=2.4)
    fig = c.figure(width, height, "The cost of a small app",
                   c.subtitle(run, f"{cells[0].size:,} elements. Dot: median; whisker: range "
                                   "of the launches or iterations, or the 95 % interval of the "
                                   "median for the response. Lower is better"))
    axes = fig.subplots(1, len(panels), squeeze=False)[0]
    fig.subplots_adjust(**adjust)
    for at, (ax, (title, note, value, fmt)) in enumerate(zip(axes, panels)):
        c.hstyle(ax, title, note=note)
        c.dots(ax, cells, value, fmt,
               label=lambda cell: f"{cell.framework}  {run.version(cell.framework)}".rstrip())
        if at:
            ax.set_yticklabels([])
    return c.save(fig, run, "baseline")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
