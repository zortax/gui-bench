"""Shared loading and styling of the gui-bench plots.

A run directory (format 3) holds `run.json` and one directory per cell,
`<scenario>/<framework>/<hz>hz/<rows>/`, with `cell.json` and the measurement
reports as JSON.
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
FRAMEWORKS = ["zgui", "gpui-kit", "freya", "vizia", "floem", "iced", "egui", "dioxus", "tauri"]
HUES = [
    ("#92a2d5", "#4a5a8f", 0.3),   # blue
    ("#f5a191", "#a8542f", 0.3),   # orange
    ("#90b99f", "#3f7a58", 0.32),  # green
    ("#e6b99d", "#8a5a2c", 0.5),   # yellow
    ("#e29eca", "#9a4a7e", 0.3),   # magenta
    ("#85b5ba", "#3d7076", 0.32),  # cyan
    ("#aca1cf", "#6a5a94", 0.3),   # purple
    ("#ea83a5", "#a63d61", 0.3),   # rose
    # The web stack is the reference beside the native frameworks: it takes the neutral, so no
    # ninth hue has to sit between the eight.
    ("#a9a5b0", "#2c2a30", 0.75),  # neutral
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
EDITIONS = {"gpui-kit-tuned": "gpui-kit", "gpui-kit-fast": "gpui-kit", "iced-tuned": "iced",
            "freya-056a5b4": "freya", "dioxus-skia": "dioxus"}
MIB = 1024.0 * 1024.0
# The frame budgets in milliseconds: one refresh at 144 Hz and at 60 Hz.
BUDGETS = [(144, 1000.0 / 144.0), (60, 1000.0 / 60.0)]
# The share of missed refreshes up to which a scroll, a resize or an animation counts as smooth.
SMOOTH_SHARE = 0.01
# The responses of the app scenario: the report, and what the user does.
RESPONSES = [
    ("hover-row", "Hover a table row"),
    ("hover-nav", "Hover a nav item"),
    ("key-type", "Type a character"),
    ("click-compare", "Toggle the chart compare"),
    ("click-menu", "Open the menu"),
    ("click-sort", "Sort the table"),
    ("click-filter", "Filter by status"),
    ("key-search", "Search: hide every row"),
    ("page-messages", "Switch to the messages"),
    ("page-overview", "Switch to the overview"),
]
# The part of the app that a measurement exercises, where an app may build that part by hand.
PARTS = {"click-menu": "menu", "live": "spinner"}
# The pacing measurements of the app scenario: the report, and what the user does.
PACED = [
    ("scroll", "Scroll, touchpad glide"),
    ("scroll-fling", "Scroll, fling"),
    ("live", "Live values and spinner"),
]


@dataclass
class Cell:
    """The reports of one scenario, framework, refresh rate and size."""

    scenario: str
    framework: str
    hz: float
    rows: int
    elements: int | None
    part: str
    stopped: str | None
    path: Path
    meta: dict
    files: dict = field(default_factory=dict)

    def report(self, name):
        """Returns a loaded report such as `hover-row`, `idle` or `scroll`, or None."""
        return self.files.get(name)

    @property
    def label(self):
        return self.framework

    @property
    def size(self):
        """The size on the x axis: the elements of the app, or the rows of the list."""
        return self.elements if self.elements is not None else self.rows


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

    def series(self, framework, scenario="app", hz=None):
        """Returns the cells of one framework that ran at the main rate (or at `hz`), by size."""
        hz = hz or self.index["hz"]["main"]
        return sorted((c for c in self.of(scenario, framework=framework, hz=hz) if not c.stopped),
                      key=lambda c: c.rows)

    def stop(self, framework, scenario="app"):
        """Returns the smallest size that a framework did not run because it was over the
        response budget before, or None."""
        sizes = [c.size for c in self.of(scenario, framework=framework) if c.stopped]
        return min(sizes) if sizes else None

    def version(self, framework):
        """Returns the crate version of a framework as text, with the revision of a source that
        is no release, such as `0.2.0 @ 1351ffb`, or an empty text."""
        entry = self.index.get("framework_versions", {}).get(framework)
        if not entry:
            return ""
        _, _, revision = entry.get("source", "").partition(" @ ")
        return entry["version"] + (f" @ {revision}" if revision else "")

    def flagged(self, framework, part):
        """Returns the note of a part that the app of a framework builds by hand, or None."""
        build = self.index.get("builds", {}).get(framework, {})
        return next((f["note"] for f in build.get("flags", []) if f["part"] == part), None)

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
    """Sorts cells by framework order, then size."""
    return (framework_rank(cell.framework), cell.rows)


def load(path):
    """Loads a run directory of format 3."""
    path = Path(path).resolve()
    index = json.loads((path / "run.json").read_text())
    if index.get("format") != 3:
        raise SystemExit(f"{path} is no run of format 3; plot it with the scripts of its commit")
    cells = []
    for entry in index["cells"]:
        info = entry["cell"]
        cell_dir = path / entry["dir"]
        cell = Cell(info["scenario"], info["framework"], float(info["hz"]), int(info["rows"]),
                    info.get("elements"), info.get("part", "full"), entry.get("stopped"),
                    cell_dir, entry)
        for file in cell_dir.glob("*.json"):
            if file.stem != "cell":
                cell.files[file.stem] = json.loads(file.read_text())
        cells.append(cell)
    return Run(path, index, cells)


def valid(report):
    """Returns true when the host did not disturb a measurement."""
    return bool(report) and report.get("validity", {}).get("valid", False)


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


def response_samples(report):
    """Returns the response time of every repeat of a probe in milliseconds: from the input to the
    commit of the frame that shows the response, plus the median time until the GPU finished it.
    The refresh the frame then waits for is not part of it."""
    ready = metric(report, "latency", "commit_to_ready_p50") or 0.0
    return [v + ready for v in latency_samples(report, "input_to_commit_ms")]


def timeouts(report):
    """Returns the probes of a report that saw no response."""
    return metric(report, "latency", "timeouts") or 0.0


def missed_shares(report):
    """Returns, per iteration of a pacing measurement, the share of its refreshes that showed no
    new frame: the missed refreshes over the missed and the presented ones."""
    sec = section(report, "frame_pacing")
    shares = []
    for it in sec["iterations"] if sec else []:
        missed, shown = it["report"].get("missed_vblanks"), it["report"].get("presented")
        if missed is not None and shown is not None and missed + shown > 0:
            shares.append(missed / (missed + shown))
    return shares


def missed_share(report):
    """Returns the median share of refreshes without a new frame over the iterations, or None."""
    shares = missed_shares(report)
    return percentile(shares, 0.5) if shares else None


def median_ci(values, draws=400, seed=1):
    """Returns (median, low, high): the median and its 95 % bootstrap interval."""
    import random
    if not values:
        return None
    rng = random.Random(seed)
    n = len(values)
    medians = sorted(percentile([values[rng.randrange(n)] for _ in range(n)], 0.5)
                     for _ in range(draws))
    return (percentile(values, 0.5), medians[int(0.025 * draws)], medians[int(0.975 * draws)])


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


def log_values(ax):
    """A log value axis with plain numbers at the major ticks only."""
    from matplotlib.ticker import NullFormatter
    ax.set_yscale("log")
    ax.yaxis.set_major_formatter(FuncFormatter(thousands))
    ax.yaxis.set_minor_formatter(NullFormatter())


def size_axis(ax, sizes, label="elements in the app"):
    """A log size axis with a tick at every measured size."""
    ax.set_xscale("log")
    ax.set_xticks(sizes)
    ax.set_xticklabels([compact(s) for s in sizes])
    ax.minorticks_off()
    ax.set_xlabel(label)


def compact(value):
    """Formats a count in few characters: 558, 3.5k, 33k, 1M."""
    if value >= 1_000_000:
        return f"{value / 1_000_000:g}M"
    if value >= 10_000:
        return f"{value / 1000:.0f}k"
    if value >= 1000:
        return f"{value / 1000:.1f}k".replace(".0k", "k")
    return f"{value:.0f}"


def budget_lines(ax, upto=None):
    """Draws the frame budgets as dotted lines, named outside the right edge of the panel."""
    for hz, ms in BUDGETS:
        if upto is not None and ms > upto:
            continue
        ax.axhline(ms, color=MUTED, linewidth=0.9, linestyle=(0, (1, 3)), zorder=1)
        ax.annotate(f"{hz} Hz", xy=(1, ms), xycoords=("axes fraction", "data"), xytext=(3, 0),
                    textcoords="offset points", va="center", fontsize=7.2, color=MUTED,
                    annotation_clip=False)


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


def legend(fig, run, scenario=None, extra=()):
    """A legend row under the title block: framework colors as chips, then `extra` handles in
    muted ink."""
    handles = [Line2D([], [], color=run.color(f), marker="o", markersize=7, linewidth=1.8,
                      linestyle="--" if f in EDITIONS else "-", label=f)
               for f in run.frameworks(scenario)]
    handles += list(extra)
    height = fig.get_figheight()
    y = 1 - fig._header - 0.02 / height
    fig._legend_row = True
    per_row = max(1, int(fig.get_figwidth() / 1.45))
    fig.legend(handles=handles, loc="upper left", ncol=min(len(handles), per_row), fontsize=9.5,
               labelcolor=TEXT, bbox_to_anchor=(0.4 / fig.get_figwidth() - 0.006, y),
               handletextpad=0.3, columnspacing=1.4, borderaxespad=0)


def hand_built():
    """The legend handle of a part that an app builds by hand."""
    return Line2D([], [], color=MUTED, marker="o", linestyle="none", markersize=7,
                  markerfacecolor=SURFACE, markeredgewidth=1.5, label="built by hand in the app")


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
    text = (f"{out['width']}x{out['height']} at {idx['hz']['main']:g} Hz, "
            f"{idx['effort']['iterations']} iterations per measurement")
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


def dots(ax, cells, value, fmt="{:.1f}", label=None, hollow_of=None):
    """A horizontal dot plot, one row per cell in framework order. `value(cell)` returns None,
    (median, lo, hi) or (median, q1, q3, lo, hi). The dot is the median in the framework color with
    a round marker, hollow where `hollow_of(cell)` is true; a thin whisker spans lo..hi and a thick bar q1..q3. Rows of one framework
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
        hollow = hollow_of(cell) if hollow_of else False
        ax.plot([med], [y], marker="o", markersize=7,
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


def band_line(ax, xs, med, lo, hi, color_, linestyle="-", marker="o", whisker=None,
              hollow=False):
    """A median line with markers over a band from `lo` to `hi`, and optional thin whiskers
    `whisker = (lows, highs)` at the points. Hollow markers mark a part built by hand."""
    soft = _mix(color_, SURFACE, 0.55)
    ax.fill_between(xs, lo, hi, color=soft, alpha=0.22, linewidth=0, zorder=1)
    if whisker:
        for x, a, b in zip(xs, *whisker):
            ax.plot([x, x], [a, b], color=soft, linewidth=1.1, zorder=2,
                    solid_capstyle="round")
    ax.plot(xs, med, color=color_, linewidth=2.2, linestyle=linestyle, marker=marker,
            markersize=5.5, markeredgecolor=color_ if hollow else SURFACE,
            markerfacecolor=SURFACE if hollow else color_, markeredgewidth=1.2, zorder=3,
            solid_capstyle="round")


def line_style(framework):
    """The line style of a framework: editions are dashed."""
    return "--" if framework in EDITIONS else "-"


def matrix(ax, rows, columns, cells, row_inks=None):
    """Draws a table of shaded cells. `cells[r][c]` is `(text, shade)`: `shade` in 0..1 picks the
    step of one blue ramp, and None leaves the cell empty. Rows are named at the left, columns
    above."""
    import textwrap
    light, dark = "#eef0f8", "#3f4f86"
    ax.set_xlim(0, len(columns))
    ax.set_ylim(len(rows), 0)
    ax.axis("off")
    ax._card = True
    for r, name in enumerate(rows):
        ax.text(-0.12, r + 0.5, name, ha="right", va="center", fontsize=9.5, fontweight="bold",
                color=row_inks[r] if row_inks else TEXT)
        for col, (text, shade) in enumerate(cells[r]):
            fill = BAND if shade is None else _mix(light, dark, shade)
            ax.add_patch(FancyBboxPatch((col + 0.03, r + 0.06), 0.94, 0.88,
                                        boxstyle="round,pad=0,rounding_size=0.06",
                                        facecolor=fill, edgecolor="none"))
            ink_ = MUTED if shade is None else ("#ffffff" if shade > 0.55 else TEXT)
            ax.text(col + 0.5, r + 0.5, text, ha="center", va="center", fontsize=9, color=ink_)
    for col, name in enumerate(columns):
        ax.text(col + 0.5, -0.12, textwrap.fill(name, 13), ha="center", va="bottom",
                fontsize=8.6, color=TEXT_2, linespacing=1.05)
