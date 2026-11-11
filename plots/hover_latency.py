"""Hover latency against row count: input to the presented frame that shows the hover.

One panel per refresh rate. The line is the median of every repeat, the band the middle half
(25th to 75th percentile) and the whiskers the 5th to 95th percentile. The log variant spreads the
fast frameworks apart and shows ratios.
Usage: python3 plots/hover_latency.py [run dir]
"""

import sys

from matplotlib.ticker import FuncFormatter, LogLocator, NullFormatter

import common as c


def series(run, fw, hz, scenario):
    """Returns the row counts and the latency spread of every cell."""
    xs, stats = [], []
    for cell in run.series(fw, hz, scenario):
        s = c.latency_spread(cell.report("hover"))
        if s:
            xs.append(cell.rows)
            stats.append(s)
    return xs, stats


def plot(run, log=False, scenario="rows"):
    rates = run.rates(scenario)
    if not rates:
        return None
    title = "Hover latency" + (" (log scale)" if log else "")
    what = "all rows built" if scenario == "rows" else "virtualized list"
    fig = c.figure(6.4 * len(rates) + 1.2, 6.6, title,
                   f"Input to the first frame on screen that shows the hover color, {what}. "
                   "Line: median · band: middle half · whiskers: 5th to 95th percentile")
    axes = fig.subplots(1, len(rates), sharey=True, squeeze=False)[0]
    fig.subplots_adjust(left=0.07, right=0.9, top=c.top_of_panels(fig, 0.3), bottom=0.13,
                        wspace=0.28)
    lows = []
    ends = {}
    for ax, hz in zip(axes, rates):
        ends[hz] = []
        fws = run.frameworks(scenario)
        for i, fw in enumerate(fws):
            xs, stats = series(run, fw, hz, scenario)
            if not xs:
                continue
            xs = c.dodge(xs, i, len(fws))
            med, q1, q3, p05, p95 = zip(*stats)
            color = run.color(fw)
            c.band_line(ax, xs, med, q1, q3, color, whisker=(p05, p95))
            lows.append(min(p05))
            ends[hz].append((xs[-1], med[-1], f"{fw} {med[-1]:.0f} ms", c.ink(fw)))
        refresh = 1000.0 / hz
        ax.axhline(refresh, color=c.MUTED, linewidth=0.9, linestyle=(0, (1, 2.5)), zorder=1)
        ax.annotate(f"one refresh · {refresh:.1f} ms", xy=(0.0, refresh),
                    xycoords=("axes fraction", "data"), xytext=(2, 3), textcoords="offset points",
                    ha="left", va="bottom", fontsize=8.4, color=c.MUTED)
        c.style(ax, f"{hz:g} Hz", "milliseconds" if ax is axes[0] else None, "rows")
        c.rows_axis(ax, sorted({cell.rows for cell in run.of(scenario)}))
    if log:
        axes[0].set_yscale("log")
        axes[0].set_ylim(bottom=max(1.0, min(lows, default=1.0) * 0.8))
        axes[0].yaxis.set_major_locator(LogLocator(base=10, subs=(1.0, 2.0, 5.0)))
        axes[0].yaxis.set_major_formatter(FuncFormatter(lambda v, _: f"{v:g}"))
        axes[0].yaxis.set_minor_formatter(NullFormatter())
    else:
        axes[0].set_ylim(bottom=0)
    for ax, hz in zip(axes, rates):
        c.end_labels(ax, ends[hz])
    c.legend(fig, run, scenario=scenario)
    return c.save(fig, run, "hover_latency_log" if log else "hover_latency", scenario)


if __name__ == "__main__":
    loaded = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for name in ("rows", "virtual"):
        print(plot(loaded, scenario=name))
        print(plot(loaded, log=True, scenario=name))
