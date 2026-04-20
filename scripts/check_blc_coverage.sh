#!/usr/bin/env bash
# List BLC identifiers declared in docs/business-logic-constraints/*.md and report
# which never appear in backend test sources (Venom `# BLC:` lines, Rust `BLC-…` in
# http_tests / audit_events_tests).
#
# Usage:
#   ./scripts/check_blc_coverage.sh
#   STRICT=1 ./scripts/check_blc_coverage.sh   # exit 1 if any doc BLC is unreferenced in tests
#
# Always prints a summary to stdout. Exit 0 by default; STRICT=1 exits 1 when the
# "missing" list is non-empty.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DOCS_GLOB="$ROOT/docs/business-logic-constraints/*.md"
TEST_DIRS=(
  "$ROOT/backend/tests"
  "$ROOT/backend/src/http_tests.rs"
  "$ROOT/backend/src/audit_events_tests.rs"
)

if ! compgen -G "$DOCS_GLOB" >/dev/null; then
  echo "error: no BLC docs at $DOCS_GLOB" >&2
  exit 2
fi

IDS=$(grep -rhoE 'BLC-[A-Z]+-[0-9]+[a-z]?' $DOCS_GLOB | sort -u)
IDS_COUNT=$(printf '%s\n' "$IDS" | grep -c . || true)

missing=()
while IFS= read -r id; do
  [[ -z "$id" ]] && continue
  hit=0
  for path in "${TEST_DIRS[@]}"; do
    if [[ -f "$path" ]] && grep -q "$id" "$path" 2>/dev/null; then
      hit=1
      break
    fi
    if [[ -d "$path" ]] && grep -rq "$id" "$path" 2>/dev/null; then
      hit=1
      break
    fi
  done
  if [[ "$hit" -eq 0 ]]; then
    missing+=("$id")
  fi
done <<< "$IDS"

echo "BLC ids in docs: $IDS_COUNT"
echo "Unreferenced in backend/tests + http_tests + audit_events_tests: ${#missing[@]}"
if ((${#missing[@]})); then
  printf '%s\n' "${missing[@]}"
  if [[ "${STRICT:-0}" == 1 ]]; then
    exit 1
  fi
fi
exit 0
