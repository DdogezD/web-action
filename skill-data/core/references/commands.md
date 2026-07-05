# Command Reference

Complete reference for all web-action commands. For quick start and common patterns, see SKILL.md.

## Navigation

```bash
web-action open            # Launch browser (no navigation); stays on about:blank.
                              # Pair with `network route`, `cookies set --curl`, or
                              # `addinitscript` to stage state before the first navigation.
web-action open <url>      # Launch + navigate (aliases: goto, navigate)
                              # Supports: https://, http://, file://, about:, data://
                              # Auto-prepends https:// if no protocol given
web-action read [url]      # Fetch agent-readable text, or read rendered active-tab DOM
                              # Explicit URLs send Accept: text/markdown, then try .md if needed
                              # Walks ancestor paths for llms.txt before HTML fallback
                              # --llms and --require-md without URL use the active tab URL
                              # --filter narrows page content to matching heading sections
                              # Honors --allowed-domains, --content-boundaries, and --max-output
                              # Options: --raw, --require-md, --outline, --llms <index|full>, --filter, --timeout <ms>
web-action back            # Go back
web-action forward         # Go forward
web-action reload          # Reload page
web-action pushstate <url> # SPA client-side navigation. Auto-detects
                              # window.next.router.push (triggers RSC fetch on Next.js);
                              # falls back to history.pushState + popstate/navigate events.
web-action close           # Close browser (aliases: quit, exit)
web-action connect 9222    # Connect to browser via CDP port
```

### Pre-navigation setup (one-turn batch)

```bash
web-action batch \
  '["open"]' \
  '["network","route","*","--abort","--resource-type","script"]' \
  '["cookies","set","--curl","cookies.curl","--domain","localhost"]' \
  '["navigate","http://localhost:3000/target"]'
```

`open` with no URL gives you a clean launch so any interception, cookies, or init scripts you register take effect on the *first* real navigation. Use for SSR-only debug (`--resource-type script`), protected-origin auth, or capturing fresh `react suspense`/`vitals` state without noise from a prior page.

## Snapshot (page analysis)

```bash
web-action snapshot            # Full accessibility tree
web-action snapshot -i         # Interactive elements only (recommended)
web-action snapshot -c         # Compact output
web-action snapshot -d 3       # Limit depth to 3
web-action snapshot -s "#main" # Scope to CSS selector
```

## Interactions (use @refs from snapshot)

```bash
web-action click @e1           # Click
web-action click @e1 --new-tab # Click and open in new tab
web-action dblclick @e1        # Double-click
web-action focus @e1           # Focus element
web-action fill @e2 "text"     # Clear and type
web-action type @e2 "text"     # Type without clearing
web-action press Enter         # Press key (alias: key)
web-action press Control+a     # Key combination
web-action keydown Shift       # Hold key down
web-action keyup Shift         # Release key
web-action hover @e1           # Hover
web-action check @e1           # Check checkbox
web-action uncheck @e1         # Uncheck checkbox
web-action select @e1 "value"  # Select dropdown option
web-action select @e1 "a" "b"  # Select multiple options
web-action scroll down 500     # Scroll page (default: down 300px)
web-action scrollintoview @e1  # Scroll element into view (alias: scrollinto)
web-action drag @e1 @e2        # Drag and drop
web-action upload @e1 file.pdf # Upload files
```

Clicks fail before dispatch when another element covers the target's click point. The error names the covering element, for example `covered by <div#consent-banner>`. Dismiss or interact with that element, run a fresh snapshot, then retry the original action.

## Get Information

```bash
web-action get text @e1        # Get element text
web-action get html @e1        # Get innerHTML
web-action get value @e1       # Get input value
web-action get attr @e1 href   # Get attribute
web-action get title           # Get page title
web-action get url             # Get current URL
web-action get cdp-url         # Get CDP WebSocket URL
web-action get count ".item"   # Count matching elements
web-action get box @e1         # Get bounding box
web-action get styles @e1      # Get computed styles (font, color, bg, etc.)
```

## Check State

```bash
web-action is visible @e1      # Check if visible
web-action is enabled @e1      # Check if enabled
web-action is checked @e1      # Check if checked
```

## Screenshots and PDF

```bash
web-action screenshot          # Save to temporary directory
web-action screenshot path.png # Save to specific path
web-action screenshot --full   # Full page
web-action pdf output.pdf      # Save as PDF
```

Headless Chromium screenshots hide native scrollbars for consistent image output. Pass `--hide-scrollbars false` when launching to keep native scrollbars visible.

## Video Recording

```bash
web-action open https://example.com     # Launch a browser session first
web-action record start ./demo.webm    # Start recording
web-action click @e1                   # Perform actions
web-action record stop                 # Stop and save video
web-action record restart ./take2.webm # Stop current + start new
```

