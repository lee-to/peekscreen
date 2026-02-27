# screenshot-mcp

Universal MCP server for capturing application window and screen screenshots. Works with any MCP-compatible client (Claude Desktop, etc.).

Cross-platform support via [xcap](https://crates.io/crates/xcap): macOS, Windows, Linux.

## Installation

### Build from source

```bash
git clone https://github.com/user/screenshot-mcp.git
cd screenshot-mcp
cargo install --path .
```

### From crates.io

```bash
cargo install screenshot-mcp
```

## Configuration

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "screenshot": {
      "command": "screenshot-mcp"
    }
  }
}
```

Or if running from a local build:

```json
{
  "mcpServers": {
    "screenshot": {
      "command": "/path/to/screenshot-mcp"
    }
  }
}
```

### Environment variables

- `RUST_LOG` — Control log verbosity (default: `screenshot_mcp=info`). Logs are written to stderr.

## macOS permissions

On macOS, Screen Recording permission is required. The first time the server tries to capture a screenshot, macOS will prompt for permission.

If you don't see a prompt, go to **System Settings → Privacy & Security → Screen Recording** and add your terminal app or Claude Desktop.

## Tools

| Tool | Description | Parameters |
|------|-------------|------------|
| `list_windows` | List all visible windows | None |
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
cargo test            # unit tests
cargo test -- --ignored  # integration tests (requires display + permissions)
```
