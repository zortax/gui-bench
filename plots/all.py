"""Writes every plot of a run into `<run>/plots/<scenario>/` and the tidy `<run>/summary.csv`.

Usage: python3 plots/all.py [run dir]   (default: results/latest)
"""

import sys

import common as c
import dashboard
import efficiency
import export
import footprint
import hover_latency
import hover_stages
import resize
import scroll
import spinner
import startup
import table
import ticker
import typing_latency
import zgui_phases


def outputs(run):
    for scenario in ("rows", "virtual"):
        yield hover_latency.plot(run, scenario=scenario)
        yield hover_latency.plot(run, log=True, scenario=scenario)
        yield hover_stages.plot(run, scenario)
        yield scroll.plot(run, scenario)
        yield startup.plot(run, scenario)
    yield zgui_phases.plot(run)
    yield from table.plot(run)
    yield from ticker.plot(run)
    yield from spinner.plot(run)
    yield typing_latency.plot(run)
    yield resize.plot(run)
    yield from dashboard.plots(run)
    yield resize.plot(run, "dashboard-resize")
    yield from footprint.plot(run)
    yield efficiency.plot(run)
    yield export.export(run)


def main():
    run = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for path in outputs(run):
        if path:
            print(path)


if __name__ == "__main__":
    main()
