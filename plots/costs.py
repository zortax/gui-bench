"""Shared figure of resource and frame costs: one row of panels per metric, one column per size,
one dot per framework and variant. The dot is the median over the iterations, the whisker spans
the cheapest to the most expensive iteration. The ticker, spinner and idle plots use it.
"""

import common as c

# (section, metric, title, unit, format, scale)
CPU = ("resources", "cpu_ms_per_s", "App CPU time", "CPU ms per second", "{:.1f}", 1.0)
GPU = ("frame_cost", "gpu_fence_ms_per_s", "GPU time", "commit to fence, ms per second",
       "{:.1f}", 1.0)
GPU_BUSY = ("resources", "gpu_proc_busy_percent", "GPU busy (driver)", "percent, per process",
            "{:.0f}", 1.0)
FPS = ("frame_pacing", "fps", "Delivered frames", "frames per second", "{:.0f}", 1.0)
DAMAGE = ("frame_cost", "damage_fraction_mean", "Damaged area", "fraction of the window per frame",
          "{:.2f}", 1.0)
WAKEUPS = ("resources", "wakeups_per_s", "Wakeups", "context switches per second", "{:.0f}", 1.0)
PSS = ("resources", "pss_max", "Memory (PSS)", "MiB", "{:.0f}", 1.0 / c.MIB)
PRIVATE = ("resources", "private_max", "Private memory", "MiB", "{:.0f}", 1.0 / c.MIB)
VRAM = ("resources", "gpu_proc_mem_max", "VRAM", "MiB, per process", "{:.0f}", 1.0 / c.MIB)
COMMITS = ("idle_efficiency", "commits_per_s", "Commits while idle", "per second", "{:.1f}", 1.0)
SUSPENDED = ("idle_efficiency", "suspended_commits", "Commits while suspended", "count", "{:.0f}",
             1.0)


def value(cell, report_name, spec):
    """The (median, low, high) of a metric over the iterations, scaled, or None."""
    section, name, _, _, _, scale = spec
    return c.spread(cell.report(report_name), section, name, scale)


def figure(run, scenario, report_name, metrics, hz, title, extra, name, variant=None, folder=None):
    """Draws the cost figure of a scenario at one refresh rate. Returns the path or None."""
    cells = [cell for cell in run.of(scenario, hz=hz) if cell.report(report_name)
             and (variant is None or cell.variant == variant)]
    if not cells:
        return None
    sizes = sorted({(cell.rows, cell.updates or 0) for cell in cells})
    n = len({cell.label for cell in cells})
    variants = [v for v in c.VARIANTS if any(cell.variant == v for cell in cells)]
    width = 4.3 * len(sizes) + 1.8
    height, adjust = c.panel_grid(len(metrics), len(sizes), 0.3 * n + 0.25, width)
    fig = c.figure(width, height, title,
                   f"{extra}, {hz:g} Hz. Dot: median iteration · whisker: cheapest to most "
                   "expensive iteration")
    grid = fig.subplots(len(metrics), len(sizes), squeeze=False)
    fig.subplots_adjust(**adjust)
    for axes, spec in zip(grid, metrics):
        for ax, (rows, updates) in zip(axes, sizes):
            group = [cell for cell in cells if cell.rows == rows and (cell.updates or 0) == updates]
            c.dots(ax, group, lambda cell, s=spec: value(cell, report_name, s), spec[4])
            size = f"{rows:,} rows" + (f", {updates} update{'' if updates == 1 else 's'} per tick" if updates else "")
            c.hstyle(ax, spec[2], spec[3], note=size)
            if ax is not axes[0]:
                ax.set_yticklabels([])
    c.legend(fig, run, scenario=scenario, variants=variants if len(variants) > 1 else None)
    return c.save(fig, run, f"{name}_{hz:g}hz", folder or scenario)
