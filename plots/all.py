"""Writes every plot and the summary CSV of a run into `<run>/plots/` and `<run>/summary.csv`.

Usage: python3 plots/all.py [run dir]   (default: results/latest)
"""

import sys

import common as c
import export
import hover_latency
import hover_stages
import scroll
import startup
import zgui_phases


def main():
    run = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    outputs = [
        hover_latency.plot(run),
        hover_stages.plot(run),
        scroll.plot(run),
        startup.plot(run),
        zgui_phases.plot(run),
        export.export(run),
    ]
    for path in outputs:
        if path:
            print(path)


if __name__ == "__main__":
    main()
