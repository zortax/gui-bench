"""Hover latency against row count: input to the presented frame that shows the hover.

One panel per refresh rate. The line is the median of every repeat, the band runs from the 5th
to the 95th percentile. Usage: python3 plots/hover_latency.py [run dir]
"""

import sys

import common as c


def plot(run):
    rates = run.rates
    fig = c.figure(5.5 * len(rates) + 1, 5.2, "Hover latency (input to photon)", c.subtitle(run))
    axes = fig.subplots(1, len(rates), sharey=True, squeeze=False)[0]
    fig.subplots_adjust(left=0.07, right=0.98, top=0.8, bottom=0.13, wspace=0.08)
    for ax, hz in zip(axes, rates):
        line, marker = c.HZ_STYLE.get(hz, ("-", "o"))
        for fw in run.frameworks:
            xs, p50, p05, p95 = [], [], [], []
            for cell in run.series(fw, hz):
                values = c.latency_samples(cell.report("hover"))
                if not values:
                    continue
                xs.append(cell.rows)
                p50.append(c.percentile(values, 0.5))
                p05.append(c.percentile(values, 0.05))
                p95.append(c.percentile(values, 0.95))
            if not xs:
                continue
            color = run.color(fw)
            ax.fill_between(xs, p05, p95, color=color, alpha=0.15, linewidth=0)
            ax.plot(xs, p50, color=color, linestyle=line, marker=marker, markersize=6,
                    linewidth=2, markeredgecolor=c.SURFACE, markeredgewidth=1.5)
            ax.annotate(f"{p50[-1]:.0f} ms", xy=(xs[-1], p50[-1]), xytext=(6, 0),
                        textcoords="offset points", va="center", fontsize=8, color=c.TEXT_2)
        refresh = 1000.0 / hz
        ax.axhline(refresh, color=c.TEXT_2, linewidth=0.8, linestyle=(0, (2, 3)))
        ax.annotate("one refresh", xy=(0, refresh), xycoords=("axes fraction", "data"),
                    xytext=(2, 3), textcoords="offset points", fontsize=7, color=c.TEXT_2)
        c.style(ax, f"{hz:g} Hz", "ms (median, band: 5th to 95th percentile)" if ax is axes[0]
                else None, "rows")
        c.rows_axis(ax, sorted({cell.rows for cell in run.cells}))
        ax.set_ylim(bottom=0)
    c.legend(fig, run)
    return c.save(fig, run, "hover_latency")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
