# TODO

## Polish
- [x] Suppress dead code warnings with `#[allow(dead_code)]` on deserialized fields
- [x] Add colored output for priorities and statuses (colored crate already included)

## Features
- [x] Add `--json` output flag for scripting/piping
- [x] Shell completions via `clap_complete` (bash, zsh, fish)
- [x] Add `linear issue close ENG-123` shorthand command
- [x] Pagination support for large result sets (`--all` flag)
- [x] Labels support (list, filter by, add to issues)
- [x] Comments support (view, add)

## Quality of Life
- [x] Better date formatting with `chrono` crate
- [x] Cache team/project lookups to reduce API calls
- [x] Add `linear init` command to interactively create config file
- [x] Support issue ID lookup by identifier (ENG-123) in addition to UUID

## Documentation
- [x] Add README with installation and usage examples
- [x] Add `--help` examples for each subcommand
