# Authentication Patterns

Login flows, session persistence, OAuth, 2FA, and authenticated browsing.

**Related**: [session-management.md](session-management.md) for state persistence details, [SKILL.md](../SKILL.md) for quick start.

## Contents

- [Import Auth from Your Browser](#import-auth-from-your-browser)
- [Persistent Profiles](#persistent-profiles)
- [Session Persistence](#session-persistence)
- [Basic Login Flow](#basic-login-flow)
- [Plugins](#plugins)
- [Saving Authentication State](#saving-authentication-state)
- [Restoring Authentication](#restoring-authentication)
- [OAuth / SSO Flows](#oauth--sso-flows)
- [Two-Factor Authentication](#two-factor-authentication)
- [HTTP Basic Auth](#http-basic-auth)
- [Cookie-Based Auth](#cookie-based-auth)
- [Token Refresh Handling](#token-refresh-handling)
- [Security Best Practices](#security-best-practices)

## Import Auth from Your Browser

The fastest way to authenticate is to reuse cookies from a Chrome session you are already logged into.

**Step 1: Start Chrome with remote debugging**

```bash
# macOS
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --remote-debugging-port=9222

# Linux
google-chrome --remote-debugging-port=9222

# Windows
"C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
```

Log in to your target site(s) in this Chrome window as you normally would.

> **Security note:** `--remote-debugging-port` exposes full browser control on localhost. Any local process can connect and read cookies, execute JS, etc. Only use on trusted machines and close Chrome when done.

**Step 2: Grab the auth state**

```bash
# Auto-discover the running Chrome and save its cookies + localStorage
web-action --auto-connect state save ./my-auth.json
```

**Step 3: Reuse in automation**

```bash
# Load auth at launch
web-action --state ./my-auth.json open https://app.example.com/dashboard

# Or load into an already-launched session
web-action open about:blank
web-action state load ./my-auth.json
web-action open https://app.example.com/dashboard
```

This works for any site, including those with complex OAuth flows, SSO, or 2FA, as long as Chrome already has valid session cookies.

> **Security note:** State files contain session tokens in plaintext. Add them to `.gitignore`, delete when no longer needed, and set `AGENT_BROWSER_ENCRYPTION_KEY` for encryption at rest. See [Security Best Practices](#security-best-practices).

**Tip:** Combine with `--session <id> --restore` so the imported auth auto-persists across restarts:

```bash
SESSION="$(web-action session id --scope worktree --prefix myapp)"
web-action --session "$SESSION" --restore --state ./my-auth.json open https://app.example.com/dashboard
# From now on, state is auto-saved/restored for this session
```

## Persistent Profiles

Use `--profile` to point web-action at a Chrome user data directory. This persists everything (cookies, IndexedDB, service workers, cache) across browser restarts without explicit save/load:

```bash
# First run: login once
web-action --profile ~/.myapp-profile open https://app.example.com/login
# ... complete login flow ...

# All subsequent runs: already authenticated
web-action --profile ~/.myapp-profile open https://app.example.com/dashboard
```

Use different paths for different projects or test users:

```bash
web-action --profile ~/.profiles/admin open https://app.example.com
web-action --profile ~/.profiles/viewer open https://app.example.com
```

Or set via environment variable:

```bash
export AGENT_BROWSER_PROFILE=~/.myapp-profile
web-action open https://app.example.com/dashboard
```

## Session Persistence

Use `--restore` with a stable `--session` to auto-save and restore cookies + localStorage without managing files:

```bash
# Auto-saves state on close, auto-restores on next launch
SESSION="$(web-action session id --scope worktree --prefix twitter)"
web-action --session "$SESSION" --restore open https://twitter.com
# ... login flow ...
web-action --session "$SESSION" --restore close  # state saved to ~/.web-action/sessions/

# Next time: state is automatically restored
web-action --session "$SESSION" --restore open https://twitter.com
```

Encrypt state at rest:

```bash
export AGENT_BROWSER_ENCRYPTION_KEY=$(openssl rand -hex 32)
web-action --session secure --restore open https://app.example.com
```

## Basic Login Flow

```bash
# Navigate to login page
web-action open https://app.example.com/login
web-action wait --load networkidle

# Get form elements
web-action snapshot -i
# Output: @e1 [input type="email"], @e2 [input type="password"], @e3 [button] "Sign In"

# Fill credentials
web-action fill @e1 "user@example.com"
web-action fill @e2 "password123"

# Submit
web-action click @e3
web-action wait --load networkidle

# Verify login succeeded
web-action get url  # Should be dashboard, not login
```

## Plugins

Use credential provider plugins when credentials live in external vault software. Plugins are configured in `web-action.json` and run as external executables over the `web-action.plugin.v1` stdio JSON protocol.

Add a plugin with `plugin add`. A plain `name` or `@scope/name` resolves from npm; `owner/repo` resolves from GitHub:

```bash
web-action plugin add web-action-plugin-vault --name vault
web-action plugin add @company/web-action-plugin-vault --name vault
web-action plugin add org/web-action-plugin-cloud-browser
```

```json
{
  "plugins": [
    {
      "name": "vault",
      "command": "web-action-plugin-vault",
      "capabilities": ["credential.read"]
    },
    {
      "name": "cloud-browser",
      "command": "web-action-plugin-cloud-browser",
      "capabilities": ["browser.provider"]
    },
    {
      "name": "stealth",
      "command": "web-action-plugin-stealth",
      "capabilities": ["launch.mutate"]
    },
    {
      "name": "captcha",
      "command": "web-action-plugin-captcha",
      "capabilities": ["command.run", "captcha.solve"]
    }
  ]
}
```

Inspect configured plugins before use:

```bash
web-action plugin list
web-action plugin show vault
```

Resolve credentials just-in-time for one login:

```bash
web-action auth login my-app --credential-provider vault --item "My App"
```

Use a plugin as a browser provider or a generic domain command:

```bash
web-action --provider cloud-browser open https://example.com
web-action plugin run captcha captcha.solve --payload '{"siteKey":"...","url":"https://example.com"}'
```

`plugin run` is for `command.run` and custom capabilities. Core capabilities and protocol request types use their dedicated command paths.

Use `--url`, `--username-selector`, `--password-selector`, and `--submit-selector` on `auth login` to override plugin-provided metadata for the current login only.

Gate plugin secret access separately from normal login automation:

```bash
web-action --confirm-actions plugin:vault:credential.read auth login my-app --credential-provider vault --item "My App"
web-action --confirm-actions plugin:cloud-browser:browser.provider --provider cloud-browser open https://example.com
web-action --confirm-actions plugin:stealth:launch.mutate open https://example.com
```

Do not put vault tokens or passwords in plugin command args. Use the vault vendor's own login/session mechanism or environment outside web-action config.

## Saving Authentication State

After logging in, save state for reuse:

```bash
# Login first (see above)
web-action open https://app.example.com/login
web-action snapshot -i
web-action fill @e1 "user@example.com"
web-action fill @e2 "password123"
web-action click @e3
web-action wait --url "**/dashboard"

# Save authenticated state
web-action state save ./auth-state.json
```

## Restoring Authentication

Skip login by loading saved state:

```bash
# Load saved auth state
web-action state load ./auth-state.json

# Navigate directly to protected page
web-action open https://app.example.com/dashboard

# Verify authenticated
web-action snapshot -i
```

## OAuth / SSO Flows

For OAuth redirects:

```bash
# Start OAuth flow
web-action open https://app.example.com/auth/google

# Handle redirects automatically
web-action wait --url "**/accounts.google.com**"
web-action snapshot -i

# Fill Google credentials
web-action fill @e1 "user@gmail.com"
web-action click @e2  # Next button
web-action wait 2000
web-action snapshot -i
web-action fill @e3 "password"
web-action click @e4  # Sign in

# Wait for redirect back
web-action wait --url "**/app.example.com**"
web-action state save ./oauth-state.json
```

## Two-Factor Authentication

Handle 2FA with manual intervention:

```bash
# Login with credentials
web-action open https://app.example.com/login --headed  # Show browser
web-action snapshot -i
web-action fill @e1 "user@example.com"
web-action fill @e2 "password123"
web-action click @e3

# Wait for user to complete 2FA manually
echo "Complete 2FA in the browser window..."
web-action wait --url "**/dashboard" --timeout 120000

# Save state after 2FA
web-action state save ./2fa-state.json
```

## HTTP Basic Auth

For sites using HTTP Basic Authentication:

```bash
# Set credentials before navigation
web-action set credentials username password

# Navigate to protected resource
web-action open https://protected.example.com/api
```

## Cookie-Based Auth

Manually set authentication cookies:

```bash
# Set auth cookie
web-action cookies set session_token "abc123xyz"

# Navigate to protected page
web-action open https://app.example.com/dashboard
```

## Token Refresh Handling

For sessions with expiring tokens:

```bash
#!/bin/bash
# Wrapper that handles token refresh

STATE_FILE="./auth-state.json"

# Try loading existing state
if [[ -f "$STATE_FILE" ]]; then
    web-action state load "$STATE_FILE"
    web-action open https://app.example.com/dashboard

    # Check if session is still valid
    URL=$(web-action get url)
    if [[ "$URL" == *"/login"* ]]; then
        echo "Session expired, re-authenticating..."
        # Perform fresh login
        web-action snapshot -i
        web-action fill @e1 "$USERNAME"
        web-action fill @e2 "$PASSWORD"
        web-action click @e3
        web-action wait --url "**/dashboard"
        web-action state save "$STATE_FILE"
    fi
else
    # First-time login
    web-action open https://app.example.com/login
    # ... login flow ...
fi
```

## Security Best Practices

1. **Never commit state files** - They contain session tokens
   ```bash
   echo "*.auth-state.json" >> .gitignore
   ```

2. **Use environment variables for credentials**
   ```bash
   web-action fill @e1 "$APP_USERNAME"
   web-action fill @e2 "$APP_PASSWORD"
   ```

3. **Clean up after automation**
   ```bash
   web-action cookies clear
   rm -f ./auth-state.json
   ```

4. **Use short-lived sessions for CI/CD**
   ```bash
   # Don't persist state in CI
   web-action open https://app.example.com/login
   # ... login and perform actions ...
   web-action close  # Session ends, nothing persisted
   ```
