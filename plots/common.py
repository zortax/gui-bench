"""Shared loading and styling of the gui-bench plots.

A run directory (format 2) holds `run.json` and one directory per cell,
`<scenario>/<framework>/<variant>/<hz>hz/<size>/`, with `cell.json` and the
measurement reports as JSON.
"""

import json
import math
from dataclasses import dataclass, field
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib import font_manager  # noqa: E402
from matplotlib.lines import Line2D  # noqa: E402
from matplotlib.patches import FancyBboxPatch  # noqa: E402
from matplotlib.ticker import FuncFormatter  # noqa: E402

# The page, in the Oldworld light scheme: a warm paper ground, lighter cards, ink in three steps.
PAGE = "#ebe9e4"
SURFACE = "#faf9f7"
TEXT = "#2c2a30"
TEXT_2 = "#5d5963"
MUTED = "#8b8693"
GRID = "#ebe8e3"
RULE = "#d8d5cf"
BAND = "#f3f1ed"
# Libertinus Sans ships with the plots (SIL Open Font License, `fonts/Libertinus-OFL.txt`), so
# every machine renders the same text.
FONT_DIR = Path(__file__).resolve().parent / "fonts"
for _font in sorted(FONT_DIR.glob("*.ttf")):
    font_manager.fontManager.addfont(str(_font))
FONTS = ["Libertinus Sans", "DejaVu Sans"]
# Frameworks take Oldworld hues in fixed order, so a framework keeps its color whatever a run
# selects. Each hue has a pastel (the dark scheme's tone, for bands and fills), an ink (the light
# scheme's tone, for text) and a mid tone between them for lines and marks, set so it holds on
# paper.
FRAMEWORKS = ["zgui", "gpui-kit", "freya", "vizia", "floem", "iced", "egui", "dioxus"]
HUES = [
    ("#92a2d5", "#4a5a8f", 0.3),   # blue
    ("#f5a191", "#a8542f", 0.3),   # orange
    ("#90b99f", "#3f7a58", 0.32),  # green
    ("#e6b99d", "#8a5a2c", 0.5),   # yellow
    ("#e29eca", "#9a4a7e", 0.3),   # magenta
    ("#85b5ba", "#3d7076", 0.32),  # cyan
    ("#aca1cf", "#6a5a94", 0.3),   # purple
    ("#ea83a5", "#a63d61", 0.3),   # rose
]


