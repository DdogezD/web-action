# web-action

Headless browser automation CLI for AI agents — persistent sessions with stealth.

Forked from [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) and enhanced with anti-detection stealth, `--tabname` multi-tab isolation, persistent profiles, and cross-frame interaction (`--frame`). Uses your system browser — no bundled Chrome download needed.

## Why This Fork?

| | `agent-browser` | `web-action` |
|---|---|---|
| **Browser** | Chrome for Testing (downloaded) | System browser (auto-detected) |
| **Profile** | Temp dir (lost on close) | Persistent (`~/.web-action/profiles/main/`) |
| **Stealth** | None | CDP-level anti-detection (no `Runtime.enable`, no `--enable-automation`, SW stub, webdriver spoof) |
| **Tab routing** | Index-based (`tab 0`, `tab 1`) | `--tabname` named labels (parallel-safe) |
| **Iframe interaction** | Manual `frame`/`mainframe` commands only | Snapshot auto-inlines iframe content with frame boundary annotations; refs work transparently across frames; `--frame <sel>` scopes any CSS-selector command to a single iframe (supports ref, CSS, name, title, and URL matching) |
| **Engine** | Playwright-flavored CDP flags | Minimal flags that don't signal automation |
| **Permissions** | Batch grant only via `Browser.grantPermissions` (no origin parameter) | `permissions` CLI with per-type grant/deny/prompt/reset via `Browser.setPermission`, explicit origin scoping |

## Installation

### One-line install (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/DdogezD/web-action/main/install.sh | bash
```

This clones the repo, builds the Rust binary, and adds `web-action` to your PATH.

**Requirements:** git, rust (cargo), and a Chromium-based browser (chrome, chromium, brave, edge).

### Manual install from source

```bash
git clone https://github.com/DdogezD/web-action.git
cd web-action
cargo build --release --manifest-path cli/Cargo.toml
cp cli/target/release/web-action ~/.local/bin/   # or anywhere in PATH
```

Use `--executable-path` to specify a custom browser if auto-detection doesn't find yours.

## Quick Start

```bash
web-action open example.com
web-action snapshot -i               # Get interactive elements with refs (@e1, @e2)
web-action click @e2                  # Click by ref
web-action fill @e3 "test@example.com"
web-action screenshot page.png
```

## Login Persistence

The browser profile is stored persistently. Login once, stay logged in across sessions.

```bash
# First time: login manually (or navigate, cookies persist)
web-action open https://accounts.google.com

# Later — session state survives daemon restarts
web-action open https://mail.google.com  # Still logged in!
```

Profile directory: `~/.web-action/profiles/main/`

When using `--restore`, state is saved when the browser closes and periodically while it remains open. Periodic autosave waits for commands to settle and runs at most once per `WEB_ACTION_AUTOSAVE_INTERVAL_MS` (default: `30000`; set to `0` to save only on close). It respects the `--restore-save` policy.

## Cross-Frame Interaction (`--frame`)

When elements live inside an iframe, use `--frame` to target them with CSS selectors without manually switching frames:

```bash
# snapshot shows iframe boundaries, refs inside iframes work directly
web-action snapshot -i
# - Iframe "Payment" [ref=e3]
#   -- frame "Payment" [ref=e3] --
#   - textbox "Card number" [ref=e4]

# Ref-based interaction auto-routes to the correct frame (no --frame needed)
web-action fill @e4 "4111111111111111"

# CSS selectors need --frame to scope into the iframe
web-action click --frame "#payment-iframe" ".pay-button"
web-action fill --frame '[name="checkout"]' "[name=cc]" "4111111111111111"
web-action snapshot --frame "#widget" -i          # scoped snapshot
```

`--frame` accepts:
- A CSS selector for the iframe element (`"#payment-iframe"`, `'[name="form"]'`)
- An element ref pointing to an iframe (`@e3`)
- An iframe name or URL substring (`"payment"`, `"checkout.html"`)

The frame context only lasts for that one command; subsequent commands revert to the main frame.

## Multi-Tab Isolation (`--tabname`)

Multiple CLI clients can operate independent tabs in the same browser instance.

```bash
# Agent A: browse Reddit
web-action --tabname reddit open https://reddit.com
web-action --tabname reddit snapshot -i
web-action --tabname reddit click @e5

