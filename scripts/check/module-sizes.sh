#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR/.." rev-parse --show-toplevel 2>/dev/null || (cd "$SCRIPT_DIR/.." && pwd))"
TARGET_DIR="${REPO_ROOT}/core/src"
OUTPUT_DIR="${REPO_ROOT}/.spec-workflow/analysis"
CONFIG_DIR="${REPO_ROOT}/.spec-workflow/config"
OUTPUT_JSON="${OUTPUT_DIR}/module-sizes.json"
OUTPUT_HTML="${OUTPUT_DIR}/module-sizes.html"
ALLOWLIST_FILE="${CONFIG_DIR}/module-size-allowlist.json"
THRESHOLD="${MODULE_MAX_LINES:-500}"

if ! command -v tokei >/dev/null 2>&1; then
    echo "tokei is required for module size analysis. Install with 'cargo install tokei'." >&2
    exit 1
fi

mkdir -p "$OUTPUT_DIR" "$CONFIG_DIR"

TOKEI_TMP="$(mktemp)"
trap 'rm -f "$TOKEI_TMP"' EXIT
tokei "$TARGET_DIR" --exclude target --output json --files >"$TOKEI_TMP"

export REPO_ROOT TARGET_DIR OUTPUT_JSON OUTPUT_HTML ALLOWLIST_FILE THRESHOLD TOKEI_TMP

if python3 - <<'PY'
import json
import math
import os
from datetime import datetime, timezone
from pathlib import Path
import sys

repo_root = Path(os.environ["REPO_ROOT"])
target_dir = Path(os.environ["TARGET_DIR"])
threshold = int(os.environ["THRESHOLD"])
output_json = Path(os.environ["OUTPUT_JSON"])
output_html = Path(os.environ["OUTPUT_HTML"])
allowlist_file = Path(os.environ["ALLOWLIST_FILE"])
tokei_tmp = Path(os.environ["TOKEI_TMP"])
with tokei_tmp.open() as f:
    raw = json.load(f)

allowlist_data = {"allow": []}
if allowlist_file.exists():
    with allowlist_file.open() as f:
        allowlist_data = json.load(f)

allow_map = {}
for entry in allowlist_data.get("allow", []):
    path = Path(entry["path"]).as_posix()
    allow_map[path] = {
        "max_lines": int(entry.get("max_lines", threshold)),
        "reason": entry.get("reason", "no reason provided"),
    }

rust = raw.get("Rust", {})
files = []
violations = []
for file_entry in rust.get("reports", []):
    path = Path(file_entry["name"]).resolve()
    if path.suffix != ".rs":
        continue
    if target_dir not in path.parents and path != target_dir:
        continue
    rel_path = path.relative_to(repo_root).as_posix()
    stats = file_entry.get("stats", {})
    code = int(stats.get("code", 0))
    comments = int(stats.get("comments", 0))
    blanks = int(stats.get("blanks", 0))
    allowance = allow_map.get(rel_path, None)
    effective_limit = allowance["max_lines"] if allowance else threshold
    is_violation = code > effective_limit
    split_count = max(2, math.ceil(code / float(threshold))) if code > threshold else 1
    recommendation = (
        f"Split into ~{split_count} modules grouped by responsibility "
        f"so each stays under {threshold} lines (current: {code} code lines)."
    ) if code > threshold else ""
    record = {
        "path": rel_path,
        "code": code,
        "comments": comments,
        "blanks": blanks,
        "allowlisted": allowance is not None,
        "allow_reason": allowance["reason"] if allowance else "",
        "effective_limit": effective_limit,
        "recommendation": recommendation,
    }
    files.append(record)
    if is_violation:
        violations.append(record)

files.sort(key=lambda f: f["code"], reverse=True)
violations.sort(key=lambda f: f["code"], reverse=True)

report = {
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "threshold": threshold,
    "target_dir": target_dir.relative_to(repo_root).as_posix(),
    "allowlist_file": allowlist_file.relative_to(repo_root).as_posix(),
    "summary": {
        "total_files": len(files),
        "over_threshold": sum(1 for f in files if f["code"] > threshold),
        "violations": len(violations),
    },
    "violations": violations,
    "files": files,
}

output_json.write_text(json.dumps(report, indent=2))

html = f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Module Size Dashboard</title>
  <style>
    body {{ font-family: Arial, sans-serif; background: #0b1021; color: #e8eefc; margin: 24px; }}
    h1 {{ margin-bottom: 4px; }}
    .meta {{ color: #9fb3ff; margin-bottom: 16px; }}
    table {{ width: 100%; border-collapse: collapse; margin-top: 12px; }}
    th, td {{ padding: 8px 10px; border-bottom: 1px solid #1f2d4d; text-align: left; font-size: 14px; }}
    th {{ background: #101a33; position: sticky; top: 0; }}
    tr:nth-child(even) {{ background: #0f162d; }}
    .warn {{ color: #ffb347; }}
    .bad {{ color: #ff6b6b; font-weight: 600; }}
    .pill {{ padding: 2px 8px; border-radius: 12px; font-size: 12px; }}
    .pill-ok {{ background: #1f4b2c; color: #8ce99a; }}
    .pill-allow {{ background: #4d380f; color: #ffd166; }}
  </style>
</head>
<body>
  <h1>Module Size Dashboard</h1>
  <div class="meta">
    Threshold: {threshold} code lines · Generated: {report["generated_at"]} · Target: {report["target_dir"]}
  </div>
  <table>
    <thead>
      <tr>
        <th>Path</th>
        <th>Code</th>
        <th>Limit</th>
        <th>Status</th>
        <th>Recommendation</th>
      </tr>
    </thead>
    <tbody>
"""

for f in files:
    status = "OK"
    pill_class = "pill-ok"
    recommendation = f["recommendation"]
    if f["code"] > f["effective_limit"]:
        status = "VIOLATION"
        pill_class = "bad"
    elif f["code"] > threshold:
        status = "ALLOWLISTED" if f["allowlisted"] else "WARNING"
        pill_class = "pill-allow" if f["allowlisted"] else "warn"
    html += f"""      <tr>
        <td>{f["path"]}</td>
        <td>{f["code"]}</td>
        <td>{f["effective_limit"]}</td>
        <td><span class="pill {pill_class}">{status}</span></td>
        <td>{recommendation or f["allow_reason"]}</td>
      </tr>
"""

html += """    </tbody>
  </table>
</body>
</html>
"""

output_html.write_text(html)

if violations:
    sys.exit(2)
PY
then
    echo "Module size analysis complete. Report saved to:"
    echo "  - ${OUTPUT_JSON}"
    echo "  - ${OUTPUT_HTML}"
else
    echo "Module size violations detected (>${THRESHOLD} lines without allowlist):"
    python3 - "$OUTPUT_JSON" <<'PY'
import json
import sys
report = json.load(open(sys.argv[1]))
for v in report["violations"]:
    print(f"- {v['path']}: {v['code']} lines (limit {v['effective_limit']})")
print(f"\nAdd exceptions to {report['allowlist_file']} if needed.")
PY
    exit 1
fi
