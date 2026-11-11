"""Startup against row count: process spawn to the first presented frame with the full list.

Every cell launches the app once, so each refresh rate gives one sample per row count; the line
style tells the rates apart. The labels give the values at the lowest rate.
Usage: python3 plots/startup.py [run dir]
"""

import sys

import common as c


def plot(run, scenario="rows"):
    rates = run.rates(scenario)
    if not rates:
        return None
    what = "all rows built" if scenario == "rows" else "virtualized list"
    fig = c.figure(10, 6.4, "Startup",
                   f"Process spawn to the first presented frame, {what}. One launch per refresh "
                   "rate and row count")
    ax = fig.subplots()
    fig.subplots_adjust(left=0.1, right=0.82, top=c.top_of_panels(fig, 0.15), bottom=0.13)
    ends = []
    for fw in run.frameworks(scenario):
        for hz in rates:
            line, marker = c.HZ_STYLE.get(hz, ("-", "o"))
            pts = [(cell.rows, (cell.report("startup") or {}).get("first_present_ms"))
                   for cell in run.series(fw, hz, scenario)]
            pts = [(x, y) for x, y in pts if y is not None]
            if not pts:
                continue
            xs, ys = zip(*pts)
            ax.plot(xs, ys, color=run.color(fw), linestyle=line, marker=marker, markersize=5.5,
                    linewidth=2, markeredgecolor=c.SURFACE, markeredgewidth=1.2)
            if hz == rates[0]:
                ends.append((xs[-1], ys[-1], f"{fw} {ys[-1]:,.0f} ms", c.ink(fw)))
    c.style(ax, None, "milliseconds", "rows")
    c.rows_axis(ax, sorted({cell.rows for cell in run.of(scenario)}))
    ax.set_ylim(bottom=0)
    c.end_labels(ax, ends)
    c.legend(fig, run, rates, scenario=scenario)
    return c.save(fig, run, "startup", scenario)


if __name__ == "__main__":
    loaded = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for name in ("rows", "virtual"):
        print(plot(loaded, name))