## Wait

```bash
web-action wait @e1                     # Wait for element
web-action wait 2000                    # Wait milliseconds
web-action wait --text "Success"        # Wait for text (or -t)
web-action wait --url "**/dashboard"    # Wait for URL pattern (or -u)
web-action wait --load networkidle      # Wait for network idle (or -l)
web-action wait --fn "window.ready"     # Wait for JS condition (or -f)
```

## Mouse Control

```bash
web-action mouse move 100 200      # Move mouse
web-action mouse down left         # Press button
web-action mouse up left           # Release button
web-action mouse wheel 100         # Scroll wheel
```

## Semantic Locators (alternative to refs)

```bash
web-action find role button click --name "Submit"
web-action find text "Sign In" click
web-action find text "Sign In" click --exact      # Exact match only
web-action find label "Email" fill "user@test.com"
web-action find placeholder "Search" type "query"
web-action find alt "Logo" click
web-action find title "Close" click
web-action find testid "submit-btn" click
web-action find first ".item" click
web-action find last ".item" click
web-action find nth 2 "a" hover
```

## Browser Settings

```bash
web-action set viewport 1920 1080          # Set viewport size
web-action set viewport 1920 1080 2        # 2x retina (same CSS size, higher res screenshots)
web-action set device "iPhone 14"          # Emulate device
web-action set geo 37.7749 -122.4194       # Set geolocation (alias: geolocation)
web-action set offline on                  # Toggle offline mode
web-action set headers '{"X-Key":"v"}'     # Extra HTTP headers
web-action set credentials user pass       # HTTP basic auth (alias: auth)
web-action set media dark                  # Emulate color scheme
web-action set media light reduced-motion  # Light mode + reduced motion
```

## Cookies and Storage

```bash
web-action cookies                     # Get all cookies
web-action cookies set name value      # Set cookie
web-action cookies clear               # Clear cookies
web-action storage local               # Get all localStorage
web-action storage local key           # Get specific key
web-action storage local set k v       # Set value
web-action storage local clear         # Clear all
```

## Network

```bash
web-action network route <url>              # Intercept requests
web-action network route <url> --abort      # Block requests
web-action network route <url> --body '{}'  # Mock response
web-action network unroute [url]            # Remove routes
web-action network requests                 # View tracked requests
web-action network requests --filter api    # Filter requests
```

## Tabs and Windows

```bash
web-action tab                              # List tabs with tabId and label
web-action tab new [url]                    # New tab
web-action tab new --label docs [url]       # New tab with a memorable label
web-action tab t2                           # Switch to tab by id
web-action tab docs                         # Switch to tab by label
web-action tab close                        # Close current tab
web-action tab close t2                     # Close tab by id
web-action tab close docs                   # Close tab by label
web-action window new                       # New window
```

Tab ids are stable strings of the form `t1`, `t2`, `t3`. They're never reused within a session, so the same id keeps referring to the same tab across commands. Positional integers are **not** accepted — `tab 2` errors with a teaching message; use `t2`.

User-assigned labels (`docs`, `app`, `admin`) are interchangeable with ids everywhere a tab ref is accepted. Labels are the agent-friendly way to write multi-tab workflows:

```bash
web-action tab new --label docs https://docs.example.com
web-action tab new --label app  https://app.example.com
web-action tab docs                   # switch to docs
web-action snapshot                   # populate refs for docs
web-action click @e1                  # ref click on docs
web-action tab app                    # switch to app
web-action tab close docs             # close by label
```

Labels are never auto-generated, never rewritten on navigation, and must be unique within a session. To interact with another tab, switch to it first: the daemon maintains a single active tab, so refs (`@eN`) belong to the tab that was active when the snapshot ran.

## Frames

```bash
web-action frame "#iframe"     # Switch to iframe by CSS selector
web-action frame @e3           # Switch to iframe by element ref
web-action frame main          # Back to main frame
```

### Iframe support

Iframes are detected automatically during snapshots. When the main-frame snapshot runs, `Iframe` nodes are resolved and their content is inlined beneath the iframe element in the output (one level of nesting; iframes within iframes are not expanded).

```bash
web-action snapshot -i
# @e3 [Iframe] "payment-frame"
#   @e4 [input] "Card number"
#   @e5 [button] "Pay"

# Interact directly — refs inside iframes already work
web-action fill @e4 "4111111111111111"
web-action click @e5

# Or switch frame context for scoped snapshots
web-action frame @e3               # Switch using element ref
web-action snapshot -i             # Snapshot scoped to that iframe
web-action frame main              # Return to main frame
```

