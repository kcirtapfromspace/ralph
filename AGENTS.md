# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs Claude Code repeatedly until all PRD items are complete. Each iteration is a fresh Claude Code instance with clean context.

## Commands

```bash
# Install ralph globally
./install.sh

# Initialize a project
ralph --init

# Run Ralph in current directory
ralph

# Run with max 20 iterations
ralph 20

# Run in a specific directory
ralph -d /path/to/project

# Show help
ralph --help

# Run the flowchart dev server
cd flowchart && npm run dev
```

## Key Files

- `bin/ralph` - Global CLI binary
- `install.sh` - Installer script
- `prompt.md` - Instructions given to each Claude Code instance
- `prd.json.example` - Example PRD format
- `flowchart/` - Interactive React Flow diagram explaining how Ralph works

## Flowchart

The `flowchart/` directory contains an interactive visualization built with React Flow. It's designed for presentations - click through to reveal each step with animations.

To run locally:
```bash
cd flowchart
npm install
npm run dev
```

## Patterns

- Each iteration spawns a fresh Claude Code instance with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations
