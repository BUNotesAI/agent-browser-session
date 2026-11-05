---
name: agent-browser-session
description: Browser automation CLI for AI agents with persistent login and multi-tab isolation. Use when the user needs to interact with websites — navigating pages, filling forms, clicking buttons, taking screenshots, extracting data, testing web apps, or automating any browser task. Triggers on "open a website", "fill out a form", "click a button", "take a screenshot", "scrape data", "test this web app", "login to a site", "browse multiple sites", or any task requiring programmatic web interaction.
allowed-tools: Bash(agent-browser-session:*)
---

# Browser Automation with agent-browser-session

Uses **Patchright** (anti-detection Playwright fork) + system Chrome + persistent profile. Headed by default for login persistence.

## Binary

Always use `agent-browser-session`. Do NOT use `agent-browser` (upstream) — it's headless/ephemeral and gets blocked by Google, Twitter, etc.

```bash
agent-browser-session open https://example.com    # default tab (ZEROTABPAGE)
agent-browser-session --tabname reddit open https://reddit.com  # named tab
```

## Core Workflow

Every browser automation follows this pattern:

1. **Navigate**: `agent-browser-session open <url>`
2. **Snapshot**: `agent-browser-session snapshot -i` (get element refs like `@e1`, `@e2`)
3. **Interact**: Use refs to click, fill, select
4. **Re-snapshot**: After navigation or DOM changes, get fresh refs

```bash
agent-browser-session open https://example.com/form
agent-browser-session snapshot -i
# Output: @e1 [input "email"], @e2 [input "password"], @e3 [button "Submit"]

agent-browser-session fill @e1 "user@example.com"
agent-browser-session fill @e2 "password123"
agent-browser-session click @e3
agent-browser-session wait --load networkidle
agent-browser-session snapshot -i  # Check result
```

## Multi-Tab Isolation (`--tabname`)

Multiple agents can operate independent tabs in the same browser. Each tab has its own Page, CDP session, snapshot refs, and frame context. Cookies are shared (login once, all tabs see it).

```bash
# Agent A: browse Reddit
agent-browser-session --tabname reddit open https://reddit.com
agent-browser-session --tabname reddit snapshot -i
agent-browser-session --tabname reddit click @e5

# Agent B: simultaneously browse Hacker News
agent-browser-session --tabname hackernews open https://news.ycombinator.com
agent-browser-session --tabname hackernews snapshot -i
agent-browser-session --tabname hackernews click @e3
```

Without `--tabname`, commands go to a default `ZEROTABPAGE` tab. You can mix named and default freely.

### Concurrent Operation

Named tabs support true parallel operation:

```bash
agent-browser-session --tabname tab-a open https://reddit.com &
agent-browser-session --tabname tab-b open https://news.ycombinator.com &
wait
```

### Tab Isolation Guarantees

| Isolated per tab | Shared across tabs |
|------------------|--------------------|
| Page (DOM, URL) | Cookies, localStorage |
| CDP session | Browser profile |
| Snapshot refs (`@e1`, `@e2`) | Chrome process |
| Frame context | |

## Command Chaining

Commands can be chained with `&&`. The browser persists via a background daemon.

```bash
agent-browser-session open https://example.com && agent-browser-session wait --load networkidle && agent-browser-session snapshot -i
```

Chain when you don't need intermediate output. Run separately when you need to parse snapshot refs first.

## Essential Commands

```bash
# Navigation
agent-browser-session open <url>              # Navigate
agent-browser-session back                    # Go back
agent-browser-session forward                 # Go forward
agent-browser-session reload                  # Reload page

# Snapshot (always use -i for interactive elements)
agent-browser-session snapshot -i             # Interactive elements with refs
agent-browser-session snapshot -i --compact   # Compact mode (fewer structural elements)
agent-browser-session snapshot -i -s "#main"  # Scope to CSS selector
agent-browser-session snapshot -i -d 3        # Limit depth

# Interaction (use @refs from snapshot)
agent-browser-session click @e1               # Click element
agent-browser-session fill @e2 "text"         # Clear and type text
agent-browser-session type @e2 "text"         # Type without clearing
agent-browser-session select @e1 "option"     # Select dropdown option
agent-browser-session check @e1               # Check checkbox
agent-browser-session uncheck @e1             # Uncheck checkbox
agent-browser-session press Enter             # Press key
agent-browser-session hover @e1               # Hover over element
agent-browser-session scroll down 500         # Scroll page
agent-browser-session scrollintoview @e1      # Scroll element into view

# Get information
agent-browser-session get text @e1            # Get element text
agent-browser-session get html @e1            # Get element HTML
agent-browser-session get url                 # Get current URL
agent-browser-session get title               # Get page title
agent-browser-session get value @e1           # Get input value
agent-browser-session get attr @e1 href       # Get attribute
agent-browser-session get count "li.item"     # Count elements

# Wait
agent-browser-session wait @e1                # Wait for element
agent-browser-session wait --load networkidle # Wait for network idle
agent-browser-session wait --url "**/page"    # Wait for URL pattern
agent-browser-session wait --fn "window.ready"# Wait for JS condition
agent-browser-session wait --text "Welcome"   # Wait for text
agent-browser-session wait 2000               # Wait milliseconds

# Capture
agent-browser-session --json screenshot       # Screenshot base64 in data.base64
agent-browser-session screenshot page.png     # Screenshot to file
agent-browser-session screenshot --full       # Full page screenshot
agent-browser-session pdf output.pdf          # Save as PDF

# Tabs
agent-browser-session tab list                # List all tabs (includes named tabs)
agent-browser-session tab new                 # New tab
agent-browser-session --tabname new-page open https://url  # New named page with URL
agent-browser-session tab 2                   # Switch to tab index 2
agent-browser-session tab close               # Close current tab

# Cookies & Storage
agent-browser-session cookies                 # Get all cookies
agent-browser-session cookies get             # Get cookies
agent-browser-session cookies set name value  # Set cookie
agent-browser-session cookies clear           # Clear cookies
agent-browser-session storage local           # Get all localStorage
agent-browser-session storage local get key   # Get specific key
agent-browser-session storage local set k v   # Set key-value

# JavaScript
agent-browser-session eval "document.title"
agent-browser-session eval "document.querySelectorAll('a').length"
```

