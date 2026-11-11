"""zgui frame phases from its fakeland app marks: the mean CPU time of each phase of a frame
against row count, while scrolling (or while hovering, for lists too short to scroll).

The waits (`input_wait`, `held`) are time the frame spends idle and are left out of the stack.
"other" is the rest of the `frame` span that no named phase covers.
Usage: python3 plots/zgui_phases.py [run dir]
"""

import sys

import common as c

# Phases of one frame in order, folded into groups the palette can carry.
GROUPS = [
    ("events + update", ["events", "update"]),
    ("style", ["style"]),
    ("layout", ["layout"]),
    ("paint", ["paint"]),
    ("record", ["record"]),
    ("acquire + submit", ["acquire", "submit"]),
]


OTHER = "#b9b8b1"


def phases(cell):
    report = cell.report("phases-scroll") or cell.report("phases-hover")
    return report or {}


def plot(run, framework="zgui"):
    rates = run.rates
    cells_by_rate = {hz: [cell for cell in run.series(framework, hz) if phases(cell)] for hz in rates}
    if not any(cells_by_rate.values()):
        return None
    fig = c.figure(6.5 * len(rates) + 1, 5.4, f"{framework} frame phases (app marks, mean per frame)",
                   c.subtitle(run))
    axes = fig.subplots(1, len(rates), sharey=True, squeeze=False)[0]
    fig.subplots_adjust(left=0.07, right=0.98, top=0.8, bottom=0.22, wspace=0.08)
    for ax, hz in zip(axes, rates):
        cells = cells_by_rate[hz]
        for i, cell in enumerate(cells):
            data = phases(cell)
            bottom = 0.0
            # The phases take the slots after the frameworks, so no phase looks like a framework.
            for (label, names), color in zip(GROUPS, c.SLOTS[2:]):
                value = sum(data[n]["mean_ms"] for n in names if n in data)
                ax.bar(i, value, 0.6, bottom=bottom, color=color, edgecolor=c.SURFACE,
                       linewidth=1, label=label if i == 0 else None)
                bottom += value
            frame = data.get("frame", {})
            other = frame.get("mean_ms", 0.0) - bottom
            if other > 0:
                ax.bar(i, other, 0.6, bottom=bottom, color=OTHER, edgecolor=c.SURFACE,
                       linewidth=1, label="other" if i == 0 else None)
                bottom += other
            if frame:
                ax.text(i, bottom, f"frame {frame['mean_ms']:.1f} ms\np95 {frame['p95_ms']:.1f}",
                        ha="center", va="bottom", fontsize=7, color=c.TEXT_2)
        ax.set_xticks(range(len(cells)))
        ax.set_xticklabels([f"{cell.rows:,}" for cell in cells])
        c.style(ax, f"{hz:g} Hz", "ms per frame" if ax is axes[0] else None, "rows")
        ax.margins(y=0.15)
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", ncol=len(GROUPS) + 1 if len(rates) > 1 else 4,
               frameon=False, fontsize=9,
               labelcolor=c.TEXT, bbox_to_anchor=(0.5, 0.01))
    return c.save(fig, run, f"{framework}_phases")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
