# agent-browser-session

A fork of [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) optimized for **persistent login sessions** and **multi-tab parallel operation**. Uses [Patchright](https://github.com/AjaxMultiCommentary/patchright) (anti-detection Playwright fork) + system Chrome + persistent userDataDir.

## Why This Fork?

| | `agent-browser` (upstream) | `agent-browser-session` (this fork) |
|---|---|---|
| Browser | Bundled Chromium | System Chrome |
| Profile | Ephemeral (lost on close) | Persistent (`~/.agent-browser/headed-profile/main/`) |
| Mode | Headless by default | Headed by default |
| Anti-detection | Playwright (detectable) | Patchright (stealth) |
| Multi-tab | Shared `activePageIndex` (conflict) | `--tabname` isolation (parallel-safe) |
| Best for | Quick scraping, no login | OAuth, login flows, multi-agent browsing |

**Both can coexist** — install upstream via `npm i -g agent-browser` and this fork separately.

## Installation

```bash
git clone https://github.com/BUNotesAI/agent-browser-session
cd agent-browser-session
pnpm install
pnpm build
pnpm build:native   # Requires Rust (https://rustup.rs)
pnpm link --global   # Makes agent-browser-session available globally
```

## Quick Start

```bash
agent-browser-session open example.com
agent-browser-session snapshot -i               # Get interactive elements with refs
agent-browser-session click @e2                  # Click by ref
agent-browser-session fill @e3 "test@example.com"
agent-browser-session screenshot page.png
```

## Login Persistence

The browser profile is stored persistently. Login once, stay logged in across sessions.

```bash
# First time: login manually in the browser window
agent-browser-session open https://accounts.google.com

# Later: login state persists automatically
agent-browser-session open https://mail.google.com  # Already logged in!
```

## Multi-Tab Isolation (`--tabname`)

Multiple CLI clients can operate independent tabs in the same browser instance. Each tab has its own Page, CDP session, snapshot refs, and frame context.

```bash
# Agent A: browse Reddit
agent-browser-session --tabname reddit open https://reddit.com
agent-browser-session --tabname reddit snapshot -i
agent-browser-session --tabname reddit click @e5

# Agent B: simultaneously browse Hacker News (same browser, different tab)
agent-browser-session --tabname hackernews open https://news.ycombinator.com
agent-browser-session --tabname hackernews snapshot -i
agent-browser-session --tabname hackernews click @e3
```

Tab isolation guarantees:

| Isolated per tab | Shared across tabs |
|------------------|--------------------|
| Page (DOM, URL) | Cookies, localStorage |
| CDP session | Browser profile |
| Snapshot refs (`@e1`, `@e2`) | Chrome process |
| Frame context | |

Without `--tabname`, commands default to a built-in `ZEROTABPAGE` tab — this is always safe and backward-compatible.

### Concurrent Operation

Named tabs support true parallel operation:

```bash
# Run simultaneously — both navigate at the same time
agent-browser-session --tabname tab-a open https://reddit.com &
agent-browser-session --tabname tab-b open https://news.ycombinator.com &
wait
```

## Commands

All commands are compatible with upstream `agent-browser`. See the [upstream README](https://github.com/vercel-labs/agent-browser#readme) for the full command reference.

### Core Commands

```bash
agent-browser-session open <url>              # Navigate to URL
agent-browser-session click <sel>             # Click element
agent-browser-session fill <sel> <text>       # Clear and fill
agent-browser-session press <key>             # Press key
agent-browser-session snapshot                # Accessibility tree with refs
agent-browser-session snapshot -i             # Interactive elements only
agent-browser-session screenshot [path]       # Take screenshot
```

### Get Info

```bash
agent-browser-session get text <sel>          # Get text content
agent-browser-session get url                 # Get current URL
agent-browser-session get title               # Get page title
```

### Wait

```bash
agent-browser-session wait <selector>         # Wait for element
agent-browser-session wait --load networkidle # Wait for network idle
agent-browser-session wait --url "**/dash"    # Wait for URL pattern
```

### Tabs

```bash
agent-browser-session tab                     # List tabs (includes named tabs)
agent-browser-session tab new [url]           # New tab
agent-browser-session tab <n>                 # Switch to tab n
agent-browser-session tab close [n]           # Close tab
```

### Cookies & Storage

```bash
agent-browser-session cookies                 # Get all cookies
agent-browser-session storage local           # Get localStorage
```

## Options

| Option | Description |
|--------|-------------|
| `--tabname <name>` | Named tab for client isolation (parallel-safe) |
| `--headed [true\|false]` | Browser mode (default: `true`, use `--headed false` for headless) |
| `--bundled` | Use bundled Chrome instead of system Chrome |
| `--channel <name>` | Browser channel: `chrome`, `msedge`, `chrome-beta` |
| `--executable-path <path>` | Custom browser executable |
| `--json` | JSON output for agents |
| `--version`, `-V` | Show version |

## Headed / Headless Mode

Default is **headed** (visible browser window). Headed and headless profiles are physically isolated to prevent headless from polluting auth cookies.

```bash
# Headed (default) — uses ~/.agent-browser/headed-profile/main/
agent-browser-session open https://example.com

# Headless — uses ~/.agent-browser/headless-profile/main/
agent-browser-session --headed false open https://example.com
```

## Rate Limiting

A **5-second delay** before each navigation by default, to be friendly to target servers.

```bash
# Disable delay for faster testing
AGENT_BROWSER_NAV_DELAY_MS=0 agent-browser-session open example.com

# Custom delay
AGENT_BROWSER_NAV_DELAY_MS=2000 agent-browser-session open example.com
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AGENT_BROWSER_TABNAME` | Named tab for isolation | `ZEROTABPAGE` |
| `AGENT_BROWSER_HEADED` | Browser mode | `true` |
| `AGENT_BROWSER_NAV_DELAY_MS` | Navigation delay (ms) | `5000` |
| `AGENT_BROWSER_EXECUTABLE_PATH` | Custom browser path | system Chrome |
| `AGENT_BROWSER_STREAM_PORT` | WebSocket streaming port | disabled |
| `AGENT_BROWSER_SOCKET_DIR` | Override base directory | `~/.agent-browser` |

## Architecture

```
┌──────────────────┐    Unix Socket     ┌───────────────────┐
│  Rust CLI         │◄─────────────────►│  Node.js Daemon    │
│  (fast startup)   │  ~/.agent-browser/ │  (Patchright)      │
└──────────────────┘   sys/main.sock    └───────────────────┘
                                                  │
                                         tabBindings: Map
                                          ┌───────┴───────┐
                                          │ reddit  │ hn   │
                                          │ Page    │ Page  │
                                          │ CDP     │ CDP   │
                                          │ RefMap  │ RefMap │
                                          └─────────┴───────┘
                                                  │
                                         ┌────────┴────────┐
                                         │  System Chrome    │
                                         │  ~/.agent-browser │
                                         │  /headed-profile/ │
                                         │  main/            │
                                         └──────────────────┘
```

### Directory Layout

```
~/.agent-browser/
├── sys/                          ← IPC files (socket, pid)
│   ├── main.sock
│   └── main.pid
├── headed-profile/               ← Headed browser data (cookies, auth)
│   └── main/
└── headless-profile/             ← Headless browser data (isolated)
    └── main/
```

### Key Design Decisions

- **Socket-only daemon detection** — No PID-based checking (avoids PID reuse false positives)
- **Tabname as identity** — `--tabname` routes commands; no client ID or connection tracking needed
- **Close disabled** — Browser is shared across tabs; close the window manually when done
- **ZEROTABPAGE sentinel** — Commands without `--tabname` auto-assign to `ZEROTABPAGE`, ensuring all commands route through the same tab isolation path

## Using with Both Versions

```bash
# Quick scraping (upstream, headless, ephemeral)
agent-browser open example.com
agent-browser snapshot -i
agent-browser close

# Login-required sites (this fork, headed, persistent)
agent-browser-session open https://github.com/settings
# Already logged in from previous session!

# Multi-agent parallel browsing (this fork)
agent-browser-session --tabname agent-1 open https://reddit.com &
agent-browser-session --tabname agent-2 open https://news.ycombinator.com &
```

## Claude Code Skill

```bash
cp -r skills/agent-browser ~/.claude/skills/
```

## License

Apache-2.0
