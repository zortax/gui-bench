"""Scroll glides against row count: smoothness, delivered frame rate, missed refreshes and how
closely the content follows the input.

One row of panels per refresh rate. The line is the median over the glides, the band runs from the
smallest to the largest glide. Lists shorter than a glide have no scroll results.
Usage: python3 plots/scroll.py [run dir]
"""

import sys

import common as c

PANELS = [
    ("scroll_smoothness", "smoothness_score", "Smoothness", "score, 0 to 100 · higher is better"),
    ("frame_pacing", "fps", "Delivered frame rate", "frames per second"),
    ("scroll_smoothness", "missed_vblanks", "Missed refreshes", "per 1 s glide · lower is better"),
    ("scroll_smoothness", "tracking_ratio", "Tracking", "content distance / input distance"),
]


def plot(run, scenario="rows"):
    rates = run.rates(scenario)
    rows = sorted({cell.rows for cell in run.of(scenario) if cell.report("scroll")})
    if not rows:
        return None
    what = "all rows built" if scenario == "rows" else "virtualized list"
    fig = c.figure(19, 4.3 * len(rates) + 1.9, "Scroll glide",
                   f"A finger glide at 800 px/s, {what}, every frame captured and its content "
                   "motion measured. Line: median glide · band: best to worst glide")
    grid = fig.subplots(len(rates), len(PANELS), squeeze=False)
    fig.subplots_adjust(left=0.05, right=0.985, top=c.top_of_panels(fig, 0.3), bottom=0.09,
                        hspace=0.55, wspace=0.24)
    fws = run.frameworks(scenario)
    for axes, hz in zip(grid, rates):
        for ax, (sec, name, title, unit) in zip(axes, PANELS):
            for i, fw in enumerate(fws):
                pts = [(cell.rows, c.spread(cell.report("scroll"), sec, name))
                       for cell in run.series(fw, hz, scenario) if cell.report("scroll")]
                pts = [(x, s) for x, s in pts if s is not None]
                if not pts:
                    continue
                xs = c.dodge([x for x, _ in pts], i, len(fws), 0.02)
                med, lo, hi = zip(*[s for _, s in pts])
                c.band_line(ax, xs, med, lo, hi, run.color(fw))
            if name == "fps":
                ax.axhline(hz, color=c.MUTED, linewidth=0.9, linestyle=(0, (1, 2.5)))
            if name == "tracking_ratio":
                ax.axhline(1.0, color=c.MUTED, linewidth=0.9, linestyle=(0, (1, 2.5)))
            c.style(ax, f"{title} · {hz:g} Hz", unit, "rows")
            c.rows_axis(ax, rows)
            if name != "tracking_ratio":
                ax.set_ylim(bottom=0)
    c.legend(fig, run, scenario=scenario)
    return c.save(fig, run, "scroll", scenario)


if __name__ == "__main__":
    loaded = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for name in ("rows", "virtual"):
        print(plot(loaded, name))
