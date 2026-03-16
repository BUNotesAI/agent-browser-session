use crate::connection::Response;

pub fn print_response(resp: &Response, json_mode: bool) {
    if json_mode {
        println!("{}", serde_json::to_string(resp).unwrap_or_default());
        return;
    }

    if !resp.success {
        eprintln!(
            "\x1b[31m✗\x1b[0m {}",
            resp.error.as_deref().unwrap_or("Unknown error")
        );
        return;
    }

    if let Some(data) = &resp.data {
        // Navigation response
        if let Some(url) = data.get("url").and_then(|v| v.as_str()) {
            if let Some(title) = data.get("title").and_then(|v| v.as_str()) {
                println!("\x1b[32m✓\x1b[0m \x1b[1m{}\x1b[0m", title);
                println!("\x1b[2m  {}\x1b[0m", url);
                return;
            }
            println!("{}", url);
            return;
        }
        // Snapshot
        if let Some(snapshot) = data.get("snapshot").and_then(|v| v.as_str()) {
            println!("{}", snapshot);
            return;
        }
        // Title
        if let Some(title) = data.get("title").and_then(|v| v.as_str()) {
            println!("{}", title);
            return;
        }
        // Text
        if let Some(text) = data.get("text").and_then(|v| v.as_str()) {
            println!("{}", text);
            return;
        }
        // HTML
        if let Some(html) = data.get("html").and_then(|v| v.as_str()) {
            println!("{}", html);
            return;
        }
        // Value
        if let Some(value) = data.get("value").and_then(|v| v.as_str()) {
            println!("{}", value);
            return;
        }
        // Count
        if let Some(count) = data.get("count").and_then(|v| v.as_i64()) {
            println!("{}", count);
            return;
        }
        // Boolean results
        if let Some(visible) = data.get("visible").and_then(|v| v.as_bool()) {
            println!("{}", visible);
            return;
        }
        if let Some(enabled) = data.get("enabled").and_then(|v| v.as_bool()) {
            println!("{}", enabled);
            return;
        }
        if let Some(checked) = data.get("checked").and_then(|v| v.as_bool()) {
            println!("{}", checked);
            return;
        }
        // Eval result
        if let Some(result) = data.get("result") {
            println!(
                "{}",
                serde_json::to_string_pretty(result).unwrap_or_default()
            );
            return;
        }
        // Tabs
        if let Some(tabs) = data.get("tabs").and_then(|v| v.as_array()) {
            for (i, tab) in tabs.iter().enumerate() {
                let title = tab
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");
                let url = tab.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let active = tab.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                let marker = if active { "→" } else { " " };
                println!("{} [{}] {} - {}", marker, i, title, url);
            }
            return;
        }
        // Console logs
        if let Some(logs) = data.get("logs").and_then(|v| v.as_array()) {
            for log in logs {
                let level = log.get("type").and_then(|v| v.as_str()).unwrap_or("log");
                let text = log.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let color = match level {
                    "error" => "\x1b[31m",
                    "warning" => "\x1b[33m",
                    "info" => "\x1b[36m",
                    _ => "\x1b[0m",
                };
                println!("{}[{}]\x1b[0m {}", color, level, text);
            }
            return;
        }
        // Errors
        if let Some(errors) = data.get("errors").and_then(|v| v.as_array()) {
            for err in errors {
                let msg = err.get("message").and_then(|v| v.as_str()).unwrap_or("");
                println!("\x1b[31m✗\x1b[0m {}", msg);
            }
            return;
        }
        // Cookies
        if let Some(cookies) = data.get("cookies").and_then(|v| v.as_array()) {
            for cookie in cookies {
                let name = cookie.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let value = cookie.get("value").and_then(|v| v.as_str()).unwrap_or("");
                println!("{}={}", name, value);
            }
            return;
        }
        // Bounding box
        if let Some(box_data) = data.get("box") {
            println!(
                "{}",
                serde_json::to_string_pretty(box_data).unwrap_or_default()
            );
            return;
        }
        // Closed
        if data.get("closed").is_some() {
            println!("\x1b[32m✓\x1b[0m Browser closed");
            return;
        }
        // Screenshot path
        if let Some(path) = data.get("path").and_then(|v| v.as_str()) {
            println!("\x1b[32m✓\x1b[0m Screenshot saved to {}", path);
            return;
        }
        // Default success
        println!("\x1b[32m✓\x1b[0m Done");
    }
}