# Agent B: simultaneously browse Hacker News (same browser, different tab)
web-action --tabname hackernews open https://news.ycombinator.com
web-action --tabname hackernews snapshot -i
web-action --tabname hackernews click @e3
```

Tab isolation guarantees:

| Isolated per tab | Shared across tabs |
|---|---|
| Page (DOM, URL) | Cookies, localStorage |
| CDP session | Browser profile |
| Snapshot refs (`@e1`, `@e2`) | Chrome process |
| Frame context | |

Without `--tabname`, commands default to the `ZEROTABPAGE` tab.

## Stealth

web-action includes CDP-level anti-detection inspired by [Patchright](https://github.com/Kaliiiiiiiiii-Vinyzu/patchright) research:

| Mechanism | Detail |
|---|---|
| **No `Runtime.enable`** | Avoids broadcasting execution context events |
| **No `--enable-automation`** | Removes the primary bot indicator flag |
| **`--disable-blink-features=AutomationControlled`** | Hides `navigator.webdriver` |
| **No SwiftShader** | Removes `--enable-unsafe-swiftshader` (bot signal) |
| **Minimal Chrome flags** | Only flags a normal Chrome process might use |
| **Service worker stub** | Prevents SW-based fingerprinting |
| **Navigator spoofing** | `navigator.webdriver` → false, `window.chrome` → present, plugins/languages faked |

## Commands

All commands are compatible with upstream `agent-browser`. See the [upstream README](https://github.com/vercel-labs/agent-browser#readme) for the full command reference.

### Core

```bash
web-action open <url>              # Navigate to URL
web-action click <sel>             # Click element
web-action fill <sel> <text>       # Clear and fill
web-action press <key>             # Press key
web-action snapshot                # Accessibility tree with refs
web-action snapshot -i             # Interactive elements only
web-action screenshot [path]       # Take screenshot
```

### Get Info

```bash
web-action get text <sel>          # Get text content
web-action get url                 # Get current URL
web-action get title               # Get page title
```

### Wait

```bash
web-action wait <selector>         # Wait for element
web-action wait --load networkidle # Wait for network idle
web-action wait --url "**/dash"    # Wait for URL pattern
```

### Tabs

```bash
web-action tab                     # List tabs (includes named tabs)
web-action tab new [url]           # New tab
web-action tab <tN>                # Switch to tab N
web-action tab close [tN]          # Close tab
```

### Cookies & Storage

```bash
web-action cookies                 # Get all cookies
web-action storage local           # Get localStorage
```

### Daemon Management

```bash
web-action kill                    # Kill daemon + close browser
```

## Options

| Option | Description |
|---|---|
| `--tabname, -t <name>` | Named tab for client isolation (default: `ZEROTABPAGE`) |
| `--headed` | Show browser window (default: headless) |
| `--executable-path <path>` | Custom browser executable |
| `--channel <name>` | Browser channel: `chrome`, `msedge`, `chrome-beta` |
| `--profile <path>` | Custom profile directory |
| `--json` | JSON output for agents |
| `--session <name>` | Isolated session |
| `--debug` | Debug output |

## Environment Variables

| Variable | Description | Default |
|---|---|---|
| `WEB_ACTION_TABNAME` | Named tab for isolation | `ZEROTABPAGE` |
| `WEB_ACTION_HEADED` | Browser mode (`1`/`true` for visible) | (headless) |
| `WEB_ACTION_EXECUTABLE_PATH` | Custom browser path | system chromium |
| `WEB_ACTION_SESSION` | Session name | `default` |
| `WEB_ACTION_SOCKET_DIR` | Override IPC directory | `~/.web-action` |
| `WEB_ACTION_PROFILE` | Custom profile directory | `~/.web-action/profiles/main/` |
| `WEB_ACTION_IDLE_TIMEOUT_MS` | Daemon auto-shutdown (ms) | (disabled) |
| `WEB_ACTION_AUTOSAVE_INTERVAL_MS` | Minimum time between periodic restore-state saves (ms; `0` disables) | `30000` |

### Directory Layout

```
~/.web-action/
├── default.sock          # IPC socket
├── default.pid           # Daemon PID
├── default.version       # Daemon version
└── profiles/
    └── main/             # Persistent browser data (cookies, auth, cache)
```

## License

Apache-2.0
