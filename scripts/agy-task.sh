#!/usr/bin/env bash
# scripts/agy-task.sh T03 [model]               run a task
# scripts/agy-task.sh T03 --fix "what failed"   follow-up in same conversation
set -euo pipefail
id="$1"; out=".agy-out/$id.json"; mkdir -p .agy-out
common=(--sandbox --output-format json --print-timeout 30m)
if [ "${2:-}" = "--fix" ]; then
  conv=$(jq -r .conversation_id "$out")
  agy -p "$3" --conversation "$conv" "${common[@]}" > "$out"
else
  args=("${common[@]}")
  if [ -n "${2:-}" ]; then args+=(--model "$2"); fi
  agy -p "Read AGENTS.md first. Then do this task exactly:
$(cat "tasks/$id.md")" "${args[@]}" > "$out"
fi
jq -r '"status: \(.status)", .response' "$out" | tail -n 40
git diff --stat