## Common Patterns

### Form Submission

```bash
agent-browser-session open https://example.com/signup
agent-browser-session snapshot -i
agent-browser-session fill @e1 "Jane Doe"
agent-browser-session fill @e2 "jane@example.com"
agent-browser-session select @e3 "California"
agent-browser-session check @e4
agent-browser-session click @e5
agent-browser-session wait --load networkidle
```

### Login Flow (Persistent)

```bash
# First time: login manually in the headed browser window
agent-browser-session open https://github.com/login
# ... user logs in ...

# Later: already logged in (cookies persisted in profile)
agent-browser-session open https://github.com/settings
```

### Data Extraction

```bash
agent-browser-session open https://example.com/products
agent-browser-session snapshot -i
agent-browser-session get text @e5           # Get specific element text

# JSON output for parsing
agent-browser-session snapshot -i --json
agent-browser-session get text @e1 --json
```

### Multi-Agent Parallel Browsing

```bash
# Agent 1: research on Reddit
agent-browser-session --tabname reddit open https://reddit.com
agent-browser-session --tabname reddit snapshot -i
agent-browser-session --tabname reddit click @e12

# Agent 2: simultaneously research on HN
agent-browser-session --tabname hackernews open https://news.ycombinator.com
agent-browser-session --tabname hackernews snapshot -i
agent-browser-session --tabname hackernews click @e5

# Both tabs independent — each has its own refs, page, CDP session
```

### Connect via CDP

```bash
agent-browser-session --cdp 9222 snapshot
```

## Ref Lifecycle (Important)

Refs (`@e1`, `@e2`, etc.) are invalidated when the page changes. Always re-snapshot after:

- Clicking links or buttons that navigate
- Form submissions
- Dynamic content loading (dropdowns, modals)

```bash
agent-browser-session click @e5              # Navigates to new page
agent-browser-session snapshot -i            # MUST re-snapshot
agent-browser-session click @e1              # Use new refs
```

With `--tabname`, each tab has **independent refs**. Snapshotting one tab doesn't affect another's refs.

## Semantic Locators (Alternative to Refs)

When refs are unavailable or unreliable:

```bash
agent-browser-session find text "Sign In" click
agent-browser-session find label "Email" fill "user@test.com"
agent-browser-session find role button click --name "Submit"
agent-browser-session find placeholder "Search" fill "query"
agent-browser-session find testid "submit-btn" click
```

### 已核对的参数与限制

以下行为同时由原生 CLI 参数解析和 daemon schema 回归覆盖。省略可选参数时不会发送 JSON null；命令返回成功仍须用页面状态或实际产物核对动作效果。

|命令|实际用法|
|---|---|
|截图|`screenshot`、`screenshot --full` 可省略路径；daemon 返回 base64，CLI 需加 `--json` 才读取 data.base64，默认文本只显示 Done。`screenshot /absolute/page.png` 写文件|
|请求记录|`network requests` 可不带 filter；`network requests --clear` 清空；`network requests --filter api` 按 URL 子串筛选。先调用 requests 开始记录，再触发请求|
|撤销拦截|`network unroute` 撤销全部；`network unroute "**/api/*"` 只撤销匹配项|
|响应模拟|`network route "**/fixture.json" --body '{"ok":true}'` 将原文作为响应 body；`--abort` 中止请求|
|同源请求头|`open https://example.com --headers '{"X-Probe":"fixture"}'` 将 headers 传给既有同源限定处理；不要在 argv 中放真实凭据|
|trace|`trace start` 开始，`trace stop /absolute/trace.zip` 保存；stop 必须有路径|
|tab|`tab new` 只建未命名空白页；带 URL 的新页面用 `--tabname <unique-name> open <url>`。`tab close` 关闭全局当前页，`tab close 2` 按非负全局索引关闭；这两个旧索引操作不提供具名 tab 隔离|
|frame|`frame "#child"` 进入 iframe，`frame main` 回主 frame|
|mouse|`mouse move <x> <y>`、`mouse down [button]`、`mouse up [button]`；`mouse wheel <deltaY> [deltaX]` 支持有符号偏移|
|media|`set media dark`、`set media light reduced-motion`；映射 colorScheme 与字符串 reducedMotion 枚举|
|dialog|`dialog accept` 接受默认提示，`dialog accept "fixture text"` 给 prompt 文本，`dialog dismiss` 取消|

