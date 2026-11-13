"""What a fling looks like frame by frame: one strip per framework at the largest app size that
every framework ran. Every frame is a bar as long as it stayed on screen; a frame that stayed
longer than one refresh is dark.

Usage: python3 plots/timelines.py [run dir]
"""

import sys

from matplotlib.lines import Line2D

import common as c

REPORT = "scroll-fling"


def frames(cell):
    """Returns the frames of the first measured fling of a cell: (start ms, length ms, refreshes
    missed)."""
    sec = c.section(cell.report(REPORT), "scroll_smoothness")
    if not sec or not sec["iterations"]:
        return []
    steps = sec["iterations"][0]["report"].get("steps", [])
    out, at = [], 0.0
    for step in steps:
        out.append((at, step["interval_ms"], step.get("vblanks_missed", 0)))
        at += step["interval_ms"]
    return out


def plot(run):
    frameworks = run.frameworks("app")
    have = {f: {cell.rows: cell for cell in run.series(f) if frames(cell)} for f in frameworks}
    sizes = sorted({rows for cells in have.values() for rows in cells})
    if not sizes:
        return None
    # The largest size that most frameworks flung at; the others show no strip.
    rows = max(sizes, key=lambda r: (sum(r in cells for cells in have.values()), r))
    cells = [have[f][rows] for f in frameworks if rows in have[f]]
    absent = [f for f in frameworks if rows not in have[f]]
    width = 13.0
    height = 1.08 + 0.32 + 0.75 + 0.42 * len(cells) + 1.3
    fig = c.figure(width, height, "A fling, frame by frame",
                   f"{cells[0].size:,} elements in the app. Every bar is one frame, as long as "
                   f"it stayed on screen at {run.index['hz']['main']:g} Hz. Dark bars stayed "
                   "longer than one refresh: the scroll stood still"
                   + (f". No fling at this size: {', '.join(absent)}" if absent else ""))
    handles = [Line2D([], [], color=c.MUTED, linewidth=6, label="frame shown for one refresh"),
               Line2D([], [], color=c.TEXT, linewidth=6, label="frame shown for longer")]
    fig._legend_row = True
    fig.legend(handles=handles, loc="upper left", ncol=2, fontsize=9.5, labelcolor=c.TEXT,
               bbox_to_anchor=(0.4 / width - 0.006, 1 - fig._header - 0.02 / height),
               handletextpad=0.5, columnspacing=1.6, borderaxespad=0)
    ax = fig.subplots()
    fig.subplots_adjust(left=1.5 / width, right=1 - 1.6 / width, top=1 - 2.2 / height,
                        bottom=0.8 / height)
    c.style(ax, None, None, "ms since the fling started", grid="x")
    end = 0.0
    for y, cell in zip(range(len(cells) - 1, -1, -1), cells):
        strip = frames(cell)
        late = sum(missed for _, _, missed in strip)
        for start, length, missed in strip:
            # A 2 px gap of surface keeps neighbouring frames apart.
            ax.barh(y, max(length - 0.6, 0.3), left=start, height=0.62,
                    color=c.TEXT if missed else c.pastel(cell.framework), linewidth=0)
        stop = strip[-1][0] + strip[-1][1]
        end = max(end, stop)
        ax.annotate(f"{late} refreshes missed" if late else "none missed", xy=(1, y),
                    xycoords=("axes fraction", "data"), xytext=(8, 0),
                    textcoords="offset points", va="center", fontsize=8.6, color=c.TEXT_2)
    ax.set_yticks(range(len(cells) - 1, -1, -1))
    ax.set_yticklabels([cell.framework for cell in cells], fontsize=9.5, fontweight="bold")
    for tick, cell in zip(ax.get_yticklabels(), cells):
        tick.set_color(run.ink(cell.framework))
    ax.set_xlim(0, end)
    ax.set_ylim(-0.6, len(cells) - 0.4)
    return c.save(fig, run, "timelines")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
