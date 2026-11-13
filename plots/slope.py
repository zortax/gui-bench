"""What a larger app costs: for every framework and interaction, the growth of the median
response time per 1,000 elements, from a straight-line fit over the measured sizes.

Usage: python3 plots/slope.py [run dir]
"""

import math
import sys

import common as c
import response

# The slopes in ms per 1,000 elements that take the lightest and the darkest shade.
LOW, HIGH = 0.01, 30.0


def fit(points):
    """Returns the least-squares slope of (x, y) points, or None with fewer than two."""
    if len(points) < 2:
        return None
    n = len(points)
    mx = sum(x for x, _ in points) / n
    my = sum(y for _, y in points) / n
    spread = sum((x - mx) ** 2 for x, _ in points)
    return sum((x - mx) * (y - my) for x, y in points) / spread if spread else None


def cell(run, framework, name):
    """Returns (text, shade) of the slope of one framework and probe."""
    slope = fit([(p[0] / 1000.0, p[1]) for p in response.curve(run, framework, name)])
    if slope is None:
        return ("–", None)
    if slope < LOW:
        return (f"< {LOW:g}", 0.05)
    t = (math.log(slope) - math.log(LOW)) / (math.log(HIGH) - math.log(LOW))
    return (f"{slope:.2f}" if slope < 10 else f"{slope:.0f}", 0.05 + 0.95 * min(1.0, t))


def plot(run):
    frameworks = run.frameworks("app")
    if not frameworks:
        return None
    columns = [title for _, title in c.RESPONSES]
    width = 2.2 + 1.18 * len(columns)
    panel = 0.46 * len(frameworks)
    height = 1.08 + 0.75 + panel + 0.9 + 0.6
    fig = c.figure(width, height, "What a larger app costs",
                   "Milliseconds of response time added per 1,000 elements in the app (fit of "
                   "the medians over the sizes). Darker is more; near zero means the work "
                   "follows the change, not the size of the app")
    ax = fig.subplots()
    fig.subplots_adjust(left=1.9 / width, right=1 - 0.3 / width, top=1 - 2.2 / height,
                        bottom=0.7 / height)
    cells = [[cell(run, f, name) for name, _ in c.RESPONSES] for f in frameworks]
    c.matrix(ax, frameworks, columns, cells, [run.ink(f) for f in frameworks])
    return c.save(fig, run, "slope")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
