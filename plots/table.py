"""The table scenario: input-to-photon latency and app CPU time of every table operation, the
coarse/fine latency ratio per framework, and the memory a churn of replacements keeps.

The dot is the framework color, its shape the variant (filled circle: fine or the only variant,
diamond: every row reads a shared selection signal, open square: coarse).
Usage: python3 plots/table.py [run dir]
"""

import math
import sys

import matplotlib.colors as mcolors

import common as c

OPS = [
    ("op-select", "Select a row", "click"),
    ("op-update10", "Update every 10th row", "F1"),
    ("op-swap", "Swap two rows", "F2"),
    ("op-remove", "Remove a row", "F3"),
    ("op-insert", "Insert a row", "F4"),
    ("op-replace", "Replace all rows", "F5"),
    ("op-append", "Append 1,000 rows", "F6"),
    ("op-clear", "Clear or recreate all", "F8"),
]


def p50(cell, name):
    s = c.latency_spread(cell.report(name))
    return s[0] if s else None


def latency(cell, name):
    return c.latency_spread(cell.report(name))


def cpu_per_op(cell, name):
    """App CPU ms per operation, per iteration: the CPU time of an iteration over its operations.
    Every probe repeat runs the operation and its undo, so an iteration holds two per repeat."""
    report = cell.report(name)
    s = c.spread(report, "resources", "cpu_ms")
    if s is None:
        return None
    repeats = report.get("params", {}).get("repeats") or 1
    return tuple(v / (2 * repeats) for v in s)


def plot_ops(run, rows, value, title, how, unit, name):
    cells = run.of("table", rows=rows)
    if not cells:
        return None
    n = len({cell.label for cell in cells})
    variants = [v for v in c.VARIANTS if any(cell.variant == v for cell in cells)]
    width = 19.0
    height, adjust = c.panel_grid(2, 4, 0.3 * n + 0.25, width, wspace=0.16)
    fig = c.figure(width, height, title, f"{rows:,} rows, 60 Hz. {how}")
    axes = fig.subplots(2, 4, squeeze=False).flatten()
    fig.subplots_adjust(**adjust)
    for i, (ax, (file, op, key)) in enumerate(zip(axes, OPS)):
        c.dots(ax, cells, lambda cell, f=file: value(cell, f), "{:.1f}")
        c.hstyle(ax, op, unit, note=key)
        if i % 4:
            ax.set_yticklabels([])
    c.legend(fig, run, scenario="table", variants=variants)
    return c.save(fig, run, f"{name}_{rows}", "table")


def plot_speedup(run):
    """Coarse p50 over fine p50 per framework and operation: above 1, the fine variant is
    faster."""
    sizes = sorted({cell.rows for cell in run.of("table")})
    fws = [fw for fw in run.frameworks("table")
           if {"fine", "coarse"} <= {cell.variant for cell in run.of("table", framework=fw)}]
    if not fws or not sizes:
        return None
    width = 15.0
    height, adjust = c.panel_grid(len(sizes), 1, 0.42 * len(fws), width, gap=1.2, legend=False,
                                  left=1.3, bottom=1.25)
    fig = c.figure(width, height, "How much fine-grained state buys",
                   "Median latency of the coarse variant divided by the fine variant, per "
                   "operation. Blue: fine is faster · orange: coarse is faster · gray: the same")
    axes = fig.subplots(len(sizes), 1, squeeze=False)[:, 0]
    fig.subplots_adjust(**adjust)
    cmap = mcolors.LinearSegmentedColormap.from_list(
        "div", ["#a8542f", "#f5a191", "#efede9", "#92a2d5", "#4a5a8f"])
    norm = mcolors.LogNorm(vmin=1 / 16, vmax=16)
    for ax, rows in zip(axes, sizes):
        grid = []
        for fw in fws:
            fine = run.of("table", framework=fw, variant="fine", rows=rows)
            coarse = run.of("table", framework=fw, variant="coarse", rows=rows)
            line = []
            for file, _, _ in OPS:
                a = p50(coarse[0], file) if coarse else None
                b = p50(fine[0], file) if fine else None
                line.append(a / b if a and b else math.nan)
            grid.append(line)
        masked = [[v if not math.isnan(v) else 1.0 for v in line] for line in grid]
        ax.pcolormesh(masked, cmap=cmap, norm=norm, edgecolors=c.SURFACE, linewidth=3)
        for i, line in enumerate(grid):
            for j, v in enumerate(line):
                strong = not math.isnan(v) and (v > 4 or v < 0.25)
                text = "no data" if math.isnan(v) else f"×{v:.2g}" if v >= 1 else f"÷{1 / v:.2g}"
                ax.text(j + 0.5, i + 0.5, text, ha="center", va="center", fontsize=10.1,
                        color="white" if strong else c.TEXT,
                        fontweight="bold" if strong else "normal")
        ax.set_xticks([j + 0.5 for j in range(len(OPS))])
        ax.set_xticklabels([op for _, op, _ in OPS], fontsize=9.5)
        ax.set_yticks([i + 0.5 for i in range(len(fws))])
        ax.set_yticklabels(fws, fontsize=10.1)
        for tick, fw in zip(ax.get_yticklabels(), fws):
            tick.set_color(c.ink(fw))
            tick.set_fontweight("bold")
        ax.invert_yaxis()
        c.style(ax, f"{rows:,} rows", grid="none")
        ax.spines["bottom"].set_visible(False)
    return c.save(fig, run, "speedup", "table")


def kept_per_cycle(cell, keys):
    """KiB per replace cycle that the app keeps, or None."""
    churn = cell.report("churn")
    if not churn:
        return None
    before = churn["before"].get("process") or {}
    after = churn["after"].get("process") or {}
    for key in keys:
        if before.get(key) is not None and after.get(key) is not None:
            v = (after[key] - before[key]) / churn["cycles"] / 1024.0
            return (v, v, v)
    return None


def plot_churn(run):
    cells = [cell for cell in run.of("table") if cell.report("churn")]
    if not cells:
        return None
    sizes = sorted({cell.rows for cell in cells})
    n = len({cell.label for cell in cells})
    variants = [v for v in c.VARIANTS if any(cell.variant == v for cell in cells)]
    panels = [(rows, keys, title) for rows in sizes for keys, title in
              ((("pss_bytes", "rss_bytes"), "Memory (PSS)"), (("gpu_mem_bytes",), "VRAM"))]
    width = 4.4 * len(panels) + 1.8
    height, adjust = c.panel_grid(1, len(panels), 0.3 * n + 0.25, width)
    cycles = cells[0].report("churn")["cycles"]
    fig = c.figure(width, height, "Memory kept per replace-all cycle",
                   f"{cycles} replacements of every row (F5), memory after minus before, per "
                   "cycle. Near zero: nothing leaks · one sample per cell")
    axes = fig.subplots(1, len(panels), squeeze=False)[0]
    fig.subplots_adjust(**adjust)
    for i, (ax, (rows, keys, title)) in enumerate(zip(axes, panels)):
        group = [cell for cell in cells if cell.rows == rows]
        c.dots(ax, group, lambda cell, k=keys: kept_per_cycle(cell, k), "{:.0f}")
        c.hstyle(ax, title, "KiB per cycle", note=f"{rows:,} rows")
        lo = min((kept_per_cycle(cell, keys) or (0,))[0] for cell in group)
        hi = max((kept_per_cycle(cell, keys) or (0,))[0] for cell in group)
        span = max(hi - lo, 1.0)
        ax.set_xlim(min(lo, 0) - span * 0.1, max(hi, 0) + span * 0.25)
        ax.axvline(0, color=c.MUTED, linewidth=0.9)
        if i:
            ax.set_yticklabels([])
    c.legend(fig, run, scenario="table", variants=variants)
    return c.save(fig, run, "churn", "table")


def plot(run):
    out = []
    for rows in sorted({cell.rows for cell in run.of("table")}):
        out.append(plot_ops(run, rows, latency, "Table operations: input to photon",
                            "Dot: median · thick bar: middle half · whisker: 5th to 95th "
                            "percentile of the probe repeats", "milliseconds", "latency"))
        out.append(plot_ops(run, rows, cpu_per_op, "Table operations: app CPU time",
                            "CPU time of the app per operation. Dot: median iteration · whisker: "
                            "cheapest to most expensive iteration", "CPU ms per operation",
                            "cpu"))
    out.append(plot_speedup(run))
    out.append(plot_churn(run))
    return out


if __name__ == "__main__":
    for path in plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")):
        print(path)
