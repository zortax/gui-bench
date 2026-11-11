"""The ticker scenario: what a timer that updates 1 or 100 rows every 16 ms costs. CPU and GPU
time per second are the power proxy; delivered frames show whether the app keeps up; the damaged
area shows how much of the window it redraws for a few changed labels.
Usage: python3 plots/ticker.py [run dir]
"""

import sys

import common as c
import costs

METRICS = [costs.CPU, costs.GPU, costs.FPS, costs.DAMAGE, costs.WAKEUPS]


def plot(run):
    return [costs.figure(run, "ticker", "pacing", METRICS, hz, "Ticker: cost of live row updates",
                         "3 s windows, a tick every 16 ms", "ticker")
            for hz in run.rates("ticker")]


if __name__ == "__main__":
    for path in plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")):
        print(path)
