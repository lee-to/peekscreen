# peekscreen

[![CI](https://github.com/lee-to/peekscreen/actions/workflows/ci.yml/badge.svg)](https://github.com/lee-to/peekscreen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

MCP server for capturing window and screen screenshots. Works with any MCP-compatible client (Claude Desktop, Claude Code, etc.).

Currently tested on **macOS**. Windows and Linux support is possible via [xcap](https://crates.io/crates/xcap) but not yet verified.

## Quick start

**1. Install:**

### macOS (Apple Silicon)

```bash
curl -fsSL https://github.com/lee-to/peekscreen/releases/latest/download/peekscreen-aarch64-apple-darwin.tar.gz | tar xz -C /usr/local/bin
```

### macOS (Intel)

```bash
curl -fsSL https://github.com/lee-to/peekscreen/releases/latest/download/peekscreen-x86_64-apple-darwin.tar.gz | tar xz -C /usr/local/bin
```

**2. Add to your AI agent:**

```bash
# Claude Code
claude mcp add peekscreen -- peekscreen
```

Done! The `peekscreen` server is now available in your agent.

## Build from source

```bash
git clone https://github.com/lee-to/peekscreen.git
cd peekscreen
cargo install --path .
```

## Configuration

### Claude Desktop

Add to your config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "peekscreen": {
      "command": "peekscreen"
    }
  }
}
```

### Environment variables

- `RUST_LOG` — Control log verbosity (default: `peekscreen=info`). Logs are written to stderr.

## macOS permissions

On macOS, **Screen Recording** permission is required for the process that launches the MCP server.

If you run an AI agent from a terminal (e.g. Claude Code in iTerm2, Terminal.app, Warp, etc.), you must grant Screen Recording permission **to that terminal app** — not just to Claude Desktop.

Go to **System Settings → Privacy & Security → Screen Recording** and add the relevant app. A restart of the terminal may be required after granting permission.

> **Troubleshooting:** If `list_windows` returns an empty list or a window you can see on screen is missing from the results, the problem is almost certainly missing Screen Recording permission for the app that runs the MCP server.

## Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_windows` | List all visible windows | — |
| `screenshot_window` | Capture a window screenshot | `title?`, `id?`, `max_width?`, `format?` |
| `screenshot_screen` | Capture an entire screen | `monitor_id?`, `max_width?`, `format?` |

### list_windows

Returns a JSON array of all visible windows with: `id`, `title`, `app_name`, `width`, `height`, `is_focused`.

### screenshot_window

Capture a specific window. Find by:
- **title** — case-insensitive substring match (e.g., `"Visual Studio"`)
- **id** — exact window ID from `list_windows`
- **no params** — captures the currently focused window

Optional: `max_width` (default: 1920), `format` (`"png"` or `"jpeg"`, default: `"png"`).

### screenshot_screen

Capture an entire monitor. Defaults to the primary monitor. Specify `monitor_id` for a different monitor.

Optional: `max_width` (default: 1920), `format` (`"png"` or `"jpeg"`, default: `"png"`).

## Development

```bash
cargo build
cargo test              # unit tests
cargo test -- --ignored # integration tests (requires display + permissions)
cargo clippy            # lints
cargo fmt --check       # formatting
```

## License

[MIT](LICENSE)
