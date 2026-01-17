# Ralph Codebase Guidelines for AI Agents

## Project Overview

Ralph is an enterprise-ready autonomous AI agent framework built in Rust. This branch (`ralph/terminal-ux-enhancement`) focuses on creating a best-in-class terminal UX experience leveraging Ghostty terminal features.

## Directory Structure

```
/Users/thinkstudio/ralph/          # Rust source code
├── src/
│   ├── main.rs                    # CLI entry point
│   ├── lib.rs                     # Library exports
│   ├── mcp/                       # MCP server implementation
│   │   ├── mod.rs
│   │   ├── server.rs              # RalphMcpServer struct
│   │   ├── tools/                 # MCP tools
│   │   └── resources/             # MCP resources
│   ├── quality/                   # Quality framework
│   │   ├── mod.rs
│   │   ├── profiles.rs            # Quality profiles
│   │   ├── gates.rs               # Quality gate checker
│   │   └── blog_generator.rs      # Blog post generation
│   ├── integrations/              # External integrations
│   │   ├── mod.rs
│   │   ├── traits.rs              # ProjectTracker trait
│   │   ├── github.rs              # GitHub Projects provider
│   │   ├── linear.rs              # Linear provider
│   │   └── webhooks/              # Webhook server
│   └── ui/                        # Terminal UI (NEW - to be implemented)
│       ├── mod.rs
│       ├── colors.rs
│       ├── display.rs
│       ├── spinner.rs
│       ├── story_view.rs
│       ├── quality_gates.rs
│       ├── summary.rs
│       ├── interrupt.rs
│       └── ghostty.rs
├── Cargo.toml
└── tests/

/Users/thinkstudio/ralphmacchio/   # Task management
├── prd.json                       # User stories for this branch
├── progress.txt                   # Implementation log
├── AGENTS.md                      # This file
└── tasks/                         # Planning documents
```

## Coding Standards

### Rust Conventions
- Run `cargo fmt` after every file change
- Run `cargo clippy -- -D warnings` before committing
- Run `cargo check` for quick type checking
- Use `#![allow(dead_code)]` for scaffolding code

### Module Organization
- Module files at `src/{module}/mod.rs`
- Export public types in mod.rs
- One struct/enum per file for major types

### Testing
- Unit tests in the same file as code (`#[cfg(test)]`)
- Integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests

## Terminal UI Implementation Guidelines

### Crates to Use
- `indicatif` - Progress bars, spinners
- `console` - Terminal detection
- `owo-colors` - 24-bit RGB colors
- `crossterm` - Advanced terminal control

### Color Scheme (24-bit RGB)
```rust
success:     (34, 197, 94)   // Green
error:       (239, 68, 68)   // Red
warning:     (234, 179, 8)   // Yellow
in_progress: (59, 130, 246)  // Blue
muted:       (107, 114, 128) // Gray
story_id:    (34, 211, 238)  // Cyan
```

### Box Drawing Characters
- Rounded corners: `╭ ╮ ╰ ╯`
- Sharp corners: `┌ ┐ └ ┘`
- Lines: `─ │`
- T-joints: `├ ┤ ┬ ┴ ┼`

### Status Icons
- Passed: `✓`
- Failed: `✗`
- Pending: `○`
- Spinner: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`

### Ghostty-Specific Features
```rust
// OSC 8 Hyperlink
"\x1b]8;;{url}\x07{text}\x1b]8;;\x07"

// Title Update
"\x1b]0;{title}\x07"

// Synchronized Output
"\x1b[?2026h"  // Begin
"\x1b[?2026l"  // End
```

## PRD Format

Stories in `prd.json` follow this structure:
```json
{
  "id": "US-001",
  "title": "Story title",
  "description": "As a..., I want..., so that...",
  "acceptanceCriteria": ["AC1", "AC2", ...],
  "priority": 1,
  "passes": false,
  "notes": ""
}
```

## Commit Message Format

```
feat: [US-XXX] - Story title

- Detail 1
- Detail 2

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
```

## Common Commands

```bash
# Quality checks
cargo fmt
cargo clippy -- -D warnings
cargo check
cargo test

# Build
cargo build --release

# Run MCP server
cargo run -- mcp-server --prd /path/to/prd.json
```
