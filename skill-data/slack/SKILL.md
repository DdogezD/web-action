---
name: slack
description: Interact with Slack workspaces using browser automation. Use when the user needs to check unread channels, navigate Slack, send messages, extract data, find information, search conversations, or automate any Slack task. Triggers include "check my Slack", "what channels have unreads", "send a message to", "search Slack for", "extract from Slack", "find who said", or any task requiring programmatic Slack interaction.
allowed-tools: Bash(web-action:*)
---

# Slack Automation

Interact with Slack workspaces to check messages, extract data, and automate common tasks.

## Quick Start

Connect to an existing Slack browser session or open Slack:

```bash
# Connect to existing session on port 9222 (typical for already-open Slack)
web-action connect 9222

# Or open Slack if not already running
web-action open https://app.slack.com
```

Then take a snapshot to see what's available:

```bash
web-action snapshot -i
```

## Core Workflow

1. **Connect/Navigate**: Open or connect to Slack
2. **Snapshot**: Get interactive elements with refs (`@e1`, `@e2`, etc.)
3. **Navigate**: Click tabs, expand sections, or navigate to specific channels
4. **Extract/Interact**: Read data or perform actions
5. **Screenshot**: Capture evidence of findings

```bash
# Example: Check unread channels
web-action connect 9222
web-action snapshot -i
# Look for "More unreads" button
web-action click @e21  # Ref for "More unreads" button
web-action screenshot slack-unreads.png
```

## Common Tasks

### Checking Unread Messages

```bash
# Connect to Slack
web-action connect 9222

# Take snapshot to locate unreads button
web-action snapshot -i

# Look for:
# - "More unreads" button (usually near top of sidebar)
# - "Unreads" toggle in Activity tab (shows unread count)
# - Channel names with badges/bold text indicating unreads

# Navigate to Activity tab to see all unreads in one view
web-action click @e14  # Activity tab (ref may vary)
web-action wait 1000
web-action screenshot activity-unreads.png

# Or check DMs tab
web-action click @e13  # DMs tab
web-action screenshot dms.png

# Or expand "More unreads" in sidebar
web-action click @e21  # More unreads button
web-action wait 500
web-action screenshot expanded-unreads.png
```

### Navigating to a Channel

```bash
# Search for channel in sidebar or by name
web-action snapshot -i

# Look for channel name in the list (e.g., "engineering", "product-design")
# Click on the channel treeitem ref
web-action click @e94  # Example: engineering channel ref
web-action wait --load networkidle
web-action screenshot channel.png
```

### Finding Messages/Threads

```bash
# Use Slack search
web-action snapshot -i
web-action click @e5  # Search button (typical ref)
web-action fill @e_search "keyword"
web-action press Enter
web-action wait --load networkidle
web-action screenshot search-results.png
```

### Extracting Channel Information

```bash
# Get list of all visible channels
web-action snapshot --json > slack-snapshot.json

# Parse for channel names and metadata
# Look for treeitem elements with level=2 (sub-channels under sections)
```

### Checking Channel Details

```bash
# Open a channel
web-action click @e_channel_ref
web-action wait 1000

# Get channel info (members, description, etc.)
web-action snapshot -i
web-action screenshot channel-details.png

# Scroll through messages
web-action scroll down 500
web-action screenshot channel-messages.png
```

### Taking Notes/Capturing State

When you need to document findings from Slack:

```bash
# Take annotated screenshot (shows element numbers)
web-action screenshot --annotate slack-state.png

# Take full-page screenshot
web-action screenshot --full slack-full.png

# Get current URL for reference
web-action get url

# Get page title
web-action get title
```

## Sidebar Structure

Understanding Slack's sidebar helps you navigate efficiently:

```
- Threads
- Huddles
- Drafts & sent
- Directories
- [Section Headers - External connections, Starred, Channels, etc.]
  - [Channels listed as treeitems]
- Direct Messages
  - [DMs listed]
- Apps
  - [App shortcuts]
- [More unreads] button (toggles unread channels list)
```

Key refs to look for:
- `@e12` - Home tab (usually)
- `@e13` - DMs tab
- `@e14` - Activity tab
- `@e5` - Search button
- `@e21` - More unreads button (varies by session)

## Tabs in Slack

After clicking on a channel, you'll see tabs:
- **Messages** - Channel conversation
- **Files** - Shared files
- **Pins** - Pinned messages
- **Add canvas** - Collaborative canvas
- Other tabs depending on workspace setup

Click tab refs to switch views and get different information.

## Extracting Data from Slack

### Get Text Content

```bash
# Get a message or element's text
web-action get text @e_message_ref
```

### Parse Accessibility Tree

```bash
# Full snapshot as JSON for programmatic parsing
web-action snapshot --json > output.json

# Look for:
# - Channel names (name field in treeitem)
# - Message content (in listitem/document elements)
# - User names (button elements with user info)
# - Timestamps (link elements with time info)
```

### Count Unreads

```bash
# After expanding unreads section:
web-action snapshot -i | grep -c "treeitem"
# Each treeitem with a channel name in the unreads section is one unread
```

## Best Practices

- **Connect to existing sessions**: Use `web-action connect 9222` if Slack is already open. This is faster than opening a new browser.
- **Take snapshots before clicking**: Always `snapshot -i` to identify refs before clicking buttons.
- **Re-snapshot after navigation**: After navigating to a new channel or section, take a fresh snapshot to find new refs.
- **Use JSON snapshots for parsing**: When you need to extract structured data, use `snapshot --json` for machine-readable output.
- **Pace interactions**: Add `sleep 1` between rapid interactions to let the UI update.
- **Check accessibility tree**: The accessibility tree shows what screen readers (and your automation) can see. If an element isn't in the snapshot, it may be hidden or require scrolling.
- **Scroll in sidebar**: Use `web-action scroll down 300 --selector ".p-sidebar"` to scroll within the Slack sidebar if channel list is long.

## Limitations

- **Cannot access Slack API**: This uses browser automation, not the Slack API. No OAuth, webhooks, or bot tokens needed.
- **Session-specific**: Screenshots and snapshots are tied to the current browser session.
- **Rate limiting**: Slack may rate-limit rapid interactions. Add delays between commands if needed.
- **Workspace-specific**: You interact with your own workspace -- no cross-workspace automation.

## Debugging

### Check console for errors

```bash
web-action console
web-action errors
```

### Get current page state

```bash
web-action get url
web-action get title
web-action screenshot page-state.png
```

## Example: Full Unread Check

```bash
#!/bin/bash

# Connect to Slack
web-action connect 9222

# Take initial snapshot
echo "=== Checking Slack unreads ==="
web-action snapshot -i > snapshot.txt

# Check Activity tab for unreads
web-action click @e14  # Activity tab
web-action wait 1000
web-action screenshot activity.png
ACTIVITY_RESULT=$(web-action get text @e_main_area)
echo "Activity: $ACTIVITY_RESULT"

# Check DMs
web-action click @e13  # DMs tab
web-action wait 1000
web-action screenshot dms.png

# Check unread channels in sidebar
web-action click @e21  # More unreads button
web-action wait 500
web-action snapshot -i > unreads-expanded.txt
web-action screenshot unreads.png

# Summary
echo "=== Summary ==="
echo "See activity.png, dms.png, and unreads.png for full details"
```

## References

- **Slack docs**: https://slack.com/help
- **Web experience**: https://app.slack.com
- **Keyboard shortcuts**: Type `?` in Slack for shortcut list
