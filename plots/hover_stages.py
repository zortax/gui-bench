"""Where hover latency goes: the median of each stage from input to presentation.

Stages: input to the commit that shows the hover (the app's work), commit to GPU ready, ready to
the display latch (waiting for the next refresh), and latch to presentation. One panel per refresh
rate, one stacked bar per framework and row count. Usage: python3 plots/hover_stages.py [run dir]
"""

import sys

import common as c

STAGES = [
    ("input_to_commit_p50", "input → commit (app)"),
    ("commit_to_ready_p50", "commit → GPU ready"),
    ("ready_to_latch_p50", "ready → latch (wait for refresh)"),
    ("latch_to_present_p50", "latch → present"),
]
# The stages take the slots after the frameworks, so no stage looks like a framework.
STAGE_COLORS = c.SLOTS[2:6]


def plot(run):
    rates = run.rates
    rows = sorted({cell.rows for cell in run.cells})
    fws = run.frameworks
    fig = c.figure(6.5 * len(rates) + 1, 5.6, "Hover latency by stage (medians)", c.subtitle(run))
    axes = fig.subplots(1, len(rates), sharey=True, squeeze=False)[0]
    fig.subplots_adjust(left=0.07, right=0.98, top=0.78, bottom=0.22, wspace=0.08)
    width = 0.8 / len(fws)
    for ax, hz in zip(axes, rates):
        for j, fw in enumerate(fws):
            cells = {cell.rows: cell for cell in run.series(fw, hz)}
            for i, r in enumerate(rows):
                report = cells[r].report("hover") if r in cells else None
                if report is None:
                    continue
                x = i + (j - (len(fws) - 1) / 2) * width
                bottom = 0.0
                for (name, label), color in zip(STAGES, STAGE_COLORS):
                    value = max(c.metric(report, "latency", name) or 0.0, 0.0)
                    ax.bar(x, value, width * 0.92, bottom=bottom, color=color,
                           edgecolor=c.SURFACE, linewidth=1, label=label if (i, j) == (0, 0) else None)
                    bottom += value
                ax.text(x, bottom, fw, rotation=90, ha="center", va="bottom", fontsize=7,
                        color=c.TEXT_2)
        ax.set_xticks(range(len(rows)))
        ax.set_xticklabels([f"{r:,}" for r in rows])
        c.style(ax, f"{hz:g} Hz", "ms" if ax is axes[0] else None, "rows")
        ax.margins(y=0.12)
    handles, labels = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels, loc="lower center", ncol=len(STAGES) if len(rates) > 1 else 2,
               frameon=False, fontsize=9,
               labelcolor=c.TEXT, bbox_to_anchor=(0.5, 0.01))
    return c.save(fig, run, "hover_stages")


if __name__ == "__main__":
    print(plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
