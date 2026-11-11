"""Scroll glides against row count: smoothness, delivered frame rate, missed refreshes and how
closely the content follows the input.

Lists shorter than a glide have no scroll results. Usage: python3 plots/scroll.py [run dir]
"""

import sys

import common as c

PANELS = [
    ("scroll_smoothness", "smoothness_score", "Smoothness score", "0 to 100, higher is smoother"),
    ("frame_pacing", "fps", "Delivered frame rate", "frames per second"),
    ("scroll_smoothness", "missed_vblanks", "Missed refreshes per glide", "refreshes"),
    ("scroll_smoothness", "tracking_ratio", "Tracking", "content distance / input distance"),
]


def plot(run):
    rates = run.rates
    rows = sorted({cell.rows for cell in run.cells if cell.report("scroll")})
    fig = c.figure(13, 8.6, "Scroll glide (finger, 800 px/s)", c.subtitle(run))
    axes = fig.subplots(2, 2).flatten()
    fig.subplots_adjust(left=0.07, right=0.97, top=0.86, bottom=0.08, hspace=0.38, wspace=0.18)
    for ax, (sec, name, title, unit) in zip(axes, PANELS):
        for fw in run.frameworks:
            for hz in rates:
                line, marker = c.HZ_STYLE.get(hz, ("-", "o"))
                pts = [(cell.rows, c.metric(cell.report("scroll"), sec, name))
                       for cell in run.series(fw, hz) if cell.report("scroll")]
                pts = [(x, y) for x, y in pts if y is not None]
                if not pts:
                    continue
                xs, ys = zip(*pts)
                ax.plot(xs, ys, color=run.color(fw), linestyle=line, marker=marker,
                        markersize=6, linewidth=2, markeredgecolor=c.SURFACE, markeredgewidth=1.5)
        if name == "fps":
            for hz in rates:
                ax.axhline(hz, color=c.TEXT_2, linewidth=0.8, linestyle=(0, (2, 3)))
        if name == "tracking_ratio":
            ax.axhline(1.0, color=c.TEXT_2, linewidth=0.8, linestyle=(0, (2, 3)))
        c.style(ax, title, unit, "rows")
        if rows:
            c.rows_axis(ax, rows)
        if name != "tracking_ratio":
            ax.set_ylim(bottom=0)
    c.legend(fig, run, rates)
    return c.save(fig, run, "scroll")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
