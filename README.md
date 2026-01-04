pane [![Release](https://github.com/ecsumed/pane/actions/workflows/release.yml/badge.svg)](https://github.com/ecsumed/pane/actions/workflows/release.yml)
======

`watch` plus `tmux-resurrect`. Heavily inspired by [hwatch](https://github.com/blacknon/hwatch/tree/master).

## Install
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ecsumed/pane/releases/download/v0.1.0/pane-installer.sh | sh
```

## Features
* Multiple panes (via Tokio Async)
* Session load/save
* Records command results for viewing history/diffs
* Search through command output
* Display as raw text, multiline, counter, sparkline, diff, etc.

## TODO
* Remove `pane_key_to_friendly_id`
* Fix infinte scroll
* Fix move into Search in observe mode
* Hide Search for non-plain modes?
* Conver chrono to jiff?
* Fix interval hang on inc/dec
* Improve resize functionality
* Improve move functionality
* More tests
