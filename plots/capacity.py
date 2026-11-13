"""How large the app can grow before an interaction misses the frame budget: for every framework
and interaction, the number of elements up to which the 95th percentile of the response time
stays within one refresh, and up to which scrolling and animation miss at most 1 % of the
refreshes.

Usage: python3 plots/capacity.py [run dir]
"""

import math
import sys

import common as c
import response


def response_capacity(run, framework, name, budget):
    """Returns (text, shade) of one cell: the size at which the p95 response crosses `budget`,
    interpolated between the measured sizes on log axes."""
    points = [(p[0], p[3]) for p in response.curve(run, framework, name)]
    return crossing(run, framework, points, lambda v: v <= budget, budget)


def paced_capacity(run, framework, name, hz):
    """Returns (text, shade) of one cell: the largest measured size at which the measurement
    misses at most 1 % of the refreshes at `hz`. A size that holds the main rate holds a slower
    one."""
    main = run.index["hz"]["main"]
    points = []
    for cell in run.series(framework):
        share = c.missed_share(cell.report(name))
        if share is None:
            continue
        if share > c.SMOOTH_SHARE and hz < main:
            slow = next((s for s in run.series(framework, hz=hz) if s.rows == cell.rows), None)
            share = c.missed_share(slow.report(name)) if slow else None
            share = 1.0 if share is None else share
        points.append((cell.size, share))
    return crossing(run, framework, points, lambda v: v <= c.SMOOTH_SHARE, None)


def crossing(run, framework, points, passes, budget):
    """Returns (text, shade) for a list of (size, value) by size: the size up to which every
    value passes."""
    sizes = sorted({cell.size for cell in run.of("app")})
    if not points:
        return ("–", None)
    held = 0
    while held < len(points) and passes(points[held][1]):
        held += 1
    if held == 0:
        return (f"< {c.compact(points[0][0])}", 0.0)
    if held == len(points):
        last = points[-1][0]
        text = f"≥ {c.compact(last)}" if not run.stop(framework) else c.compact(last)
        return (text, shade(last, sizes))
    (x0, y0), (x1, y1) = points[held - 1], points[held]
    if budget is None or y0 <= 0 or y1 <= y0:
        return (c.compact(x0), shade(x0, sizes))
    t = (math.log(budget) - math.log(y0)) / (math.log(y1) - math.log(y0))
    size = math.exp(math.log(x0) + t * (math.log(x1) - math.log(x0)))
    return (c.compact(size), shade(size, sizes))


def shade(size, sizes):
    """Returns the place of a size between the smallest and the largest measured size, on a log
    scale, from 0.12 to 1."""
    lo, hi = math.log(sizes[0]), math.log(sizes[-1])
    t = (math.log(size) - lo) / (hi - lo) if hi > lo else 1.0
    return 0.12 + 0.88 * min(1.0, max(0.0, t))


def plot(run):
    frameworks = run.frameworks("app")
    if not frameworks:
        return None
    columns = [title for _, title in c.RESPONSES] + [title for _, title in c.PACED]
    width = 2.2 + 1.18 * len(columns)
    panel = 0.46 * len(frameworks)
    height = 1.08 + 0.75 + 2 * (panel + 0.75) + 0.9 + 0.6
    fig = c.figure(width, height, "How far each framework scales",
                   "Elements in the app up to which an interaction stays within one frame: the "
                   "95th percentile of the response time, and at most 1 % missed refreshes for "
                   "scrolling and animation. Darker is more; ≥ means the largest size still held")
    axes = fig.subplots(2, 1)
    fig.subplots_adjust(left=1.9 / width, right=1 - 0.3 / width, top=1 - 2.2 / height,
                        bottom=0.7 / height, hspace=(0.75 + 0.55) / panel)
    inks = [run.ink(f) for f in frameworks]
    for ax, (hz, budget) in zip(axes, c.BUDGETS):
        cells = []
        for framework in frameworks:
            row = [response_capacity(run, framework, name, budget) for name, _ in c.RESPONSES]
            row += [paced_capacity(run, framework, name, hz) for name, _ in c.PACED]
            cells.append(row)
        c.matrix(ax, frameworks, columns, cells, inks)
        ax.set_title(f"Within a {hz} Hz frame ({budget:.1f} ms)", loc="left", pad=34)
    return c.save(fig, run, "capacity")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
