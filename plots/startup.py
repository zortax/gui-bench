"""Startup against row count: process spawn to the first presented frame with the full list.

Every cell launches the app once, so each refresh rate gives one sample per row count.
Usage: python3 plots/startup.py [run dir]
"""

import sys

import common as c


def plot(run):
    rates = run.rates
    fig = c.figure(8, 5.2, "Startup (spawn to first presented frame)", c.subtitle(run))
    ax = fig.subplots()
    fig.subplots_adjust(left=0.1, right=0.93, top=0.78, bottom=0.12)
    for fw in run.frameworks:
        for hz in rates:
            line, marker = c.HZ_STYLE.get(hz, ("-", "o"))
            pts = [(cell.rows, (cell.report("startup") or {}).get("first_present_ms"))
                   for cell in run.series(fw, hz)]
            pts = [(x, y) for x, y in pts if y is not None]
            if not pts:
                continue
            xs, ys = zip(*pts)
            ax.plot(xs, ys, color=run.color(fw), linestyle=line, marker=marker, markersize=6,
                    linewidth=2, markeredgecolor=c.SURFACE, markeredgewidth=1.5)
            if hz == rates[0]:
                ax.annotate(f"{ys[-1]:.0f} ms", xy=(xs[-1], ys[-1]), xytext=(6, 0),
                            textcoords="offset points", va="center", fontsize=8,
                            color=c.TEXT_2)
    c.style(ax, None, "ms", "rows")
    c.rows_axis(ax, sorted({cell.rows for cell in run.cells}))
    ax.set_ylim(bottom=0)
    c.legend(fig, run, rates, below_title=True)
    return c.save(fig, run, "startup")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
