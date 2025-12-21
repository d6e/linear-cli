# TODO

## Polish
- [ ] Suppress dead code warnings with `#[allow(dead_code)]` on deserialized fields
- [ ] Add colored output for priorities and statuses (colored crate already included)

## Features
- [ ] Add `--json` output flag for scripting/piping
- [ ] Shell completions via `clap_complete` (bash, zsh, fish)
- [ ] Add `linear issue close ENG-123` shorthand command
- [ ] Pagination support for large result sets
- [ ] Labels support (list, filter by, add to issues)
- [ ] Comments support (view, add)

## Quality of Life
- [ ] Better date formatting with `chrono` crate
- [ ] Cache team/project lookups to reduce API calls
- [ ] Add `linear init` command to interactively create config file
- [ ] Support issue ID lookup by identifier (ENG-123) in addition to UUID

## Documentation
- [ ] Add README with installation and usage examples
- [ ] Add `--help` examples for each subcommand
