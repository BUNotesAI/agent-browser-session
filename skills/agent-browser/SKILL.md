---
name: agent-browser
description: Automates browser interactions for web testing, form filling, screenshots, and data extraction. Use when the user needs to navigate websites, interact with web pages, fill forms, take screenshots, test web applications, or extract information from web pages.
---

# Browser Automation with agent-browser

## Quick start

```bash
agent-browser open <url>        # Navigate to page
agent-browser snapshot -i       # Get interactive elements with refs
agent-browser click @e1         # Click element by ref
agent-browser fill @e2 "text"   # Fill input by ref
agent-browser close             # Close browser
```

## Core workflow

1. Navigate: `agent-browser open <url>`
2. Snapshot: `agent-browser snapshot -i` (returns elements with refs like `@e1`, `@e2`)
3. Interact using refs from the snapshot
4. Re-snapshot after navigation or significant DOM changes

## Commands

### Navigation
```bash
agent-browser open <url>      # Navigate to URL
agent-browser back            # Go back
agent-browser forward         # Go forward  
agent-browser reload          # Reload page
agent-browser close           # Close browser
```

### Snapshot (page analysis)
```bash
agent-browser snapshot        # Full accessibility tree
agent-browser snapshot -i     # Interactive elements only (recommended)
agent-browser snapshot -c     # Compact output
agent-browser snapshot -d 3   # Limit depth to 3
```

### Interactions (use @refs from snapshot)
```bash
agent-browser click @e1           # Click
agent-browser dblclick @e1        # Double-click
agent-browser fill @e2 "text"     # Clear and type
agent-browser type @e2 "text"     # Type without clearing
agent-browser press Enter         # Press key
agent-browser press Control+a     # Key combination
agent-browser hover @e1           # Hover
agent-browser check @e1           # Check checkbox
agent-browser uncheck @e1         # Uncheck checkbox
agent-browser select @e1 "value"  # Select dropdown
agent-browser scroll down 500     # Scroll page
agent-browser scrollintoview @e1  # Scroll element into view
```

### Get information
```bash
agent-browser get text @e1        # Get element text
agent-browser get value @e1       # Get input value
agent-browser get title           # Get page title
agent-browser get url             # Get current URL
```

### Screenshots
```bash
agent-browser screenshot          # Screenshot to stdout
agent-browser screenshot path.png # Save to file
agent-browser screenshot --full   # Full page
```

### Wait
```bash
agent-browser wait @e1                     # Wait for element
agent-browser wait 2000                    # Wait milliseconds
agent-browser wait --text "Success"        # Wait for text
agent-browser wait --load networkidle      # Wait for network idle
```

### Semantic locators (alternative to refs)
```bash
agent-browser find role button click --name "Submit"
agent-browser find text "Sign In" click
agent-browser find label "Email" fill "user@test.com"
```

## Example: Form submission

```bash
agent-browser open https://example.com/form
agent-browser snapshot -i
# Output shows: textbox "Email" [ref=e1], textbox "Password" [ref=e2], button "Submit" [ref=e3]

agent-browser fill @e1 "user@example.com"
agent-browser fill @e2 "password123"
agent-browser click @e3
agent-browser wait --load networkidle
agent-browser snapshot -i  # Check result
```

## Persistent State (automatic)

Browser state (cookies, localStorage) is automatically persisted to `~/tmp/agent-browser/`. No need to manually save/load state for authentication:

```bash
# Login once - state is automatically saved
agent-browser open https://app.example.com/login
agent-browser snapshot -i
agent-browser fill @e1 "username"
agent-browser fill @e2 "password"
agent-browser click @e3
agent-browser wait --url "**/dashboard"
agent-browser close

# Later: cookies and localStorage are preserved
agent-browser open https://app.example.com/dashboard  # Already logged in!
```

## Tabs (多标签页)

管理多个浏览器标签页，共享 cookies 和 localStorage：

### 基本命令
```bash
agent-browser tab new              # 创建新 tab
agent-browser tab new <url>        # 创建新 tab 并导航到 URL
agent-browser tab list             # 列出所有 tabs
agent-browser tab <n>              # 切换到第 n 个 tab (从 0 开始)
agent-browser tab close            # 关闭当前 tab
agent-browser tab close <n>        # 关闭第 n 个 tab
```

### 使用场景

**1. 保持登录状态访问多页面**
```bash
agent-browser open https://app.com/login
# ... 完成登录 ...
agent-browser tab new https://app.com/dashboard  # 共享登录态
agent-browser tab new https://app.com/settings   # 同样已登录
```

**2. 外部链接自动追踪**
点击 `target="_blank"` 链接时，新页面会自动被追踪：
```bash
agent-browser click @e1  # 点击打开新窗口的链接
agent-browser tab list   # 查看所有 tabs，包括新打开的
agent-browser tab 1      # 切换到新 tab
```

**3. 并行数据采集**
```bash
agent-browser open https://shop.com/list
agent-browser tab new https://shop.com/item/1
agent-browser tab new https://shop.com/item/2
agent-browser tab 1 && agent-browser snapshot -i  # 采集第一个商品
agent-browser tab 2 && agent-browser snapshot -i  # 采集第二个商品
```

## Sessions

Note: All sessions share the same userDataDir (`~/tmp/agent-browser/`), so parallel sessions may cause file locking issues. Use one session at a time for best results.

```bash
agent-browser --session mytest open site.com  # Named session
agent-browser session list                     # List active sessions
```

## JSON output (for parsing)

Add `--json` for machine-readable output:
```bash
agent-browser snapshot -i --json
agent-browser get text @e1 --json
```

## Debugging

```bash
agent-browser open example.com --headed  # Show browser window
agent-browser console                    # View console messages
agent-browser errors                     # View page errors
```