The `frame` command accepts:
- **Element refs** — `frame @e3` resolves the ref to an iframe element
- **CSS selectors** — `frame "#payment-iframe"` finds the iframe by selector
- **Frame name/URL** — matches against the browser's frame tree

## Dialogs

By default, `alert` and `beforeunload` dialogs are automatically accepted so they never block the agent. `confirm` and `prompt` dialogs still require explicit handling. Use `--no-auto-dialog` to disable this behavior.

```bash
web-action dialog accept [text]  # Accept dialog
web-action dialog dismiss        # Dismiss dialog
web-action dialog status         # Check if a dialog is currently open
```

## JavaScript

```bash
web-action eval "document.title"          # Simple expressions only
web-action eval -b "<base64>"             # Any JavaScript (base64 encoded)
web-action eval --stdin                   # Read script from stdin
```

Use `-b`/`--base64` or `--stdin` for reliable execution. Shell escaping with nested quotes and special characters is error-prone.

```bash
# Base64 encode your script, then:
web-action eval -b "ZG9jdW1lbnQucXVlcnlTZWxlY3RvcignW3NyYyo9Il9uZXh0Il0nKQ=="

# Or use stdin with heredoc for multiline scripts:
cat <<'EOF' | web-action eval --stdin
const links = document.querySelectorAll('a');
Array.from(links).map(a => a.href);
EOF
```

## Authentication and Plugins

```bash
web-action auth save <name> --url <url> --username <user> --password-stdin
web-action auth login <name>          # Login using saved credentials
web-action auth login <name> --credential-provider <plugin> [--item <ref>] [--url <url>]
web-action auth login <name> --username-selector <s> --password-selector <s> [--submit-selector <s>]
web-action auth list                  # List saved auth profiles
web-action auth show <name>           # Show profile metadata, no passwords
web-action auth delete <name>         # Delete a saved profile
web-action plugin add <ref>           # Add a plugin from npm or GitHub
web-action plugin list                # List configured plugins
web-action plugin show <name>         # Show one configured plugin
web-action plugin run <name> <type> --payload <json>
                                          # Run an arbitrary plugin request
```

Credential provider plugins run out-of-process over the `web-action.plugin.v1` stdio JSON protocol and must declare `credential.read`. Use `--confirm-actions plugin:<name>:credential.read` to require explicit approval before a plugin resolves secrets.

Other capabilities use the same protocol:
- `browser.provider`: `web-action --provider <name> open <url>`
- `launch.mutate`: append local launch args, extensions, or init scripts
- `command.run`: `web-action plugin run <name> <type> --payload <json>`

`plugin run` is for `command.run` and custom capabilities. Core capabilities and protocol request types use their dedicated command paths.

## State Management

```bash
web-action state save auth.json    # Save cookies, storage, auth state
web-action state load auth.json    # Restore saved state
```

## MCP Server

```bash
web-action mcp
web-action mcp --tools all
web-action mcp --tools core,network,react
```

Starts a stdio Model Context Protocol server. MCP clients should configure the server command as `web-action` with args `["mcp"]`. The server defaults to MCP protocol 2025-11-25 and accepts older supported client protocol versions during initialization.

The default tools profile is `core`, which keeps MCP context small for everyday browser automation. Use `--tools all` for the full typed CLI parity surface, or combine profiles with commas, such as `--tools core,network,react`.

Profiles:

- `core` - Default. Navigation, snapshots, interaction, waits, reads, screenshots, JavaScript eval, close, tab basics, and profile discovery
- `network` - Network routes, request inspection, HAR, headers, credentials, offline
- `state` - Cookies, storage, auth, saved state, sessions, profiles, skills
- `debug` - Console/errors, tracing, profiling, recording, clipboard, plugins, doctor, dashboard, install, upgrade, chat, diff, batch, confirm/deny
- `tabs` - Back/forward/reload, tabs, windows, frames, dialogs
- `react` - React tree/inspect/renders/suspense, vitals, pushstate
- `mobile` - Viewport/device/geolocation/media, touch, swipe, mouse, keyboard
- `all` - Every MCP tool, including the full typed CLI parity surface

Common tools include:

- `agent_browser_tools_profiles`
- `agent_browser_open`
- `agent_browser_snapshot`
- `agent_browser_click`
- `agent_browser_fill`
- `agent_browser_type`
- `agent_browser_press`
- `agent_browser_wait_for_selector`
- `agent_browser_screenshot`
- `agent_browser_get_url`
- `agent_browser_eval`
- `agent_browser_close`

