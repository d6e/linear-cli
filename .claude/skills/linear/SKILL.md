---
name: linear
description: Manage Linear issues using the `linear` CLI. Use when the user wants to create, view, update, or close Linear issues, list their tasks, check issue status, add comments, attach files, or query teams/projects/cycles.
---

# Linear CLI

A command-line interface for Linear issue tracking. Use this skill when managing issues, tracking work, or querying Linear data.

## Issue Management

### List Issues

```bash
# List issues assigned to you
linear issue list --mine

# Filter by team, status, or project
linear issue list --team ENG
linear issue list --status "In Progress"
linear issue list --project Backend

# Combine filters
linear issue list --mine --status "In Progress" --team ENG

# Get all results (beyond default limit)
linear issue list --mine --all

# Shorthand
linear issues --mine
```

### View Issue Details

```bash
linear issue show ENG-123
```

### Create Issue

```bash
# Basic creation
linear issue create -t "Fix login bug"

# With all options
linear issue create \
  -t "Fix login bug" \
  -d "Users cannot log in with Google OAuth" \
  --team ENG \
  --project Backend \
  --priority 2
```

**Priority levels:** 0=None, 1=Urgent, 2=High, 3=Medium, 4=Low

### Update Issue

```bash
# Update title
linear issue update ENG-123 --title "New title"

# Change status
linear issue update ENG-123 --status "In Review"

# Change priority
linear issue update ENG-123 --priority 1

# Reassign
linear issue update ENG-123 --assignee "user@example.com"

# Multiple updates
linear issue update ENG-123 --status "In Progress" --priority 2
```

### Close Issue

```bash
linear issue close ENG-123
```

## Comments

```bash
# List comments on an issue
linear issue comments ENG-123

# Add a comment
linear issue comment ENG-123 "This is fixed in the latest PR"
```

## Attachments

```bash
# List attachments
linear issue attachments ENG-123

# Attach a URL
linear issue attach ENG-123 https://example.com/doc -t "Design Doc"

# Upload a file
linear issue upload ENG-123 ./screenshot.png -t "Error Screenshot"
```

## Organization Queries

```bash
# List all teams
linear teams

# List projects (optionally filter by team)
linear projects
linear projects --team ENG

# List sprint cycles
linear cycles
linear cycles --team ENG

# List labels
linear labels --team ENG
```

## Output Formats

```bash
# Default: colored terminal tables
linear issues --mine

# JSON output (for scripting)
linear issues --mine --json
```

## Common Workflows

### Start working on a task

```bash
# Find your issues
linear issues --mine --status "Todo"

# Pick one and update status
linear issue update ENG-123 --status "In Progress"
```

### Create issue from bug report

```bash
linear issue create \
  -t "Login fails with special characters in password" \
  -d "Steps to reproduce: ..." \
  --team ENG \
  --priority 2
```

### Review and close completed work

```bash
# Check what's in review
linear issues --mine --status "In Review"

# Close completed issue
linear issue close ENG-456
```

### Add context to an issue

```bash
# Add a comment with findings
linear issue comment ENG-123 "Root cause: race condition in auth handler"

# Attach relevant file
linear issue upload ENG-123 ./debug-log.txt -t "Debug output"
```

## Shell Completions

```bash
# Generate completions for your shell
linear completions bash >> ~/.bashrc
linear completions zsh >> ~/.zshrc
linear completions fish > ~/.config/fish/completions/linear.fish
```