def _mix(a, b, t):
    """Mixes two hex colors in sRGB: `t` 0 gives `a`, 1 gives `b`."""
    pa = [int(a[i:i + 2], 16) for i in (1, 3, 5)]
    pb = [int(b[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{round(x * (1 - t) + y * t):02x}" for x, y in zip(pa, pb))


SLOTS = [_mix(p, i, t) for p, i, t in HUES]
PASTELS = [p for p, _, _ in HUES]
INKS = [i for _, i, _ in HUES]
# An edition of a framework (another build of the same app) takes its framework's hue, one step
# deeper, and sorts right after it.
EDITIONS = {"gpui-kit-fast": "gpui-kit", "freya-056a5b4": "freya", "dioxus-skia": "dioxus"}
# Variants of the table and ticker are a second encoding on top of the framework color.
VARIANTS = ["fine", "shared", "coarse", "default"]
VARIANT_MARKER = {"fine": "o", "shared": "D", "coarse": "s", "default": "o"}
VARIANT_NAME = {"fine": "fine", "shared": "shared signal", "coarse": "coarse", "default": "one variant"}
# Refresh rates are a second encoding on top of the framework color.
HZ_STYLE = {60.0: ("-", "o"), 120.0: ("-.", "^"), 144.0: ("--", "s")}
MIB = 1024.0 * 1024.0


@dataclass
class Cell:
    """The reports of one scenario, framework, variant, refresh rate and size."""

    scenario: str
    framework: str
    variant: str
    hz: float
    rows: int
    updates: int | None
    path: Path
    meta: dict
    files: dict = field(default_factory=dict)

    def report(self, name):
        """Returns a loaded report such as `hover`, `idle` or `op-swap`, or None."""
        return self.files.get(name)

    @property
    def label(self):
        """The framework, with the variant when it has one."""
        return self.framework if self.variant == "default" else f"{self.framework} {self.variant}"

    @property
    def size(self):
        """The size as text: the row count, with the ticker updates."""
        rows = f"{self.rows:,} rows"
        return rows if self.updates is None else f"{rows}, {self.updates} updates/tick"


@dataclass
class Run:
    """A loaded run directory."""

    path: Path
    index: dict
    cells: list

    def of(self, scenario, **match):
        """Returns the cells of a scenario whose fields equal the given values."""
        return [c for c in self.cells if c.scenario == scenario
                and all(getattr(c, k) == v for k, v in match.items())]

    def frameworks(self, scenario=None):
        names = {c.framework for c in self.cells if scenario is None or c.scenario == scenario}
        return sorted(names, key=lambda f: (framework_rank(f), f))

    def rates(self, scenario=None):
        return sorted({c.hz for c in self.cells if scenario is None or c.scenario == scenario})

    def sizes(self, scenario):
        return sorted({(c.rows, c.updates or 0) for c in self.cells if c.scenario == scenario})

    def series(self, framework, hz, scenario="rows", variant=None):
        """Returns the cells of one framework and rate, by row count."""
        return sorted(
            (c for c in self.of(scenario, framework=framework, hz=hz)
             if variant is None or c.variant == variant),
            key=lambda c: c.rows,
        )

    def color(self, framework):
        return color(framework)

    def ink(self, framework):
        return ink(framework)


def _slot(framework):
    framework = EDITIONS.get(framework, framework)
    order = FRAMEWORKS if framework in FRAMEWORKS else FRAMEWORKS + [framework]
    return order.index(framework) % len(SLOTS)


def color(framework):
    """The mid tone of a framework: lines and marks."""
    tone = SLOTS[_slot(framework)]
    return _mix(tone, INKS[_slot(framework)], 0.55) if framework in EDITIONS else tone


def pastel(framework):
    """The pastel of a framework: bands, ranges and fills."""
    tone = PASTELS[_slot(framework)]
    return _mix(tone, INKS[_slot(framework)], 0.3) if framework in EDITIONS else tone


def ink(framework):
    """The ink of a framework: its name and values as text."""
    tone = INKS[_slot(framework)]
    return _mix(tone, TEXT, 0.35) if framework in EDITIONS else tone


def framework_rank(framework):
    """The place of a framework in plots: editions right after their framework."""
    base = EDITIONS.get(framework, framework)
    rank = FRAMEWORKS.index(base) if base in FRAMEWORKS else len(FRAMEWORKS)
    return rank + (0.5 if framework in EDITIONS else 0.0)


def order_key(cell):
    """Sorts cells by framework order, then variant order."""
    fw = framework_rank(cell.framework)
    var = VARIANTS.index(cell.variant) if cell.variant in VARIANTS else 99
    return (fw, var)


def load(path):
    """Loads a run directory of format 2."""
    path = Path(path).resolve()
    index = json.loads((path / "run.json").read_text())
    if index.get("format") != 2:
        raise SystemExit(f"{path} is no run of format 2; plot it with the scripts of its commit")
    cells = []
    for entry in index["cells"]:
        info = entry["cell"]
        cell_dir = path / entry["dir"]
        cell = Cell(info["scenario"], info["framework"], info["variant"], float(info["hz"]),
                    int(info["rows"]), info.get("updates"), cell_dir, entry)
        for file in cell_dir.glob("*.json"):
            if file.stem != "cell":
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


def theme():
    """Sets the house style for every figure."""
    plt.rcParams.update({
        "font.family": "sans-serif",
        "font.sans-serif": FONTS,
        "font.size": 10.1,
        "axes.facecolor": SURFACE,
        "axes.edgecolor": RULE,
        "axes.labelcolor": TEXT_2,
        "axes.labelsize": 9.5,
        "axes.titlesize": 11.8,
        "axes.titleweight": "bold",
        "axes.titlecolor": TEXT,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "axes.spines.left": False,
        "axes.axisbelow": True,
        "xtick.color": TEXT_2,
        "ytick.color": TEXT_2,
        "xtick.labelsize": 9,
        "ytick.labelsize": 9,
        "xtick.major.size": 0,
        "ytick.major.size": 0,
        "xtick.minor.size": 0,
        "ytick.minor.size": 0,
        "xtick.major.pad": 5,
        "ytick.major.pad": 6,
        "grid.color": GRID,
        "grid.linewidth": 0.9,
        "legend.frameon": False,
        "figure.facecolor": PAGE,
        "savefig.facecolor": PAGE,
    })


theme()


def thousands(value, _=None):
    """Formats a tick value with thousands separators and no trailing zeros."""
    if abs(value) >= 1000:
        return f"{value:,.0f}"
    return f"{value:g}"


def style(ax, title=None, ylabel=None, xlabel=None, grid="y", note=None):
    """Styles a panel: a card behind it, a left-aligned title with an optional note in muted ink,
    a grid on the value axis only, and a hairline base."""
    ax.set_facecolor(SURFACE)
    ax._card = True
    if title:
        ax.set_title(title, loc="left", pad=20 if note else 10)
    if note:
        ax.text(0, 1.0, note, transform=ax.transAxes, ha="left", va="bottom", fontsize=9,
                color=MUTED, fontweight="normal")
        ax.title.set_y(1.0)
    if ylabel:
        ax.set_ylabel(ylabel)
    if xlabel:
        ax.set_xlabel(xlabel)
    if grid == "none":
        ax.grid(False)
        ax.spines["bottom"].set_visible(False)
        return
    ax.grid(axis=grid, visible=True)
    ax.grid(axis="x" if grid == "y" else "y", visible=False)
    ax.spines["bottom"].set_color(RULE)
    if grid == "x":
        ax.spines["bottom"].set_visible(False)
    ax.yaxis.set_major_formatter(FuncFormatter(thousands)) if grid == "y" else \
        ax.xaxis.set_major_formatter(FuncFormatter(thousands))


def hstyle(ax, title=None, xlabel=None, note=None):
    """The style of a horizontal dot panel: the value grid is vertical."""
    style(ax, title, None, xlabel, grid="x", note=note)
    ax.set_xlim(left=0)


def rows_axis(ax, rows):
    """A log row-count axis with a tick at every measured count."""
    ax.set_xscale("log")
    ax.set_xticks(rows)
    ax.set_xticklabels([f"{r:,}" for r in rows])
    ax.minorticks_off()


def end_labels(ax, items, gap=11):
    """Writes direct labels to the right of line ends. `items` holds `(x, y, text)` or
    `(x, y, text, color)` in data coordinates. Labels keep at least `gap` points apart and stay as
    close to their line end as that allows. Call it after the axis limits and scales are final."""
    if not items:
        return
    fig = ax.figure
    to_points = 72.0 / fig.dpi
    placed = []
    for item in items:
        x, y, text = item[:3]
        color = item[3] if len(item) > 3 else TEXT_2
        p = ax.transData.transform((x, y))[1] * to_points
        placed.append([p, p, x, y, text, color])
    placed.sort(key=lambda p: p[0])
    for i in range(1, len(placed)):
        placed[i][1] = max(placed[i][1], placed[i - 1][1] + gap)
    shift = (sum(p[1] for p in placed) - sum(p[0] for p in placed)) / len(placed)
    for p in placed:
        p[1] -= shift
    for i in range(1, len(placed)):
        placed[i][1] = max(placed[i][1], placed[i - 1][1] + gap)
    for want, got, x, y, text, color in placed:
        name, _, rest = text.partition(" ")
        ann = ax.annotate(name, xy=(x, y), xytext=(9, got - want), textcoords="offset points",
                          va="center", fontsize=9, color=color, fontweight="bold",
                          annotation_clip=False)
        if rest:
            ax.annotate(" " + rest, xy=(1, 0.5), xycoords=ann, va="center", fontsize=9,
                        color=TEXT_2, annotation_clip=False)


def figure(width, height, title, subtitle, note=None):
    """Returns a figure with the title block: a bold title, a subtitle that says how to read the
    figure, room for a legend row under it, and a footnote line at the bottom. The block has a
    fixed height in inches, so every figure size lays it out the same."""
    fig = plt.figure(figsize=(width, height))
    left = 0.4 / width
    fig.text(left, 1 - 0.3 / height, title, ha="left", va="top", color=TEXT, fontsize=19,
             fontweight="bold")
    fig.text(left, 1 - 0.74 / height, subtitle, ha="left", va="top", color=TEXT_2,
             fontsize=10.6)
    if note:
        fig.text(left, 0.34 / height, note, ha="left", va="bottom", color=MUTED, fontsize=8.4)
    fig._header = 1.08 / height
    return fig


def top_of_panels(fig, below=0.25):
    """Returns the figure fraction where the panels may start: under the title block, the
    legend row and `below` inches more for panel titles."""
    return 1 - fig._header - (0.32 + below) / fig.get_figheight()


def legend(fig, run, rates=None, below_title=False, scenario=None, variants=None, y=None):
    """A legend row under the title block: framework colors as chips, then refresh rates (line
    style) or variants (marker shape) in muted ink."""
    handles = [Line2D([], [], color=run.color(f), marker="o", linestyle="none", markersize=8,
                      label=f) for f in run.frameworks(scenario)]
    for hz in rates or []:
        line, marker = HZ_STYLE.get(hz, ("-", "o"))
        handles.append(Line2D([], [], color=MUTED, linestyle=line, marker=marker, markersize=5,
                              linewidth=1.5, label=f"{hz:g} Hz"))
    for v in variants or []:
        handles.append(Line2D([], [], color=MUTED, marker=VARIANT_MARKER[v], linestyle="none",
                              markersize=6, markerfacecolor=SURFACE, markeredgewidth=1.4,
                              label=VARIANT_NAME[v]))
    height = fig.get_figheight()
    y = y if y is not None else 1 - fig._header - 0.02 / height
    fig._legend_row = True
    fig.legend(handles=handles, loc="upper left", ncol=len(handles), fontsize=9.5,
               labelcolor=TEXT, bbox_to_anchor=(0.4 / fig.get_figwidth() - 0.006, y),
               handletextpad=0.3, columnspacing=1.4, borderaxespad=0)


def panel_grid(nrows, ncols, axes_h, width, gap=1.3, legend=True, wspace=0.55, left=1.6,
               right=0.35, bottom=0.95):
    """Returns (figure height, adjust kwargs) for a grid of panels whose axes are `axes_h` inches
    tall with `gap` inches between rows for a panel's title, note and axis label. Pass the height
    to `figure` and the kwargs to `fig.subplots_adjust`."""
    top_in = 1.08 + (0.32 if legend else 0) + 0.75
    height = top_in + nrows * axes_h + (nrows - 1) * gap + bottom
    return height, dict(left=left / width, right=1 - right / width, top=1 - top_in / height,
                        bottom=bottom / height, hspace=gap / axes_h, wspace=wspace)


def subtitle(run, extra=None):
    idx = run.index
    out = idx["output"]
    text = f"{out['width']}x{out['height']} at scale 1, {idx['effort']['iterations']} iterations per measurement"
    return f"{extra} · {text}" if extra else text


def footnote(run):
    """The source line of a figure: the run, its revisions and the host."""
    rev = run.index.get("revisions", {})
    parts = [f"run {run.path.name}"] + [f"{k} {v}" for k, v in rev.items() if v]
    env = None
    for cell in run.cells:
        for report in cell.files.values():
            if isinstance(report, dict) and report.get("environment"):
                env = report["environment"]
                break
        if env:
            break
    if env:
        parts.append(f"{env.get('cpu', '')}, {env.get('gpu', '')}")
    return "gui-bench under fakeland · " + " · ".join(parts)


def _card_boxes(fig):
    """Returns the card rectangle `[x0, y0, x1, y1]` in figure coordinates of every styled panel:
    its axes with labels and title, padded. Panels in one grid row share the top and bottom,
    panels in one grid column the sides."""
    fig.canvas.draw()
    renderer = fig.canvas.get_renderer()
    inv = fig.transFigure.inverted()
    pad_x, pad_y = 0.14 / fig.get_figwidth(), 0.12 / fig.get_figheight()
    axes = [ax for ax in fig.axes if getattr(ax, "_card", False)]
    boxes = {}
    for ax in axes:
        box = ax.get_tightbbox(renderer)
        (x0, y0), (x1, y1) = inv.transform([(box.x0, box.y0), (box.x1, box.y1)])
        boxes[ax] = [x0 - pad_x, y0 - pad_y, x1 + pad_x, y1 + pad_y]
    for key, lo, hi in ((lambda p: round(p.y0, 3), 1, 3), (lambda p: round(p.x0, 3), 0, 2)):
        groups = {}
        for ax in axes:
            groups.setdefault(key(ax.get_position()), []).append(ax)
        for group in groups.values():
            low = min(boxes[ax][lo] for ax in group)
            high = max(boxes[ax][hi] for ax in group)
            for ax in group:
                boxes[ax][lo], boxes[ax][hi] = low, high
    return boxes


# The gap between the legend row (or the subtitle) and the first row of cards, in inches.
CARD_GAP = 0.22
# The height of the legend row, in inches.
LEGEND_ROW = 0.3


def _place(fig):
    """Moves every panel together so the top card edge sits `CARD_GAP` below the legend row (or
    below the subtitle), then draws the rounded cards. Returns the bottom of the lowest card in
    figure coordinates."""
    height = fig.get_figheight()
    boxes = _card_boxes(fig)
    if not boxes:
        return 0.5 / height
    above = fig._header * height + (LEGEND_ROW if getattr(fig, "_legend_row", False) else -0.12)
    want = 1 - (above + CARD_GAP) / height
    shift = want - max(box[3] for box in boxes.values())
    for ax in fig.axes:
        pos = ax.get_position()
        ax.set_position([pos.x0, pos.y0 + shift, pos.width, pos.height])
    boxes = _card_boxes(fig)
    for x0, y0, x1, y1 in boxes.values():
        card = FancyBboxPatch((x0, y0), x1 - x0, y1 - y0,
                              boxstyle="round,pad=0,rounding_size=0.008", transform=fig.transFigure,
                              facecolor=SURFACE, edgecolor="none", linewidth=0, zorder=-10)
        fig.patches.append(card)
    return min(box[1] for box in boxes.values())


def save(fig, run, name, scenario=None):
    """Lays the cards out under the title block, writes the footnote under the lowest card, and
    writes the figure, trimmed to an even margin, into `plots/` of the run or
    `plots/<scenario>/`. Returns the path."""
    out = run.path / "plots" / scenario if scenario else run.path / "plots"
    out.mkdir(parents=True, exist_ok=True)
    bottom = _place(fig)
    fig.text(0.4 / fig.get_figwidth(), bottom - 0.26 / fig.get_figheight(), footnote(run),
             ha="left", va="top", color=MUTED, fontsize=7.8)
    path = out / f"{name}.png"
    fig.savefig(path, dpi=160, bbox_inches="tight", pad_inches=0.38)
    plt.close(fig)
    return path


def resource(report, name, stat="median"):
    """Returns a metric of the resources section of a report, or None."""
    return metric(report, "resources", name, stat)


def cost(report, name, stat="median"):
    """Returns a metric of the frame-cost section of a report, or None."""
    return metric(report, "frame_cost", name, stat)


def spread(report, section_name, name, scale=1.0):
    """Returns (median, low, high) of a metric over the iterations: the low and high are the
    smallest and largest iteration. None without the metric."""
    sec = section(report, section_name)
    if not sec:
        return None
    m = next((m for m in sec["metrics"] if m["name"] == name and not m.get("variant")), None)
    if m is None or m.get("median") is None:
        return None
    values = [v for v in m.get("values") or [] if v is not None]
    lo, hi = (min(values), max(values)) if values else (m["median"], m["median"])
    return (m["median"] * scale, lo * scale, hi * scale)


def latency_spread(report, key="total_ms"):
    """Returns (median, p25, p75, p05, p95) of the per-repeat latency samples, or None."""
    values = latency_samples(report, key)
    if not values:
        return None
    return tuple(percentile(values, q) for q in (0.5, 0.25, 0.75, 0.05, 0.95))


def dots(ax, cells, value, fmt="{:.1f}", label=None):
    """A horizontal dot plot, one row per cell in framework order. `value(cell)` returns None,
    (median, lo, hi) or (median, q1, q3, lo, hi). The dot is the median in the framework color with
    the variant's marker; a thin whisker spans lo..hi and a thick bar q1..q3. Rows of one framework
    share a light band. Returns the row labels."""
    cells = sorted(cells, key=order_key)
    n = len(cells)
    ys = list(range(n - 1, -1, -1))
    top = 0.0
    for cell, y in zip(cells, ys):
        v = value(cell)
        if v is None:
            continue
        top = max(top, v[-1] if len(v) > 1 else v[0])
    top = top or 1.0
    shade = False
    previous = None
    for cell, y in zip(cells, ys):
        if cell.framework != previous:
            shade = not shade
            previous = cell.framework
        if shade:
            ax.axhspan(y - 0.5, y + 0.5, color=BAND, zorder=0, linewidth=0)
        v = value(cell)
        color_ = color(cell.framework)
        if v is None:
            ax.text(0, y, "  no data", va="center", fontsize=8.4, color=MUTED)
            continue
        if len(v) == 5:
            med, q1, q3, lo, hi = v
            soft = pastel(cell.framework)
            ax.plot([lo, hi], [y, y], color=soft, linewidth=1.4, solid_capstyle="round",
                    zorder=2)
            ax.plot([q1, q3], [y, y], color=soft, linewidth=6, solid_capstyle="round", zorder=2)
        else:
            med, lo, hi = v
            soft = pastel(cell.framework)
            ax.plot([lo, hi], [y, y], color=soft, linewidth=1.6, solid_capstyle="round",
                    zorder=2)
            for end in (lo, hi):
                ax.plot([end, end], [y - 0.17, y + 0.17], color=soft, linewidth=1.6,
                        solid_capstyle="round", zorder=2)
        marker = VARIANT_MARKER.get(cell.variant, "o")
        hollow = cell.variant == "coarse"
        ax.plot([med], [y], marker=marker, markersize=7 if marker != "D" else 6,
                color=color_, markerfacecolor=SURFACE if hollow else color_,
                markeredgewidth=1.6, markeredgecolor=color_, linestyle="none", zorder=3)
        right = max(v)
        ax.text(right + top * 0.025, y, fmt.format(med), va="center", fontsize=8.4,
                color=ink(cell.framework), zorder=4)
    labels = [label(cell) if label else cell.label for cell in cells]
    ax.set_yticks(ys)
    ax.set_yticklabels(labels, fontsize=9)
    for tick, cell in zip(ax.get_yticklabels(), cells):
        tick.set_color(ink(cell.framework))
    ax.set_ylim(-0.6, n - 0.4)
    ax.set_xlim(0, top * 1.22)
    return labels


def dodge(xs, index, count, step=0.028):
    """Shifts x positions on a log axis a little per series, so whiskers at one x stay apart."""
    factor = 1 + step * (index - (count - 1) / 2)
    return [x * factor for x in xs]


def band_line(ax, xs, med, lo, hi, color_, linestyle="-", marker="o", whisker=None):
    """A median line with markers over a band from `lo` to `hi`, and optional thin whiskers
    `whisker = (lows, highs)` at the points."""
    soft = _mix(color_, SURFACE, 0.55)
    ax.fill_between(xs, lo, hi, color=soft, alpha=0.45, linewidth=0, zorder=1)
    if whisker:
        for x, a, b in zip(xs, *whisker):
            ax.plot([x, x], [a, b], color=soft, linewidth=1.1, zorder=2,
                    solid_capstyle="round")
    ax.plot(xs, med, color=color_, linewidth=2.2, linestyle=linestyle, marker=marker,
            markersize=5.5, markeredgecolor=SURFACE, markeredgewidth=1.2, zorder=3,
            solid_capstyle="round")