/// Print command-specific help. Returns true if help was printed, false if command unknown.
pub fn print_command_help(command: &str) -> bool {
    let help = match command {
        // === Navigation ===
        "open" | "goto" | "navigate" => r##"
agent-browser-session open - Navigate to a URL

Usage: agent-browser-session open <url>

Navigates the browser to the specified URL. If no protocol is provided,
https:// is automatically prepended.

Aliases: goto, navigate

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation
  --headers <json>     Set HTTP headers (scoped to this origin)
  --headed             Show browser window

Examples:
  agent-browser-session open example.com
  agent-browser-session open https://github.com
  agent-browser-session open localhost:3000
  agent-browser-session open api.example.com --headers '{"Authorization": "Bearer token"}'
    # ^ Headers only sent to api.example.com, not other domains
"##,
        "back" => r##"
agent-browser-session back - Navigate back in history

Usage: agent-browser-session back

Goes back one page in the browser history, equivalent to clicking
the browser's back button.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session back
"##,
        "forward" => r##"
agent-browser-session forward - Navigate forward in history

Usage: agent-browser-session forward

Goes forward one page in the browser history, equivalent to clicking
the browser's forward button.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session forward
"##,
        "reload" => r##"
agent-browser-session reload - Reload the current page

Usage: agent-browser-session reload

Reloads the current page, equivalent to pressing F5 or clicking
the browser's reload button.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session reload
"##,

        // === Core Actions ===
        "click" => r##"
agent-browser-session click - Click an element

Usage: agent-browser-session click <selector>

Clicks on the specified element. The selector can be a CSS selector,
XPath, or an element reference from snapshot (e.g., @e1).

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session click "#submit-button"
  agent-browser-session click @e1
  agent-browser-session click "button.primary"
  agent-browser-session click "//button[@type='submit']"
"##,
        "dblclick" => r##"
agent-browser-session dblclick - Double-click an element

Usage: agent-browser-session dblclick <selector>

Double-clicks on the specified element. Useful for text selection
or triggering double-click handlers.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session dblclick "#editable-text"
  agent-browser-session dblclick @e5
"##,
        "fill" => r##"
agent-browser-session fill - Clear and fill an input field

Usage: agent-browser-session fill <selector> <text>

Clears the input field and fills it with the specified text.
This replaces any existing content in the field.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session fill "#email" "user@example.com"
  agent-browser-session fill @e3 "Hello World"
  agent-browser-session fill "input[name='search']" "query"
"##,
        "type" => r##"
agent-browser-session type - Type text into an element

Usage: agent-browser-session type <selector> <text>

Types text into the specified element character by character.
Unlike fill, this does not clear existing content first.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session type "#search" "hello"
  agent-browser-session type @e2 "additional text"
"##,
        "hover" => r##"
agent-browser-session hover - Hover over an element

Usage: agent-browser-session hover <selector>

Moves the mouse to hover over the specified element. Useful for
triggering hover states or dropdown menus.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session hover "#dropdown-trigger"
  agent-browser-session hover @e4
"##,
        "focus" => r##"
agent-browser-session focus - Focus an element

Usage: agent-browser-session focus <selector>

Sets keyboard focus to the specified element.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session focus "#input-field"
  agent-browser-session focus @e2
"##,
        "check" => r##"
agent-browser-session check - Check a checkbox

Usage: agent-browser-session check <selector>

Checks a checkbox element. If already checked, no action is taken.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session check "#terms-checkbox"
  agent-browser-session check @e7
"##,
        "uncheck" => r##"
agent-browser-session uncheck - Uncheck a checkbox

Usage: agent-browser-session uncheck <selector>

Unchecks a checkbox element. If already unchecked, no action is taken.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session uncheck "#newsletter-opt-in"
  agent-browser-session uncheck @e8
"##,
        "select" => r##"
agent-browser-session select - Select a dropdown option

Usage: agent-browser-session select <selector> <value>

Selects an option in a <select> dropdown by its value attribute.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session select "#country" "US"
  agent-browser-session select @e5 "option2"
"##,
        "drag" => r##"
agent-browser-session drag - Drag and drop

Usage: agent-browser-session drag <source> <target>

Drags an element from source to target location.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session drag "#draggable" "#drop-zone"
  agent-browser-session drag @e1 @e2
"##,
        "upload" => r##"
agent-browser-session upload - Upload files

Usage: agent-browser-session upload <selector> <files...>

Uploads one or more files to a file input element.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session upload "#file-input" ./document.pdf
  agent-browser-session upload @e3 ./image1.png ./image2.png
"##,

        // === Keyboard ===
        "press" | "key" => r##"
agent-browser-session press - Press a key or key combination

Usage: agent-browser-session press <key>

Presses a key or key combination. Supports special keys and modifiers.

Aliases: key

Special Keys:
  Enter, Tab, Escape, Backspace, Delete, Space
  ArrowUp, ArrowDown, ArrowLeft, ArrowRight
  Home, End, PageUp, PageDown
  F1-F12

Modifiers (combine with +):
  Control, Alt, Shift, Meta

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session press Enter
  agent-browser-session press Tab
  agent-browser-session press Control+a
  agent-browser-session press Control+Shift+s
  agent-browser-session press Escape
"##,
        "keydown" => r##"
agent-browser-session keydown - Press a key down (without release)

Usage: agent-browser-session keydown <key>

Presses a key down without releasing it. Use keyup to release.
Useful for holding modifier keys.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session keydown Shift
  agent-browser-session keydown Control
"##,
        "keyup" => r##"
agent-browser-session keyup - Release a key

Usage: agent-browser-session keyup <key>

Releases a key that was pressed with keydown.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session keyup Shift
  agent-browser-session keyup Control
"##,

        // === Scroll ===
        "scroll" => r##"
agent-browser-session scroll - Scroll the page

Usage: agent-browser-session scroll [direction] [amount]

Scrolls the page in the specified direction.

Arguments:
  direction            up, down, left, right (default: down)
  amount               Pixels to scroll (default: 300)

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session scroll
  agent-browser-session scroll down 500
  agent-browser-session scroll up 200
  agent-browser-session scroll left 100
"##,
        "scrollintoview" | "scrollinto" => r##"
agent-browser-session scrollintoview - Scroll element into view

Usage: agent-browser-session scrollintoview <selector>

Scrolls the page until the specified element is visible in the viewport.

Aliases: scrollinto

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session scrollintoview "#footer"
  agent-browser-session scrollintoview @e15
"##,

        // === Wait ===
        "wait" => r##"
agent-browser-session wait - Wait for condition

Usage: agent-browser-session wait <selector|ms|option>

Waits for an element to appear, a timeout, or other conditions.

Modes:
  <selector>           Wait for element to appear
  <ms>                 Wait for specified milliseconds
  --url <pattern>      Wait for URL to match pattern
  --load <state>       Wait for load state (load, domcontentloaded, networkidle)
  --fn <expression>    Wait for JavaScript expression to be truthy
  --text <text>        Wait for text to appear on page

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session wait "#loading-spinner"
  agent-browser-session wait 2000
  agent-browser-session wait --url "**/dashboard"
  agent-browser-session wait --load networkidle
  agent-browser-session wait --fn "window.appReady === true"
  agent-browser-session wait --text "Welcome back"
"##,

        // === Screenshot/PDF ===
        "screenshot" => r##"
agent-browser-session screenshot - Take a screenshot

Usage: agent-browser-session screenshot [path]

Captures a screenshot of the current page. If no path is provided,
outputs base64-encoded image data.

Options:
  --full, -f           Capture full page (not just viewport)

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session screenshot
  agent-browser-session screenshot ./screenshot.png
  agent-browser-session screenshot --full ./full-page.png
"##,
        "pdf" => r##"
agent-browser-session pdf - Save page as PDF

Usage: agent-browser-session pdf <path>

Saves the current page as a PDF file.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session pdf ./page.pdf
  agent-browser-session pdf ~/Documents/report.pdf
"##,

        // === Snapshot ===
        "snapshot" => r##"
agent-browser-session snapshot - Get accessibility tree snapshot

Usage: agent-browser-session snapshot [options]

Returns an accessibility tree representation of the page with element
references (like @e1, @e2) that can be used in subsequent commands.
Designed for AI agents to understand page structure.

Options:
  -i, --interactive    Only include interactive elements
  -c, --compact        Remove empty structural elements
  -d, --depth <n>      Limit tree depth
  -s, --selector <sel> Scope snapshot to CSS selector

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session snapshot
  agent-browser-session snapshot -i
  agent-browser-session snapshot --compact --depth 5
  agent-browser-session snapshot -s "#main-content"
"##,

        // === Eval ===
        "eval" => r##"
agent-browser-session eval - Execute JavaScript

Usage: agent-browser-session eval <script>

Executes JavaScript code in the browser context and returns the result.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session eval "document.title"
  agent-browser-session eval "window.location.href"
  agent-browser-session eval "document.querySelectorAll('a').length"
"##,

        // === Close ===
        "close" | "quit" | "exit" => r##"
agent-browser-session close - Close the browser

Usage: agent-browser-session close

Closes the browser instance for the current session.

Aliases: quit, exit

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session close
"##,

        // === Get ===
        "get" => r##"
agent-browser-session get - Retrieve information from elements or page

Usage: agent-browser-session get <subcommand> [args]

Retrieves various types of information from elements or the page.

Subcommands:
  text <selector>            Get text content of element
  html <selector>            Get inner HTML of element
  value <selector>           Get value of input element
  attr <selector> <name>     Get attribute value
  title                      Get page title
  url                        Get current URL
  count <selector>           Count matching elements
  box <selector>             Get bounding box (x, y, width, height)

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session get text @e1
  agent-browser-session get html "#content"
  agent-browser-session get value "#email-input"
  agent-browser-session get attr "#link" href
  agent-browser-session get title
  agent-browser-session get url
  agent-browser-session get count "li.item"
  agent-browser-session get box "#header"
"##,

        // === Is ===
        "is" => r##"
agent-browser-session is - Check element state

Usage: agent-browser-session is <subcommand> <selector>

Checks the state of an element and returns true/false.

Subcommands:
  visible <selector>   Check if element is visible
  enabled <selector>   Check if element is enabled (not disabled)
  checked <selector>   Check if checkbox/radio is checked

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session is visible "#modal"
  agent-browser-session is enabled "#submit-btn"
  agent-browser-session is checked "#agree-checkbox"
"##,

        // === Find ===
        "find" => r##"
agent-browser-session find - Find and interact with elements by locator

Usage: agent-browser-session find <locator> <value> [action] [text]

Finds elements using semantic locators and optionally performs an action.

Locators:
  role <role>              Find by ARIA role (--name <n>, --exact)
  text <text>              Find by text content (--exact)
  label <label>            Find by associated label (--exact)
  placeholder <text>       Find by placeholder text (--exact)
  alt <text>               Find by alt text (--exact)
  title <text>             Find by title attribute (--exact)
  testid <id>              Find by data-testid attribute
  first <selector>         First matching element
  last <selector>          Last matching element
  nth <index> <selector>   Nth matching element (0-based)

Actions (default: click):
  click, fill, type, hover, focus, check, uncheck

Options:
  --name <name>        Filter role by accessible name
  --exact              Require exact text match

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session find role button click --name Submit
  agent-browser-session find text "Sign In" click
  agent-browser-session find label "Email" fill "user@example.com"
  agent-browser-session find placeholder "Search..." type "query"
  agent-browser-session find testid "login-form" click
  agent-browser-session find first "li.item" click
  agent-browser-session find nth 2 ".card" hover
"##,

        // === Mouse ===
        "mouse" => r##"
agent-browser-session mouse - Low-level mouse operations

Usage: agent-browser-session mouse <subcommand> [args]

Performs low-level mouse operations for precise control.

Subcommands:
  move <x> <y>         Move mouse to coordinates
  down [button]        Press mouse button (left, right, middle)
  up [button]          Release mouse button
  wheel <dy> [dx]      Scroll mouse wheel

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session mouse move 100 200
  agent-browser-session mouse down
  agent-browser-session mouse up
  agent-browser-session mouse down right
  agent-browser-session mouse wheel 100
  agent-browser-session mouse wheel -50 0
"##,

        // === Set ===
        "set" => r##"
agent-browser-session set - Configure browser settings

Usage: agent-browser-session set <setting> [args]

Configures various browser settings and emulation options.

Settings:
  viewport <w> <h>           Set viewport size
  device <name>              Emulate device (e.g., "iPhone 12")
  geo <lat> <lng>            Set geolocation
  offline [on|off]           Toggle offline mode
  headers <json>             Set extra HTTP headers
  credentials <user> <pass>  Set HTTP authentication
  media [dark|light]         Set color scheme preference
        [reduced-motion]     Enable reduced motion

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session set viewport 1920 1080
  agent-browser-session set device "iPhone 12"
  agent-browser-session set geo 37.7749 -122.4194
  agent-browser-session set offline on
  agent-browser-session set headers '{"X-Custom": "value"}'
  agent-browser-session set credentials admin secret123
  agent-browser-session set media dark
  agent-browser-session set media light reduced-motion
"##,

        // === Network ===
        "network" => r##"
agent-browser-session network - Network interception and monitoring

Usage: agent-browser-session network <subcommand> [args]

Intercept, mock, or monitor network requests.

Subcommands:
  route <url> [options]      Intercept requests matching URL pattern
    --abort                  Abort matching requests
    --body <json>            Respond with custom body
  unroute [url]              Remove route (all if no URL)
  requests [options]         List captured requests
    --clear                  Clear request log
    --filter <pattern>       Filter by URL pattern

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session network route "**/api/*" --abort
  agent-browser-session network route "**/data.json" --body '{"mock": true}'
  agent-browser-session network unroute
  agent-browser-session network requests
  agent-browser-session network requests --filter "api"
  agent-browser-session network requests --clear
"##,

        // === Storage ===
        "storage" => r##"
agent-browser-session storage - Manage web storage

Usage: agent-browser-session storage <type> [operation] [key] [value]

Manage localStorage and sessionStorage.

Types:
  local                localStorage
  session              sessionStorage

Operations:
  get [key]            Get all storage or specific key
  set <key> <value>    Set a key-value pair
  clear                Clear all storage

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session storage local
  agent-browser-session storage local get authToken
  agent-browser-session storage local set theme "dark"
  agent-browser-session storage local clear
  agent-browser-session storage session get userId
"##,

        // === Cookies ===
        "cookies" => r##"
agent-browser-session cookies - Manage browser cookies

Usage: agent-browser-session cookies [operation] [args]

Manage browser cookies for the current context.

Operations:
  get                  Get all cookies (default)
  set <name> <value>   Set a cookie
  clear                Clear all cookies

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session cookies
  agent-browser-session cookies get
  agent-browser-session cookies set session_id "abc123"
  agent-browser-session cookies clear
"##,

        // === Tabs ===
        "tab" => r##"
agent-browser-session tab - Manage browser tabs

Usage: agent-browser-session tab [operation] [args]

Manage browser tabs in the current window.

Operations:
  list                 List all tabs (default)
  new [url]            Open new tab
  close [index]        Close tab (current if no index)
  <index>              Switch to tab by index

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session tab
  agent-browser-session tab list
  agent-browser-session tab new
  agent-browser-session tab new https://example.com
  agent-browser-session tab 2
  agent-browser-session tab close
  agent-browser-session tab close 1
"##,

        // === Window ===
        "window" => r##"
agent-browser-session window - Manage browser windows

Usage: agent-browser-session window <operation>

Note: window operations are not supported in persistent context mode.
Use 'tab new' to open additional pages in the same context.

Operations:
  new                  Open new browser window (not supported)

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation
"##,

        // === Frame ===
        "frame" => r##"
agent-browser-session frame - Switch frame context

Usage: agent-browser-session frame <selector|main>

Switch to an iframe or back to the main frame.

Arguments:
  <selector>           CSS selector for iframe
  main                 Switch back to main frame

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session frame "#embed-iframe"
  agent-browser-session frame "iframe[name='content']"
  agent-browser-session frame main
"##,

        // === Dialog ===
        "dialog" => r##"
agent-browser-session dialog - Handle browser dialogs

Usage: agent-browser-session dialog <response> [text]

Respond to browser dialogs (alert, confirm, prompt).

Operations:
  accept [text]        Accept dialog, optionally with prompt text
  dismiss              Dismiss/cancel dialog

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session dialog accept
  agent-browser-session dialog accept "my input"
  agent-browser-session dialog dismiss
"##,

        // === Trace ===
        "trace" => r##"
agent-browser-session trace - Record execution trace

Usage: agent-browser-session trace <operation> [path]

Record a trace for debugging with Trace Viewer.

Operations:
  start [path]         Start recording trace
  stop [path]          Stop recording and save trace

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session trace start
  agent-browser-session trace start ./my-trace
  agent-browser-session trace stop
  agent-browser-session trace stop ./debug-trace.zip
"##,

        // === Console/Errors ===
        "console" => r##"
agent-browser-session console - View console logs

Usage: agent-browser-session console [--clear]

View browser console output (log, warn, error, info).

Options:
  --clear              Clear console log buffer

Note:
  Console logs are unavailable when using Patchright.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session console
  agent-browser-session console --clear
"##,
        "errors" => r##"
agent-browser-session errors - View page errors

Usage: agent-browser-session errors [--clear]

View JavaScript errors and uncaught exceptions.

Options:
  --clear              Clear error buffer

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session errors
  agent-browser-session errors --clear
"##,

        // === Highlight ===
        "highlight" => r##"
agent-browser-session highlight - Highlight an element

Usage: agent-browser-session highlight <selector>

Visually highlights an element on the page for debugging.

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session highlight "#target-element"
  agent-browser-session highlight @e5
"##,

        // === State ===
        "state" => r##"
agent-browser-session state - Save/load browser state

Usage: agent-browser-session state <operation> <path>

Save or restore browser state (cookies, localStorage, sessionStorage).

Operations:
  save <path>          Save current state to file
  load <path>          Load state from file

Global Options:
  --json               Output as JSON
  --tabname <name>     Named tab for isolation

Examples:
  agent-browser-session state save ./auth-state.json
  agent-browser-session state load ./auth-state.json
"##,

        // === Session ===
        "session" => r##"
agent-browser-session session - Show session info

Usage: agent-browser-session session [operation]

Show current browser session status. Session is always "main".
Use --tabname for client isolation within the session.

Operations:
  (none)               Show current session name
  list                 List all active sessions

Global Options:
  --json               Output as JSON

Examples:
  agent-browser-session session
  agent-browser-session session list
"##,

        // === Kill ===
        "kill" => r##"
agent-browser-session kill - Terminate all daemon processes

Usage: agent-browser-session kill

⚠️  FOR MANUAL USE ONLY — do NOT call from agents or scripts.

Kills all running daemon processes, closes browser windows, and cleans up
socket/pid files. Since the browser is shared across tabnames, killing it
will interrupt ALL connected agents.

Use this when you need to:
  - Restart the daemon after a code update
  - Free up resources
  - Recover from a stuck daemon

Examples:
  agent-browser-session kill
"##,

        // === Install ===
        "install" => r##"
agent-browser-session install - Install browser binaries

Usage: agent-browser-session install [--with-deps]

Downloads and installs browser binaries required for automation.

Options:
  -d, --with-deps      Also install system dependencies (Linux only)

Examples:
  agent-browser-session install
  agent-browser-session install --with-deps
"##,

        _ => return false,
    };
    println!("{}", help.trim());
    true
}

