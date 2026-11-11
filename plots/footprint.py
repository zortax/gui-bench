"""Memory footprint and idle behavior, from the idle measurement that every cell takes right after
startup. The lines show PSS and per-process VRAM against the row count of the rows and virtual
scenarios; the labels give the slope, the memory one more row costs. The idle figure shows what
an app does with no input: CPU time, wakeups, commits, and commits while suspended.
Usage: python3 plots/footprint.py [run dir]
"""

import sys

import common as c
import costs

PANELS = [
    ("rows", costs.PSS, "Memory (PSS), all rows built"),
    ("rows", costs.VRAM, "VRAM, all rows built"),
    ("virtual", costs.PSS, "Memory (PSS), virtualized list"),
    ("virtual", costs.VRAM, "VRAM, virtualized list"),
]


def slope(xs, ys):
    """The least-squares slope of y over x."""
    n = len(xs)
    if n < 2:
        return None
    mx, my = sum(xs) / n, sum(ys) / n
    den = sum((x - mx) ** 2 for x in xs)
    return None if den == 0 else sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / den


def plot_memory(run):
    hz = min(run.rates() or [60.0])
    panels = [p for p in PANELS if run.of(p[0])]
    if not panels:
        return None
    width = 5.4 * len(panels) + 1.4
    fig = c.figure(width, 6.6, "Memory footprint after startup",
                   f"Right after startup, with no input, {hz:g} Hz. PSS counts shared pages in "
                   "parts; VRAM is the app's own GPU memory. Label: the slope, what one more row "
                   "costs")
    axes = fig.subplots(1, len(panels), squeeze=False)[0]
    fig.subplots_adjust(left=0.75 / width, right=1 - 1.3 / width,
                        top=c.top_of_panels(fig, 0.3), bottom=1.0 / 6.6, wspace=0.55)
    for ax, (scenario, spec, title) in zip(axes, panels):
        ends = []
        fws = run.frameworks(scenario)
        for i, fw in enumerate(fws):
            pts = [(cell.rows, costs.value(cell, "idle", spec))
                   for cell in run.series(fw, hz, scenario)]
            pts = [(x, v) for x, v in pts if v is not None]
            if not pts:
                continue
            xs = [x for x, _ in pts]
            med, lo, hi = zip(*[v for _, v in pts])
            shown = c.dodge(xs, i, len(fws), 0.015) if len(set(xs)) > 1 else xs
            c.band_line(ax, shown, med, lo, hi, c.color(fw))
            per_row = slope(xs, med)
            note = "" if per_row is None else f" {per_row * 1024:,.2f} KiB/row"
            ends.append((xs[-1], med[-1], f"{fw}{note}", c.ink(fw)))
        c.style(ax, title, "MiB", "rows")
        counts = sorted({cell.rows for cell in run.of(scenario)})
        c.rows_axis(ax, counts)
        if len(counts) == 1:
            ax.set_xlim(counts[0] / 1.6, counts[0] * 1.6)
        ax.set_ylim(bottom=0)
        c.end_labels(ax, ends)
    c.legend(fig, run)
    return c.save(fig, run, "memory", "footprint")


def plot_idle(run):
    metrics = [costs.CPU, costs.WAKEUPS, costs.COMMITS, costs.SUSPENDED]
    hz = min(run.rates("rows") or [60.0])
    return costs.figure(run, "rows", "idle", metrics, hz, "Idle: what an app does with no input",
                        "3 s with no input, the middle second with the window suspended",
                        "idle", folder="footprint")


def plot(run):
    return [plot_memory(run), plot_idle(run)]


if __name__ == "__main__":
    for path in plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")):
        print(path)