find 默认执行 click；`--name <name>` 仅 role 支持，位置可在动作前后，不会混入 fill 文本。动作表：

|locator|允许的动作|匹配选项|
|---|---|---|
|role、testid|click、fill、check、hover|role 可用 --name|
|label|click、fill、check|无 --exact|
|placeholder|click、fill|无 --exact|
|text、alt、title|click、hover|可用 --exact|
|first、last、nth|click、fill、check、hover、text|nth 索引以0起；last 选择末项|

限制：`tab new <url>`、`trace start <path>`、role/label/placeholder 等不支持的 `--exact` 及表外 find 动作会在 CLI 明确报错。前两项需要新增协议字段和执行语义，不在本次字段映射修复中扩展；trace 的输出路径只在 stop 指定。需要逐字匹配 role/label/placeholder 时，改用已核对的 selector/ref 定位方式，不声称被忽略的参数已生效。

0.4.6 本轮实测还暴露了两项执行层限制：`find last` 的请求可通过 schema，但目标 DOM 已确认存在时仍有 locator.last() 超时，先用已核对的非负 `find nth` 索引；具名 tab 的普通 selector 读取当前走 binding.page，`frame` 之后的 `get text` 不随 activeFrame 切换。未在字段映射修复中改这些执行语义，也不将命令返回成功当作跨 frame 功能已验证。

trace 的 screenshots/snapshots 虽有 daemon 字段，CLI 当前未暴露对应选项；本轮不扩展这些开关。无路径截图的默认文本显示和这些 trace 开关均保留现状。

隔离实测或并行工作需要同时隔离 IPC 与浏览器 profile 时，设置独立的 `AGENT_BROWSER_SOCKET_DIR=/absolute/task/browser-data`；仅 `--tabname` 仍共享默认浏览器和登录信息。不要在隔离实测中操作默认 main 会话，也不要用 `kill` 清其他会话。

## Options

| Option | Description |
|--------|-------------|
| `--tabname <name>` | Named tab for client isolation |
| `--headed [true\|false]` | Browser mode (default: `true`) |
| `--bundled` | Use bundled Chrome instead of system Chrome |
| `--channel <name>` | Browser channel: `chrome`, `msedge` |
| `--executable-path <path>` | Custom browser executable |
| `--cdp <port>` | Connect via Chrome DevTools Protocol |
| `--json` | JSON output |
| `--version`, `-V` | Show version |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AGENT_BROWSER_SOCKET_DIR` | IPC 文件及 headed/headless profile 的根目录；隔离实测应每任务独立设置 | `$XDG_RUNTIME_DIR/agent-browser`，否则 `~/.agent-browser` |
| `AGENT_BROWSER_TABNAME` | Named tab | `ZEROTABPAGE` |
| `AGENT_BROWSER_HEADED` | Browser mode | `true` |
| `AGENT_BROWSER_NAV_DELAY_MS` | Navigation delay (ms) | `5000` |
| `AGENT_BROWSER_EXECUTABLE_PATH` | Custom browser path | system Chrome |
| `AGENT_BROWSER_STREAM_PORT` | WebSocket streaming port | disabled |

## Rate Limiting

5-second delay before each navigation by default.

```bash
AGENT_BROWSER_NAV_DELAY_MS=0 agent-browser-session open example.com     # Disable
AGENT_BROWSER_NAV_DELAY_MS=2000 agent-browser-session open example.com  # Custom
```

## Architecture

```
CLI (Rust) ──→ Unix socket ──→ Daemon (Node.js) ──→ BrowserManager
                ~/.agent-browser/                      │
                sys/main.sock                   tabBindings Map
                                                 ┌─────┴─────┐
                                                 │ reddit │ hn │
                                                 │ Page   │Page│
                                                 └────────┴────┘
                                                       │
                                                System Chrome
                                          ~/.agent-browser/headed-profile/main/
```

- **Session**: Always `main` (hardcoded). Use `--tabname` for isolation.
- **Profile**: `~/.agent-browser/headed-profile/main/` (headed) or `headless-profile/main/` (headless)
- **Close disabled**: Browser is shared; close the window manually when done.
- **Daemon detection**: Socket-only (no PID-based checking).

## Timeouts

Default timeout is 10 seconds. Use explicit waits for slow pages:

```bash
agent-browser-session wait --load networkidle    # Wait for network to settle
agent-browser-session wait "#slow-element"       # Wait for specific element
agent-browser-session wait --fn "window.loaded"  # Wait for JS condition
```
