# linear-cli

A command-line interface for [Linear](https://linear.app) issue tracking.

## Installation

```bash
# Build from source
cargo build --release

# Binary will be at target/release/linear-cli
# Optionally copy to your PATH
cp target/release/linear-cli ~/.local/bin/linear
```

## Configuration

### API Key

Get your API key from Linear: **Settings → Security & Access → Personal API keys**

Either set an environment variable:

```bash
export LINEAR_API_KEY="lin_api_xxxxxxxxxxxxx"
```

Or create a config file at `~/.config/linear/config.toml`:

```toml
api_key = "lin_api_xxxxxxxxxxxxx"
default_team = "ENG"  # optional
```

The environment variable takes precedence over the config file.

## Usage

### Issues

```bash
# List issues
linear issues
linear issues --mine
linear issues --team ENG --status "In Progress"
linear issues --project Backend --limit 50

# Show issue details
linear issue show ENG-123

# Create issue
linear issue create -t "Fix login bug" --team ENG
linear issue create -t "New feature" -d "Description here" --priority 2

# Update issue
linear issue update ENG-123 --status Done
linear issue update ENG-123 --assignee me --priority 1
```

### Attachments

```bash
# List attachments
linear issue attachments ENG-123

# Attach a URL
linear issue attach ENG-123 https://example.com/doc -t "Reference"

# Upload a file
linear issue upload ENG-123 ./screenshot.png
linear issue upload ENG-123 ./report.pdf -t "Monthly report"
```

### Teams, Projects, Cycles

```bash
# List teams
linear teams

# List projects (optionally filter by team)
linear projects
linear projects --team ENG

# List cycles/sprints
linear cycles --team ENG
```

## Priority Values

| Value | Label  |
|-------|--------|
| 0     | None   |
| 1     | Urgent |
| 2     | High   |
| 3     | Medium |
| 4     | Low    |

## License

MIT
