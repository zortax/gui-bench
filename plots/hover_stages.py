"""Where hover latency goes: the median of each stage from input to presentation.

Stages: input to the commit that shows the hover (the app's work), commit to GPU ready, ready to
the display latch (waiting for the next refresh), and latch to presentation. One panel per
framework and refresh rate, one stacked bar per row count; each row of panels shares its scale.
Usage: python3 plots/hover_stages.py [run dir]
"""

import sys

import common as c

STAGES = [
    ("input_to_commit_p50", "input to commit (app)"),
    ("commit_to_ready_p50", "commit to GPU ready"),
    ("ready_to_latch_p50", "ready to latch (wait for refresh)"),
    ("latch_to_present_p50", "latch to present"),
]
# The stages are ordered, so they take steps of the Oldworld blue, dark to light.
STAGE_COLORS = ["#4a5a8f", "#7383bd", "#a3b1e0", "#d3daf1"]


def plot(run, scenario="rows"):
    rates = run.rates(scenario)
    rows = sorted({cell.rows for cell in run.of(scenario)})
    fws = run.frameworks(scenario)
    if not fws:
        return None
    fig = c.figure(2.6 * len(fws) + 1, 3.5 * len(rates) + 2.3, "Where hover latency goes",
                   "Median of each stage from input to the frame on screen, per framework and row "
                   "count. The dotted line is one refresh")
    grid = fig.subplots(len(rates), len(fws), sharey="row", squeeze=False)
    fig.subplots_adjust(left=0.05, right=0.985, top=c.top_of_panels(fig, 0.5), bottom=0.13,
                        wspace=0.3, hspace=0.6)
    for axes, hz in zip(grid, rates):
        for ax, fw in zip(axes, fws):
            cells = {cell.rows: cell for cell in run.series(fw, hz, scenario)}
            for i, r in enumerate(rows):
                report = cells[r].report("hover") if r in cells else None
                if report is None:
                    continue
                bottom = 0.0
                for (name, label), color in zip(STAGES, STAGE_COLORS):
                    value = max(c.metric(report, "latency", name) or 0.0, 0.0)
                    ax.bar(i, value, 0.68, bottom=bottom, color=color, edgecolor=c.SURFACE,
                           linewidth=1.2, label=label if i == 0 else None)
                    bottom += value
            ax.set_xticks(range(len(rows)))
            ax.set_xticklabels([f"{r:,}" for r in rows], rotation=45, ha="right",
                               rotation_mode="anchor")
            ylabel = "milliseconds" if ax is axes[0] else None
            c.style(ax, fw, ylabel, None, note=f"{hz:g} Hz")
            ax.title.set_color(c.ink(fw))
            ax.axhline(1000.0 / hz, color=c.TEXT_2, linewidth=0.8, linestyle=(0, (2, 3)))
            ax.margins(y=0.08)
    handles, labels = grid[0][0].get_legend_handles_labels()
    fig._legend_row = True
    fig.legend(handles, labels, loc="upper left", ncol=len(STAGES), fontsize=9.5,
               labelcolor=c.TEXT, bbox_to_anchor=(0.4 / fig.get_figwidth() - 0.004,
                                                  1 - fig._header - 0.02 / fig.get_figheight()),
               borderaxespad=0)
    return c.save(fig, run, "hover_stages", scenario)


if __name__ == "__main__":
    loaded = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for name in ("rows", "virtual"):
        print(plot(loaded, name))
