# TODO

## Fixes
- [ ] Stagger each pane's first run so a loaded session doesn't start every command at once
- [ ] Show the exit code in the pane border and keep stdout alongside stderr for failed runs

## Undecided
- [ ] Readable display names in pane borders ("Changed words" instead of `DiffWord`)
- [ ] Switch chrono to jiff

## Features
- [ ] BigText display type for labels (`tui-big-text`, largest size that fits the pane)
- [ ] Apply a display type to all panes
- [ ] Observe: mark a run with `m` and diff any two runs
- [ ] Observe: copy the visible output with `y`

## Architecture
- [ ] Central scheduler: cap concurrent commands, jitter, back off on failures
- [ ] Message-driven app state with read-only drawing
- [ ] Display types behind a trait instead of match arms
- [ ] `:` command palette
- [ ] CI on pull requests: fmt, clippy, tests
- [ ] Snapshot tests that don't depend on `ls` of the repo root

## Dashboards
- [ ] Hand-written dashboard files, separate from saved history
- [ ] Per-pane working directory and environment (e.g. `AWS_PROFILE`, `KUBECONFIG`)
- [ ] Dashboard variables that can be switched at runtime

## Signals
- [ ] Extract values from output with a regex
- [ ] Multiple series per chart
- [ ] Alerts and thresholds (border colour, beep, desktop notification)
- [ ] Long-term history in SQLite with CSV export

## Release and docs
- [ ] Homebrew installer
- [ ] Shell completions and man page
- [ ] Demo GIF and generated keybinding reference
- [ ] Publish to crates.io (blocked by the `crokey` git dependency)

## Security
- [ ] Write session files with 0600 permissions
- [ ] Option to not save output history in sessions
