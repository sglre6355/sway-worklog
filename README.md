# sway-worklog

Automated work hour tracker for [Sway](https://swaywm.org/). Monitors workspace switch events via Sway IPC to log when work starts and stops — no manual time tracking needed.

## How it works

You designate certain Sway workspaces as "work" in a config file. A daemon subscribes to workspace focus events and writes start/stop entries to a JSONL log. Reporting commands read the log to show totals.

## Setup

Build and install:

```sh
cargo build --release
cp target/release/sway-worklog ~/.local/bin/
```

Create a config file at `~/.config/sway-worklog/config.toml`:

```toml
work_workspaces = ["2", "3"]   # workspace names considered "work"
idle_timeout_minutes = 30       # close session after inactivity
# log_path = "..."              # optional, default: ~/.local/share/sway-worklog/worklog.jsonl
```

## Usage

```
sway-worklog daemon                              # run event listener (foreground)
sway-worklog summary [--week|--date YYYY-MM-DD] [--json]
sway-worklog status                              # is a work session active?
sway-worklog log [--from DATE] [--to DATE]       # dump raw entries
```

To run the daemon automatically, add to your Sway config:

```
exec sway-worklog daemon
```

### Example output

```
Work summary for 2026-03-10
===========================
  code      3h 42m
  work      1h 15m
  -------------------------
  Total     4h 57m

Sessions:
  09:15 - 12:57  code   (3h 42m)
  14:02 - 15:17  work   (1h 15m)
```

## Log format

Each line in the JSONL log is a start or stop event:

```json
{"type":"start","workspace":"2","timestamp":"2026-03-10T09:15:32+09:00"}
{"type":"stop","workspace":"2","timestamp":"2026-03-10T12:03:11+09:00","reason":"switch"}
```

Stop reasons: `switch`, `shutdown`, `signal`, `idle`, `workspace_change`.

Start/stop pairs are used instead of single duration records so that crashes only lose at most one session boundary. Orphaned starts (missing stop) are detected by the summary logic and use the current time as a tentative end.

## Daemon behavior

The daemon tracks an `active_session` and reacts to workspace focus events:

| Current state | New workspace is work? | Action |
|---|---|---|
| No session | Yes | Write `start` |
| Active session | No | Write `stop` (reason: switch) |
| Active session (different ws) | Yes | Write `stop` (workspace_change) + `start` |
| Active session (same ws) | Yes | No-op |

Additionally:
- **Shutdown event** — writes `stop` (reason: shutdown), exits
- **SIGINT/SIGTERM** — writes `stop` (reason: signal), exits
- **Idle detection** — if time since last event exceeds `idle_timeout_minutes`, backdates a `stop` to the last event time
