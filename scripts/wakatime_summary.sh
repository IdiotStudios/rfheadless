#!/usr/bin/env bash
# Small helper to summarize WakaTime activity for this project
# Requires `wakatime-cli` (https://github.com/wakatime/wakatime-cli)

if command -v wakatime-cli >/dev/null 2>&1; then
  echo "WakaTime CLI found. Showing last 7 days summary for project 'rfheadless':"
  wakatime-cli --project rfheadless summarize --period "last_7_days"
else
  echo "wakatime-cli not found. Install it or use the WakaTime dashboard">
  echo "Dashboard: https://wakatime.com/"
  echo "If you use the official plugin, you can also view local data via ~/.wakatime or the CLI."
fi