#!/usr/bin/env bash
# Append a session summary to WORKING.md
# Usage:
#  ./scripts/log_session.sh --auto [-m "note"]
#  ./scripts/log_session.sh --duration 90 -m "Worked on microtasks"
#  ./scripts/log_session.sh --project myproject --auto

set -euo pipefail
PROJECT="rfheadless"
MSG=""
DURATION_MIN=""
AUTO=false

print_help() {
  cat <<'EOF'
Usage: log_session.sh [--auto] [--duration MINUTES] [-m MESSAGE] [--project NAME]

--auto        Try to summarize today's project time from wakatime-cli (if installed)
--duration    Specify duration in minutes to append manually
-m MESSAGE    Optional message/notes for this entry
--project     Project name to query from wakatime (default: rfheadless)

Examples:
  ./scripts/log_session.sh --auto -m "Worked on microtask queue"
  ./scripts/log_session.sh --duration 45 -m "Refactor tests"

You can schedule this script in cron to run daily and append the day's total to WORKING.md.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --auto)
      AUTO=true; shift;;
    --duration)
      DURATION_MIN="$2"; shift 2;;
    -m|--message)
      MSG="$2"; shift 2;;
    --project)
      PROJECT="$2"; shift 2;;
    -h|--help)
      print_help; exit 0;;
    *)
      echo "Unknown arg: $1"; print_help; exit 1;;
  esac
done

WORKING_FILE="WORKING.md"
if [[ ! -f "$WORKING_FILE" ]]; then
  echo "WORKING.md not found in repo root" >&2
  exit 1
fi

compute_from_wakatime() {
  if ! command -v wakatime-cli >/dev/null 2>&1; then
    return 1
  fi
  OUTPUT=$(wakatime-cli --project "$PROJECT" summarize --period today 2>/dev/null || true)
  if [[ -z "$OUTPUT" ]]; then
    return 1
  fi
  # Try to find a 'Total' line
  TOTAL_LINE=$(echo "$OUTPUT" | grep -i 'total' | head -n1 || true)
  if [[ -n "$TOTAL_LINE" ]]; then
    # extract hours and minutes heuristically
    H=$(echo "$TOTAL_LINE" | grep -oE '([0-9]+)\s*hr' | grep -oE '[0-9]+' || true)
    M=$(echo "$TOTAL_LINE" | grep -oE '([0-9]+)\s*min' | grep -oE '[0-9]+' || true)
    if [[ -z "$H" && -n $(echo "$TOTAL_LINE" | grep -oE '[0-9]+:[0-9]+') ]]; then
      HM=$(echo "$TOTAL_LINE" | grep -oE '[0-9]+:[0-9]+' | head -n1)
      H=$(echo "$HM" | cut -d: -f1)
      M=$(echo "$HM" | cut -d: -f2)
    fi
    if [[ -n "$H" || -n "$M" ]]; then
      H=${H:-0}
      M=${M:-0}
      echo $((H*60 + M))
      return 0
    fi
  fi
  # fallback: try to parse any numeric minutes in output
  MINS=$(echo "$OUTPUT" | grep -oE '([0-9]+)\s*min' | tail -n1 | grep -oE '[0-9]+' || true)
  if [[ -n "$MINS" ]]; then
    echo "$MINS"; return 0
  fi
  return 1
}

# Decide duration
if [[ -n "$DURATION_MIN" ]]; then
  DURATION_MIN=$(echo "$DURATION_MIN" | tr -d '[:space:]')
else
  if $AUTO; then
    DUR=$(compute_from_wakatime)
    if [[ $? -eq 0 && -n "$DUR" ]]; then
      DURATION_MIN="$DUR"
    else
      echo "--auto requested but could not determine duration from wakatime-cli." >&2
      echo "Either install wakatime-cli or pass --duration MINUTES." >&2
      exit 1
    fi
  else
    # prompt
    read -rp "Duration (minutes): " DURATION_MIN
    if [[ -z "$DURATION_MIN" ]]; then
      echo "No duration provided; aborting." >&2; exit 1
    fi
  fi
fi

# Build entry
DATE=$(date +"%F")
H=$((DURATION_MIN / 60))
M=$((DURATION_MIN % 60))
DURATION_TEXT="${H}h ${M}m"
MSG_TEXT="${MSG}"
if [[ -n "$MSG_TEXT" ]]; then
  ENTRY="- $DATE: $DURATION_TEXT — $MSG_TEXT"
else
  ENTRY="- $DATE: $DURATION_TEXT"
fi

# Append
printf "%s\n" "$ENTRY" >> "$WORKING_FILE"
printf "Appended to %s: %s\n" "$WORKING_FILE" "$ENTRY"
exit 0
