#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_dir="${1:-$HOME/Projects/gpui-ce}"
vendor_dir="$repo_dir/vendor/gpui-ce"

test -f "$source_dir/crates/gpui/Cargo.toml"
mkdir -p "$vendor_dir"
rsync -a --exclude .git --exclude target "$source_dir/" "$vendor_dir/"

revision="$(git -C "$source_dir" rev-parse HEAD 2>/dev/null || true)"
if test -n "$(git -C "$source_dir" status --porcelain 2>/dev/null || true)"; then dirty=true; else dirty=false; fi
generated="$(date --iso-8601=seconds)"
source_display="${source_dir/#$HOME/\~}"
source_display="${source_display//\\/\\\\}"
source_display="${source_display//\"/\\\"}"
printf '{\n  "source": "%s",\n  "revision": "%s",\n  "dirty": %s,\n  "copied_at": "%s",\n  "excluded": [".git", "target"]\n}\n' "$source_display" "$revision" "$dirty" "$generated" > "$vendor_dir/GUI_BENCH_VENDOR.json"

echo "copied GPUI-CE into $vendor_dir; reconcile the benchmark instrumentation before committing"
