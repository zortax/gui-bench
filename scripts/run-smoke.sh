#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_dir"
exec cargo run -p gui-bench -- run --preset smoke --diagnostic --repetitions 1 "$@"

