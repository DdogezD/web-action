# Session Management

Multiple isolated browser sessions with state persistence and concurrent browsing.

**Related**: [authentication.md](authentication.md) for login patterns, [SKILL.md](../SKILL.md) for quick start.

## Contents

- [Named Sessions](#named-sessions)
- [Session Isolation Properties](#session-isolation-properties)
- [Session State Persistence](#session-state-persistence)
- [Common Patterns](#common-patterns)
- [Default Session](#default-session)
- [Session Cleanup](#session-cleanup)
- [Best Practices](#best-practices)

## Named Sessions

Use `--session` to isolate browser contexts. Agent skills should derive one stable id and reuse it on every command:

```bash
SESSION="$(web-action session id --scope worktree --prefix my-skill)"
web-action --session "$SESSION" --restore open https://app.example.com/login
```

`--scope worktree` uses the Git worktree root when available, then the Git root, then the canonical current directory. This is the recommended default for agents because worktrees are commonly used for parallel agent runs.

```bash
# Session 1: Authentication flow
web-action --session auth open https://app.example.com/login

# Session 2: Public browsing (separate cookies, storage)
web-action --session public open https://example.com

# Commands are isolated by session
web-action --session auth fill @e1 "user@example.com"
web-action --session public get text body
```

## Session Isolation Properties

Each session has independent:
- Cookies
- LocalStorage / SessionStorage
- IndexedDB
- Cache
- Browsing history
- Open tabs

## Session State Persistence

### Automatic Restore

```bash
# Bare --restore uses the current --session as the persistence key
SESSION="$(web-action session id --scope worktree --prefix next-dev-loop)"
web-action --session "$SESSION" --restore open https://app.example.com/dashboard
```

State is loaded before navigation and saved on close, daemon shutdown, idle timeout, and compatible relaunch. It is also saved periodically while the browser is open (after commands settle, at most once per `WEB_ACTION_AUTOSAVE_INTERVAL_MS`, default 30000; set to `0` to save only on close), so a browser window the user closes by hand still leaves a recent save behind. Idle sessions keep saving on the same interval, capturing changes the page makes on its own such as token refreshes. The default save policy is `--restore-save auto`, which skips auto-save if restore failed or validation failed; `never` disables periodic autosave too.

```bash
web-action --session "$SESSION" --restore --restore-check-url "**/dashboard" open https://app.example.com/dashboard
web-action --session "$SESSION" --restore --restore-check-text Dashboard open https://app.example.com/dashboard
web-action --session "$SESSION" --restore --restore-check-fn "!!localStorage.getItem('session')" open https://app.example.com/dashboard
```

Use `web-action session info --json` for diagnostics:

```bash
web-action --session "$SESSION" session info --json
```

### Manual State Files

Use `state save`, `state load`, and `--state <path>` when you need an explicit portable JSON file. Do not make agents construct paths under `~/.web-action/sessions/`; prefer `--restore` for reusable agent sessions.

## Common Patterns

### Authenticated Session Reuse

```bash
#!/bin/bash
SESSION="$(web-action session id --scope worktree --prefix app)"
web-action --session "$SESSION" --restore open https://app.example.com/dashboard
```

### Concurrent Scraping

```bash
#!/bin/bash
# Scrape multiple sites concurrently

# Start all sessions
web-action --session site1 open https://site1.com &
web-action --session site2 open https://site2.com &
web-action --session site3 open https://site3.com &
wait

# Extract from each
web-action --session site1 get text body > site1.txt
web-action --session site2 get text body > site2.txt
web-action --session site3 get text body > site3.txt

# Cleanup
web-action --session site1 close
web-action --session site2 close
web-action --session site3 close
```

### A/B Testing Sessions

```bash
# Test different user experiences
web-action --session variant-a open "https://app.com?variant=a"
web-action --session variant-b open "https://app.com?variant=b"

# Compare
web-action --session variant-a screenshot /tmp/variant-a.png
web-action --session variant-b screenshot /tmp/variant-b.png
```

## Default Session

When `--session` is omitted, commands use the default session:

```bash
# These use the same default session
web-action open https://example.com
web-action snapshot -i
web-action close  # Closes default session
```

## Session Cleanup

```bash
# Close specific session
web-action --session auth close

# List active sessions
web-action session list
```

## Best Practices

### 1. Name Sessions Semantically

```bash
# GOOD: Clear purpose
web-action --session github-auth open https://github.com
web-action --session docs-scrape open https://docs.example.com

# AVOID: Generic names
web-action --session s1 open https://github.com
```

### 2. Always Clean Up

```bash
# Close sessions when done
web-action --session auth close
web-action --session scrape close
```

### 3. Handle State Files Securely

```bash
# Don't commit state files (contain auth tokens!)
echo "*.auth-state.json" >> .gitignore

# Delete after use
rm /tmp/auth-state.json
```

### 4. Timeout Long Sessions

```bash
# Set timeout for automated scripts
timeout 60 web-action --session long-task get text body
```
