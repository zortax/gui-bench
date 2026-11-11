"""The spinner scenario: what one 24 px element turning once a second costs, alone and above
10,000 rows. A frugal app redraws only the spinner; the damaged area shows who redraws the whole
window, and the CPU and GPU time per second what that costs.
Usage: python3 plots/spinner.py [run dir]
"""

import sys

import common as c
import costs

METRICS = [costs.CPU, costs.GPU, costs.FPS, costs.DAMAGE, costs.WAKEUPS]


def plot(run):
    return [costs.figure(run, "spinner", "pacing", METRICS, hz, "Spinner: cost of a small animation",
                         "3 s windows", "spinner")
            for hz in run.rates("spinner")]


if __name__ == "__main__":
    for path in plot(c.load(sys.argv[1] if len(sys.argv) > 1 else "results/latest")):
        print(path)