pub fn print_help() {
    println!(
        r#"
agent-browser-session - fast browser automation CLI for AI agents

Usage: agent-browser-session <command> [args] [options]

Core Commands:
  open <url>                 Navigate to URL
  click <sel>                Click element (or @ref)
  dblclick <sel>             Double-click element
  type <sel> <text>          Type into element
  fill <sel> <text>          Clear and fill
  press <key>                Press key (Enter, Tab, Control+a)
  hover <sel>                Hover element
  focus <sel>                Focus element
  check <sel>                Check checkbox
  uncheck <sel>              Uncheck checkbox
  select <sel> <val>         Select dropdown option
  drag <src> <dst>           Drag and drop
  upload <sel> <files...>    Upload files
  scroll <dir> [px]          Scroll (up/down/left/right)
  scrollintoview <sel>       Scroll element into view
  wait <sel|ms>              Wait for element or time
  screenshot [path]          Take screenshot
  pdf <path>                 Save as PDF
  snapshot                   Accessibility tree with refs (for AI)
  eval <js>                  Run JavaScript

Navigation:
  back                       Go back
  forward                    Go forward
  reload                     Reload page

Get Info:  agent-browser-session get <what> [selector]
  text, html, value, attr <name>, title, url, count, box

Check State:  agent-browser-session is <what> <selector>
  visible, enabled, checked

Find Elements:  agent-browser-session find <locator> <value> <action> [text]
  role, text, label, placeholder, alt, title, testid, first, last, nth

Mouse:  agent-browser-session mouse <action> [args]
  move <x> <y>, down [btn], up [btn], wheel <dy> [dx]

Browser Settings:  agent-browser-session set <setting> [value]
  viewport <w> <h>, device <name>, geo <lat> <lng>
  offline [on|off], headers <json>, credentials <user> <pass>
  media [dark|light] [reduced-motion]

Network:  agent-browser-session network <action>
  route <url> [--abort|--body <json>]
  unroute [url]
  requests [--clear] [--filter <pattern>]

Storage:
  cookies [get|set|clear]    Manage cookies
  storage <local|session>    Manage web storage

Tabs:
  tab [new|list|close|<n>]   Manage tabs

Debug:
  trace start|stop [path]    Record trace
  console [--clear]          View console logs
  errors [--clear]           View page errors
  highlight <sel>            Highlight element

Sessions:
  session                    Show current session name
  session list               List active sessions

Setup:
  install                    Install browser binaries
  install --with-deps        Also install system dependencies (Linux)
  kill                       Kill all daemons + browser (manual use only)

Snapshot Options:
  -i, --interactive          Only interactive elements
  -c, --compact              Remove empty structural elements
  -d, --depth <n>            Limit tree depth
  -s, --selector <sel>       Scope to CSS selector

Options:
  --headers <json>           HTTP headers scoped to URL's origin (for auth)
  --executable-path <path>   Custom browser executable (or AGENT_BROWSER_EXECUTABLE_PATH)
  --extension <path>         Load browser extensions (repeatable).
  --channel <name>           Browser channel (chrome, msedge, chromium)
  --tabname <name>           Named tab for client isolation (multiple agents per session)
  --bundled                  Use bundled Chrome for Testing instead of system Chrome
  --json                     JSON output
  --full, -f                 Full page screenshot
  --headed [true|false]      Show browser window (default: true, use --headed false for headless)
  --cdp <port>               Connect via CDP (Chrome DevTools Protocol)
  --debug                    Debug output
  --version, -V              Show version

Note: By default, agent-browser-session uses your system Chrome for better compatibility
with existing browser profiles. Use --bundled to use the bundled Chrome for Testing.

Environment:
  AGENT_BROWSER_HEADED           Show browser window (default: true, set "false" for headless)
  AGENT_BROWSER_EXECUTABLE_PATH  Custom browser executable path
  AGENT_BROWSER_TABNAME          Named tab (or --tabname)
  AGENT_BROWSER_STREAM_PORT      Enable WebSocket streaming on port (e.g., 9223)

Examples:
  agent-browser-session open example.com
  agent-browser-session snapshot -i              # Interactive elements only
  agent-browser-session click @e2                # Click by ref from snapshot
  agent-browser-session fill @e3 "test@example.com"
  agent-browser-session find role button click --name Submit
  agent-browser-session get text @e1
  agent-browser-session screenshot --full
  agent-browser-session --cdp 9222 snapshot      # Connect via CDP port
"#
    );
}
