"""Writes a tidy CSV of every metric in a run, one line per framework, rate, row count, scenario
and metric, into `<run>/summary.csv`. Usage: python3 plots/export.py [run dir]
"""

import csv
import sys

import common as c

SCENARIOS = ["hover", "hover-marks", "scroll"]


def export(run):
    path = run.path / "summary.csv"
    with path.open("w", newline="") as f:
        out = csv.writer(f)
        out.writerow(["framework", "hz", "rows", "scenario", "section", "metric", "unit",
                      "median", "mean", "min", "max", "n"])
        for cell in sorted(run.cells, key=lambda x: (x.framework, x.hz, x.rows)):
            key = [cell.framework, f"{cell.hz:g}", cell.rows]
            startup = cell.report("startup") or {}
            for name, value in startup.items():
                if name.endswith("_ms") and value is not None:
                    out.writerow(key + ["startup", "startup", name, "ms", value, value, value,
                                        value, 1])
            for scenario in SCENARIOS:
                report = cell.report(scenario)
                for sec in (report or {}).get("sections", []):
                    for m in sec["metrics"]:
                        out.writerow(key + [scenario, sec["name"], m["name"], m["unit"],
                                            m["median"], m["mean"], m["min"], m["max"], m["n"]])
    return path


if __name__ == "__main__":
    print(export(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")))
