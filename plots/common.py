"""Shared loading and styling of the gui-bench plots.

A run directory holds `run.json` and one directory per cell,
`<framework>/<hz>hz/<rows>/`, with the scenario reports as JSON.
"""

import json
import math
from dataclasses import dataclass, field
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

SURFACE = "#fcfcfb"
TEXT = "#0b0b0b"
TEXT_2 = "#52514e"
GRID = "#e4e3df"
# Categorical slots in fixed order: frameworks take the first ones.
SLOTS = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
# Refresh rates are a second encoding on top of the framework color.
HZ_STYLE = {60.0: ("-", "o"), 120.0: ("-.", "^"), 144.0: ("--", "s")}


@dataclass
class Cell:
    """The reports of one framework, refresh rate and row count."""

    framework: str
    hz: float
    rows: int
    files: dict = field(default_factory=dict)

    def report(self, name):
        """Returns a loaded report such as `hover` or `scroll`, or None."""
        return self.files.get(name)


@dataclass
class Run:
    """A loaded run directory."""

    path: Path
    index: dict
    cells: list

    @property
    def frameworks(self):
        return [f for f in self.index["frameworks"] if any(c.framework == f for c in self.cells)]

    @property
    def rates(self):
        return sorted({c.hz for c in self.cells})

    def series(self, framework, hz):
        """Returns the cells of one framework and rate, by row count."""
        return sorted(
            (c for c in self.cells if c.framework == framework and c.hz == hz),
            key=lambda c: c.rows,
        )

    def color(self, framework):
        return SLOTS[self.frameworks.index(framework) % len(SLOTS)]


def load(path):
    """Loads a run directory."""
    path = Path(path).resolve()
    index = json.loads((path / "run.json").read_text())
    cells = []
    for entry in index["cells"]:
        info = entry["cell"]
        cell = Cell(entry["framework"], float(info["hz"]), int(info["rows"]))
        for file in (path / entry["dir"]).glob("*.json"):
            cell.files[file.stem] = json.loads(file.read_text())
        cells.append(cell)
    return Run(path, index, cells)


def section(report, name):
    """Returns a section of a bench report by name, or None."""
    if not report:
        return None
    return next((s for s in report["sections"] if s["name"] == name), None)


def metric(report, section_name, name, stat="median"):
    """Returns a metric statistic over iterations, or None."""
    sec = section(report, section_name)
    if not sec:
        return None
    m = next((m for m in sec["metrics"] if m["name"] == name and not m.get("variant")), None)
    return None if m is None else m[stat]


def latency_samples(report, key="total_ms"):
    """Returns every repeat's value of a latency field, pooled over iterations."""
    sec = section(report, "latency")
    if not sec:
        return []
    out = []
    for it in sec["iterations"]:
        for sample in it["report"].get("samples", []):
            value = sample.get(key)
            if value is not None and not sample.get("timed_out"):
                out.append(value)
    return out


def percentile(values, q):
    if not values:
        return math.nan
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, max(0, round((len(ordered) - 1) * q)))]


def style(ax, title=None, ylabel=None, xlabel=None):
    """Applies the recessive house style to an axes."""
    ax.set_facecolor(SURFACE)
    if title:
        ax.set_title(title, loc="left", color=TEXT, fontsize=11, fontweight="bold", pad=8)
    if ylabel:
        ax.set_ylabel(ylabel, color=TEXT_2, fontsize=9)
    if xlabel:
        ax.set_xlabel(xlabel, color=TEXT_2, fontsize=9)
    ax.tick_params(colors=TEXT_2, labelsize=8, length=0)
    ax.grid(axis="y", color=GRID, linewidth=0.8)
    ax.set_axisbelow(True)
    for side in ("top", "right", "left"):
        ax.spines[side].set_visible(False)
    ax.spines["bottom"].set_color(GRID)


def rows_axis(ax, rows):
    """A log row-count axis with a tick at every measured count."""
    ax.set_xscale("log")
    ax.set_xticks(rows)
    ax.set_xticklabels([f"{r:,}" for r in rows])
    ax.minorticks_off()


def figure(width, height, title, subtitle):
    """Returns a figure with the house title block."""
    plt.rcParams["font.family"] = "DejaVu Sans"
    fig = plt.figure(figsize=(width, height), facecolor=SURFACE)
    fig.text(0.015, 0.975, title, ha="left", va="top", color=TEXT, fontsize=14, fontweight="bold")
    fig.text(0.015, 0.935, subtitle, ha="left", va="top", color=TEXT_2, fontsize=9)
    return fig


def legend(fig, run, rates=None, below_title=False):
    """A figure legend of frameworks (color) and refresh rates (line style). Narrow figures put it
    on its own line below the title block."""
    from matplotlib.lines import Line2D

    handles = [Line2D([], [], color=run.color(f), linewidth=2.5, label=f) for f in run.frameworks]
    for hz in rates or []:
        line, marker = HZ_STYLE.get(hz, ("-", "o"))
        handles.append(
            Line2D([], [], color=TEXT_2, linestyle=line, marker=marker, markersize=5,
                   linewidth=1.5, label=f"{hz:g} Hz")
        )
    if below_title:
        fig.legend(handles=handles, loc="upper left", ncol=len(handles), frameon=False,
                   fontsize=9, labelcolor=TEXT, bbox_to_anchor=(0.005, 0.9))
    else:
        fig.legend(handles=handles, loc="upper right", ncol=len(handles), frameon=False,
                   fontsize=9, labelcolor=TEXT, bbox_to_anchor=(0.99, 0.985))


def subtitle(run):
    idx = run.index
    out = idx["output"]
    return (
        f"{out['width']}x{out['height']} output, rows {idx['row_height']} px tall, "
        f"{idx['effort']['iterations']} iterations per scenario · {run.path.name}"
    )


def save(fig, run, name):
    """Writes a figure into the run's `plots/` directory and returns the path."""
    out = run.path / "plots"
    out.mkdir(exist_ok=True)
    path = out / f"{name}.png"
    fig.savefig(path, dpi=150, facecolor=SURFACE)
    plt.close(fig)
    return path
