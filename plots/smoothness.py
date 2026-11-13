"""Smoothness of the continuous interactions of the app against its size: the slowest frames
and the share of refreshes without a new frame while scrolling and animating, and how long a
resize takes to reach the screen.

Usage: python3 plots/smoothness.py [run dir]
"""

import sys

import common as c


def lines(ax, run, value, sizes, part=None):
    """Draws one line per framework: `value(cell)` returns (median, low, high) or None. Markers
    are hollow for a framework whose app builds `part` by hand."""
    frameworks = run.frameworks("app")
    top = 0.0
    for at, framework in enumerate(frameworks):
        points = [(cell.size, value(cell)) for cell in run.series(framework)]
        points = [(x, v) for x, v in points if v is not None]
        if not points:
            continue
        xs = c.dodge([x for x, _ in points], at, len(frameworks), 0.012)
        c.band_line(ax, xs, [v[0] for _, v in points], [v[1] for _, v in points],
                    [v[2] for _, v in points], run.color(framework), c.line_style(framework),
                    hollow=run.flagged(framework, part) is not None)
        top = max(top, max(v[2] for _, v in points))
    c.size_axis(ax, sizes)
    return top


def shares(report):
    """Returns (median, low, high) in percent of the missed-refresh share over the iterations."""
    values = [100.0 * share for share in c.missed_shares(report)]
    return (c.percentile(values, 0.5), min(values), max(values)) if values else None


def plot(run):
    sizes = sorted({cell.size for cell in run.of("app")})
    if not sizes:
        return None
    hz = run.index["hz"]["main"]
    frame = 1000.0 / hz
    rows = len(c.PACED) + 1
    width = 13.0
    height, adjust = c.panel_grid(rows, 2, 2.3, width, gap=1.25, wspace=0.22, left=0.9)
    fig = c.figure(width, height, "Smoothness as the app grows",
                   c.subtitle(run, "Median line, band from the best to the worst iteration. "
                                   "Lower is better"))
    c.legend(fig, run, "app", [c.hand_built()])
    axes = fig.subplots(rows, 2, squeeze=False, sharex=True)
    fig.subplots_adjust(**adjust)
    for (slow, missed), (name, title) in zip(axes, c.PACED):
        c.style(slow, f"{title}: slowest frames", "ms",
                note="99th percentile of the time between two frames")
        top = lines(slow, run, lambda cell: c.spread(cell.report(name), "frame_pacing",
                                                     "interval_p99"), sizes,
                    c.PARTS.get(name))
        c.log_values(slow)
        slow.set_ylim(frame * 0.6, max(top, frame * 4) * 1.5)
        slow.axhline(frame, color=c.MUTED, linewidth=0.9, linestyle=(0, (1, 3)))
        slow.annotate(f"one refresh at {hz:g} Hz", xy=(0, frame),
                      xycoords=("axes fraction", "data"), xytext=(3, -9),
                      textcoords="offset points", fontsize=7.6, color=c.MUTED)
        c.style(missed, f"{title}: refreshes without a new frame", "%",
                note="share of the refreshes of the measurement")
        top = lines(missed, run, lambda cell: shares(cell.report(name)), sizes,
                    c.PARTS.get(name))
        missed.set_ylim(0, max(top, 4.0) * 1.1)
        missed.axhline(100 * c.SMOOTH_SHARE, color=c.MUTED, linewidth=0.9, linestyle=(0, (1, 3)))
        for ax in (slow, missed):
            ax.set_xlabel(None)
    time, stale = axes[-1]
    c.style(time, "Resize: new size to the screen", "ms",
            note="median time from the size the compositor asks for to its frame on screen")
    top = lines(time, run, lambda cell: c.spread(cell.report("resize"), "resize",
                                                 "configure_to_present_p50"), sizes)
    c.log_values(time)
    time.set_ylim(frame * 0.6, max(top, frame * 4) * 1.5)
    c.budget_lines(time, max(top, frame * 4) * 1.5)
    c.style(stale, "Resize: frames of an outdated size", "frames",
            note="frames shown at a size the window no longer has, per storm of 60 sizes")
    top = lines(stale, run, lambda cell: c.spread(cell.report("resize"), "resize",
                                                  "mismatched_frames"), sizes)
    stale.set_ylim(0, max(top, 4.0) * 1.1)
    return c.save(fig, run, "smoothness")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