Tool calls use the same config files and environment variables as the CLI. Each tool accepts typed arguments plus `extraArgs` for advanced CLI flags and exact CLI parity. Tool discovery is paginated and includes read-only/open-world annotations so modern MCP clients can load the large typed surface incrementally. Use the `session` tool argument or `AGENT_BROWSER_SESSION` to isolate browser state.

## Global Options

```bash
web-action --session <name> ...    # Isolated browser session
web-action --json ...              # JSON output for parsing
web-action --headed ...            # Show browser window (not headless)
web-action --cdp <port> ...        # Connect via Chrome DevTools Protocol
web-action -p <provider> ...       # Browser provider or configured provider plugin
web-action --proxy <url> ...       # Use proxy server
web-action --proxy-bypass <hosts>  # Hosts to bypass proxy
web-action --headers <json> ...    # HTTP headers scoped to URL's origin
web-action --executable-path <p>   # Custom browser executable
web-action --extension <path> ...  # Load browser extension (repeatable)
web-action --ignore-https-errors   # Ignore SSL certificate errors
web-action --hide-scrollbars false # Keep native scrollbars visible in headless Chromium screenshots
web-action --help                  # Show help (-h)
web-action --version               # Show version (-V)
web-action <command> --help        # Show detailed help for a command
```

## Debugging

```bash
web-action --headed open example.com   # Show browser window
web-action --cdp 9222 snapshot         # Connect via CDP port
web-action connect 9222                # Alternative: connect command
web-action console                     # View console messages
web-action console --clear             # Clear console
web-action errors                      # View page errors
web-action errors --clear              # Clear errors
web-action highlight @e1               # Highlight element
web-action inspect                     # Open Chrome DevTools for this session
web-action trace start                 # Start recording trace
web-action trace stop trace.json       # Stop and save trace
web-action profiler start              # Start Chrome DevTools profiling
web-action profiler stop trace.json    # Stop and save profile
```

## React / Web Vitals

Requires `--enable react-devtools` at launch for the `react ...` commands. `vitals` and `pushstate` are framework-agnostic.

```bash
web-action open --enable react-devtools <url>    # Launch with React hook installed
web-action react tree                            # Full component tree
web-action react inspect <fiberId>               # Props, hooks, state, source
web-action react renders start                   # Begin re-render recording
web-action react renders stop [--json]           # Stop and print render profile
web-action react suspense [--only-dynamic] [--json]  # Suspense boundaries + classifier
                                                         # --only-dynamic hides the "static" list
web-action vitals [url] [--json]                 # LCP/CLS/TTFB/FCP/INP + hydration
web-action pushstate <url>                       # SPA client-side nav (auto-detects Next router)
```

`vitals` prints a summary by default and uses the same fields as the structured `--json` response.

## Init scripts

```bash
web-action open --init-script <path>             # Register before first navigation (repeatable)
web-action addinitscript <js>                    # Register at runtime (returns identifier)
web-action removeinitscript <identifier>         # Remove a previously registered init script
```

## cURL cookie import

```bash
web-action cookies set --curl <file>                             # Auto-detects JSON/cURL/Cookie-header
web-action cookies set --curl <file> --domain example.com        # Scope to a domain
```

Supported formats: JSON array of `{name, value}`, a cURL dump from DevTools -> Network -> Copy as cURL, or a bare Cookie header. Errors never echo cookie values.

## Network route by resource type

```bash
web-action network route '*' --abort --resource-type script       # Block scripts only (SSR-lock pattern)
web-action network route '*' --resource-type image,font --body '' # Stub images and fonts
```

## Environment Variables

```bash
AGENT_BROWSER_SESSION="mysession"            # Default session name
AGENT_BROWSER_EXECUTABLE_PATH="/path/chrome" # Custom browser path
AGENT_BROWSER_EXTENSIONS="/ext1,/ext2"       # Comma-separated extension paths
AGENT_BROWSER_INIT_SCRIPTS="/a.js,/b.js"     # Comma-separated init script paths
AGENT_BROWSER_ENABLE="react-devtools"        # Comma-separated built-in init script features
AGENT_BROWSER_HIDE_SCROLLBARS="false"        # Keep native scrollbars visible in headless Chromium screenshots
AGENT_BROWSER_PROVIDER="browserbase"         # Browser provider or configured provider plugin
AGENT_BROWSER_STREAM_PORT="9223"             # Override WebSocket streaming port (default: OS-assigned)
AGENT_BROWSER_CONFIG="./web-action.json"  # Custom config file
AGENT_BROWSER_CDP="9222"                     # Connect daemon to CDP port or WebSocket URL
AGENT_BROWSER_PLUGINS='[{"name":"vault","command":"web-action-plugin-vault","capabilities":["credential.read"]},{"name":"stealth","command":"web-action-plugin-stealth","capabilities":["launch.mutate"]}]'
```
