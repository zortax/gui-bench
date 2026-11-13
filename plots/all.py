"""Writes every plot of a run into `<run>/plots/` and the tidy `<run>/summary.csv`.

Usage: python3 plots/all.py [run dir]   (default: results/latest)
"""

import sys

import baseline
import capacity
import common as c
import export
import response
import slope
import smoothness
import timelines
import virtual_list


def outputs(run):
    yield capacity.plot(run)
    yield response.plot(run)
    yield slope.plot(run)
    yield smoothness.plot(run)
    yield timelines.plot(run)
    yield baseline.plot(run)
    yield virtual_list.plot(run)
    yield export.export(run)


def main():
    run = c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")
    for path in outputs(run):
        if path:
            print(path)


if __name__ == "__main__":
    main()
