"""Response time of every interaction of the app against its size: the time from the input until
the frame that shows the response is ready, without the wait for the next refresh.

Usage: python3 plots/response.py [run dir]
"""

import sys

from matplotlib.lines import Line2D

import common as c


def curve(run, framework, name):
    """Returns the points of one framework and probe: (size, median, p05, p95, valid)."""
    points = []
    for cell in run.series(framework):
        report = cell.report(name)
        values = c.response_samples(report)
        if values:
            points.append((cell.size, c.percentile(values, 0.5), c.percentile(values, 0.05),
                           c.percentile(values, 0.95), c.valid(report)))
    return points


def plot(run):
    frameworks = run.frameworks("app")
    sizes = sorted({cell.size for cell in run.of("app")})
    if not frameworks or not sizes:
        return None
    width, cols = 19.0, 5
    rows = -(-len(c.RESPONSES) // cols)
    height, adjust = c.panel_grid(rows, cols, 2.7, width, gap=1.25, wspace=0.46, left=0.9, right=0.6)
    fig = c.figure(width, height, "Response time as the app grows",
                   c.subtitle(run, "Input to finished frame: median line, band from the 5th to "
                                   "the 95th percentile of the repeats. Lower is better; a flat "
                                   "line means the cost does not depend on the size of the app"))
    extra = [c.hand_built(),
             Line2D([], [], color=c.MUTED, marker="o", linestyle="none", markersize=9,
                    markerfacecolor="none", markeredgewidth=0.9,
                    label="the host disturbed the measurement"),
             Line2D([], [], color=c.MUTED, marker="x", linestyle="none", markersize=6,
                    markeredgewidth=1.6, label="not run: over 250 ms before")]
    c.legend(fig, run, "app", extra)
    axes = fig.subplots(rows, cols, squeeze=False, sharex=True)
    fig.subplots_adjust(**adjust)
    top = 1.0
    for ax, (name, title) in zip(axes.flat, c.RESPONSES):
        c.style(ax, title, "ms" if ax in axes[:, 0] else None)
        for at, framework in enumerate(frameworks):
            points = curve(run, framework, name)
            if not points:
                continue
            xs = c.dodge([p[0] for p in points], at, len(frameworks), 0.012)
            color = run.color(framework)
            c.band_line(ax, xs, [p[1] for p in points], [p[2] for p in points],
                        [p[3] for p in points], color, c.line_style(framework),
                        hollow=run.flagged(framework, c.PARTS.get(name)) is not None)
            bad = [(x, p[1]) for x, p in zip(xs, points) if not p[4]]
            if bad:
                ax.plot(*zip(*bad), marker="o", markersize=9, markerfacecolor="none",
                        markeredgecolor=c.MUTED, markeredgewidth=0.9, linestyle="none", zorder=4)
            top = max(top, max(p[3] for p in points))
            stop = run.stop(framework)
            if stop:
                ax.plot([stop], [points[-1][1]], marker="x", markersize=6, markeredgewidth=1.6,
                        color=color, linestyle="none", zorder=4)
    # Every panel has the same scale, so a flat cheap interaction sits low and a costly one high.
    for ax, _ in zip(axes.flat, c.RESPONSES):
        c.log_values(ax)
        ax.set_ylim(0.2, top * 1.6)
        c.budget_lines(ax, top * 1.6)
        c.size_axis(ax, sizes)
        if ax not in axes[-1]:
            ax.set_xlabel(None)
    for ax in axes.flat[len(c.RESPONSES):]:
        ax.set_visible(False)
    return c.save(fig, run, "response")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
