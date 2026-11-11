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
        out.writerow(["scenario", "framework", "variant", "hz", "rows", "updates", "measurement",
                      "section", "metric", "unit", "median", "mean", "min", "max", "n"])
        for cell in sorted(run.cells, key=lambda x: (x.scenario, x.framework, x.variant, x.hz,
                                                      x.rows, x.updates or 0)):
            key = [cell.scenario, cell.framework, cell.variant, f"{cell.hz:g}", cell.rows,
                   "" if cell.updates is None else cell.updates]
            startup = cell.report("startup") or {}
            for name, value in startup.items():
                if name.endswith("_ms") and value is not None:
                    out.writerow(key + ["startup", "startup", name, "ms", value, value, value,
                                        value, 1])
            for name, report in sorted(cell.files.items()):
                if name.startswith("phases-") or not isinstance(report, dict):
                    continue
                for sec in report.get("sections", []):
                    for m in sec["metrics"]:
                        out.writerow(key + [name, sec["name"], m["name"], m["unit"], m["median"],
                                            m["mean"], m["min"], m["max"], m["n"]])
    return path


if __name__ == "__main__":
    print(export(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
