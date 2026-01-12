WORKING LOG (private)

This file is intended for local-only progress tracking and notes. Do not push this file to remote repositories unless explicitly requested.

Format: short entries with date, time spent, brief note, and links to related commits/files.

Example:
- 2026-01-12: 3.5h — Implemented script timeouts and runtime limits; added tests and updated README.

Goals & Milestones
- M1: JS runtime safety (timeouts, runtime limits, microtasks)
- M2: CSSOM & normalization improvements
- M3: Layout prototype & rendering
- M4: Network interception & emulation

WakaTime summary commands
- If you have `wakatime-cli` installed and configured, run:
  wakatime-cli --project rfheadless summarize --period "last_7_days"

Logging sessions automatically
- Use `scripts/log_session.sh --auto -m "short note"` to append today's project total (requires `wakatime-cli`) to `WORKING.md`.
- Or: `scripts/log_session.sh --duration 45 -m "Worked on microtasks"` to append a manual duration.

Crontab example (append daily at 23:59):
- Add to your crontab: `59 23 * * * cd /path/to/rfheadless && ./scripts/log_session.sh --auto -m "daily summary"`

Local notes
- Branch: wip/rfengine (local)
- Keep small, focused commits and update this file with time spent and short descriptions.