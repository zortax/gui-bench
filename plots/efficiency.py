"""Power proxy overview: the app CPU time and GPU time per second of each scenario's steady phase,
one row per framework. The fine variant (or the only one) stands for each framework. Lower is
cheaper; a blank cell has no measurement.
Usage: python3 plots/efficiency.py [run dir]
"""

import math
import sys

import matplotlib.colors as mcolors

import common as c

# (column label, scenario, report, match)
COLUMNS = [
    ("idle\n1k rows", "rows", "idle", {"rows": 1000}),
    ("hover\n1k rows", "rows", "hover", {"rows": 1000}),
    ("scroll\n1k rows", "rows", "scroll", {"rows": 1000}),
    ("scroll\n10k rows", "rows", "scroll", {"rows": 10000}),
    ("ticker 1k\n1 row per tick", "ticker", "pacing", {"rows": 1000, "updates": 1}),
    ("ticker 10k\n100 rows per tick", "ticker", "pacing", {"rows": 10000, "updates": 100}),
    ("spinner\nno rows", "spinner", "pacing", {"rows": 0}),
    ("spinner\n10k rows", "spinner", "pacing", {"rows": 10000}),
    ("typing\n1k rows", "typing", "typing", {"rows": 1000}),
    ("scroll\n1M virtual", "virtual", "scroll", {"rows": 1_000_000}),
    ("dashboard\nidle", "dashboard", "idle", {"rows": 50}),
    ("dashboard\nhover row", "dashboard", "hover-row", {"rows": 50}),
    ("dashboard\nscroll 1k orders", "dashboard", "scroll", {"rows": 1000}),
]
PANELS = [
    (lambda r: c.resource(r, "cpu_ms_per_s"), "App CPU time", "CPU ms per second, median iteration"),
    (lambda r: c.cost(r, "gpu_fence_ms_per_s"), "GPU time",
     "commit to buffer fence, ms per second, median iteration"),
]


def pick(run, scenario, fw, hz, match):
    cells = [cell for cell in run.of(scenario, framework=fw, hz=hz, **match)]
    cells.sort(key=c.order_key)
    return cells[0] if cells else None


def plot(run, hz=60.0):
    fws = run.frameworks()
    columns = [col for col in COLUMNS if run.of(col[1], hz=hz)]
    if not fws or not columns:
        return None
    width = 1.35 * len(columns) + 2.6
    height, adjust = c.panel_grid(len(PANELS), 1, 0.42 * len(fws), width, gap=1.5, legend=False,
                                  left=1.3, bottom=1.3)
    fig = c.figure(width, height, "Power proxy: CPU and GPU time per second",
                   f"The steady phase of each scenario at {hz:g} Hz, the fine variant where there "
                   "are several. Lower is cheaper; darker is costlier on a log scale")
    axes = fig.subplots(len(PANELS), 1, squeeze=False)[:, 0]
    fig.subplots_adjust(**adjust)
    cmap = mcolors.LinearSegmentedColormap.from_list("seq", ["#f1f0f4", "#c3cceb", "#92a2d5",
                                                             "#4a5a8f", "#2c3660"])
    for ax, (value, title, unit) in zip(axes, PANELS):
        grid = []
        for fw in fws:
            line = []
            for _, scenario, report, match in columns:
                cell = pick(run, scenario, fw, hz, match)
                v = value(cell.report(report)) if cell else None
                line.append(math.nan if v is None else v)
            grid.append(line)
        finite = [v for line in grid for v in line if not math.isnan(v)]
        top = max(finite) if finite else 1.0
        norm = mcolors.LogNorm(vmin=max(0.1, top / 1000), vmax=max(top, 0.2))
        shown = [[max(v, 0.1) if not math.isnan(v) else top / 1000 for v in line]
                 for line in grid]
        ax.pcolormesh(shown, cmap=cmap, norm=norm, edgecolors=c.SURFACE, linewidth=3)
        for i, line in enumerate(grid):
            for j, v in enumerate(line):
                if math.isnan(v):
                    ax.add_patch(plt_rect(j, i))
                    ax.text(j + 0.5, i + 0.5, "–", ha="center", va="center", fontsize=10.1,
                            color=c.MUTED)
                    continue
                dark = norm(max(v, 0.1)) > 0.55
                ax.text(j + 0.5, i + 0.5, f"{v:,.1f}" if v < 100 else f"{v:,.0f}", ha="center",
                        va="center", fontsize=10.1, color="white" if dark else c.TEXT,
                        fontweight="bold" if dark else "normal")
        ax.set_xticks([j + 0.5 for j in range(len(columns))])
        ax.set_xticklabels([col[0] for col in columns], fontsize=9.5, rotation=0)
        ax.set_yticks([i + 0.5 for i in range(len(fws))])
        ax.set_yticklabels(fws, fontsize=10.1)
        for tick, fw in zip(ax.get_yticklabels(), fws):
            tick.set_color(c.ink(fw))
            tick.set_fontweight("bold")
        ax.invert_yaxis()
        c.style(ax, title, grid="none", note=unit)
    return c.save(fig, run, "efficiency")


def plt_rect(j, i):
    """A blank cell for a missing value."""
    from matplotlib.patches import Rectangle

    return Rectangle((j + 0.03, i + 0.06), 0.94, 0.88, facecolor=c.BAND, edgecolor="none")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
