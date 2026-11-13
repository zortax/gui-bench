"""Writes a tidy CSV of every metric in a run, one line per cell, measurement, section and metric,
into `<run>/summary.csv`. Usage: python3 plots/export.py [run dir]
"""

import csv
import sys

import common as c


def export(run):
    path = run.path / "summary.csv"
    with path.open("w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["scenario", "framework", "hz", "rows", "elements", "measurement", "valid",
                      "section", "metric", "unit", "median", "mean", "min", "max", "n"])
        for cell in sorted(run.cells, key=lambda x: (x.scenario, x.framework, x.hz, x.rows)):
            key = [cell.scenario, cell.framework, f"{cell.hz:g}", cell.rows,
                   "" if cell.elements is None else cell.elements]
            launches = (cell.report("startup") or {}).get("launches", [])
            for name in sorted({k for launch in launches for k in launch if k.endswith("_ms")}):
                values = [launch[name] for launch in launches if launch.get(name) is not None]
                if values:
                    out.writerow(key + ["startup", "", "startup", name, "ms",
                                        c.percentile(values, 0.5), sum(values) / len(values),
                                        min(values), max(values), len(values)])
            for name, report in sorted(cell.files.items()):
                if name.startswith("phases-") or name == "startup" or not isinstance(report, dict):
                    continue
                for sec in report.get("sections", []):
                    for m in sec["metrics"]:
                        out.writerow(key + [name, c.valid(report), sec["name"], m["name"],
                                            m["unit"], m["median"], m["mean"], m["min"], m["max"],
                                            m["n"]])
    return path


if __name__ == "__main__":
    print(export(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
