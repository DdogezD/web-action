# Video Recording

Capture browser automation as video for debugging, documentation, or verification.

**Related**: [commands.md](commands.md) for full command reference, [SKILL.md](../SKILL.md) for quick start.

## Contents

- [Basic Recording](#basic-recording)
- [Recording Commands](#recording-commands)
- [Use Cases](#use-cases)
- [Best Practices](#best-practices)
- [Output Format](#output-format)
- [Limitations](#limitations)

## Basic Recording

```bash
# Launch the browser, then start recording
web-action open https://example.com
web-action record start ./demo.webm

# Perform actions
web-action snapshot -i
web-action click @e1
web-action fill @e2 "test input"

# Stop and save
web-action record stop
```

## Recording Commands

```bash
# Launch a session first
web-action open

# Start recording to file
web-action record start ./output.webm

# Stop current recording
web-action record stop

# Restart with new file (stops current + starts new)
web-action record restart ./take2.webm
```

## Use Cases

### Debugging Failed Automation

```bash
#!/bin/bash
# Record automation for debugging

# Run your automation
web-action open https://app.example.com
web-action record start ./debug-$(date +%Y%m%d-%H%M%S).webm
web-action snapshot -i
web-action click @e1 || {
    echo "Click failed - check recording"
    web-action record stop
    exit 1
}

web-action record stop
```

### Documentation Generation

```bash
#!/bin/bash
# Record workflow for documentation

web-action open https://app.example.com/login
web-action record start ./docs/how-to-login.webm
web-action wait 1000  # Pause for visibility

web-action snapshot -i
web-action fill @e1 "demo@example.com"
web-action wait 500

web-action fill @e2 "password"
web-action wait 500

web-action click @e3
web-action wait --load networkidle
web-action wait 1000  # Show result

web-action record stop
```

### CI/CD Test Evidence

```bash
#!/bin/bash
# Record E2E test runs for CI artifacts

TEST_NAME="${1:-e2e-test}"
RECORDING_DIR="./test-recordings"
mkdir -p "$RECORDING_DIR"

web-action open
web-action record start "$RECORDING_DIR/$TEST_NAME-$(date +%s).webm"

# Run test
if run_e2e_test; then
    echo "Test passed"
else
    echo "Test failed - recording saved"
fi

web-action record stop
```

## Best Practices

### 1. Add Pauses for Clarity

```bash
# Slow down for human viewing
web-action click @e1
web-action wait 500  # Let viewer see result
```

### 2. Use Descriptive Filenames

```bash
# Include context in filename
web-action record start ./recordings/login-flow-2024-01-15.webm
web-action record start ./recordings/checkout-test-run-42.webm
```

### 3. Handle Recording in Error Cases

```bash
#!/bin/bash
set -e

cleanup() {
    web-action record stop 2>/dev/null || true
    web-action close 2>/dev/null || true
}
trap cleanup EXIT

web-action open
web-action record start ./automation.webm
# ... automation steps ...
```

### 4. Combine with Screenshots

```bash
# Record video AND capture key frames
web-action open https://example.com
web-action record start ./flow.webm
web-action screenshot ./screenshots/step1-homepage.png

web-action click @e1
web-action screenshot ./screenshots/step2-after-click.png

web-action record stop
```

## Output Format

- Default format: WebM (VP8/VP9 codec)
- Compatible with all modern browsers and video players
- Compressed but high quality

## Limitations

- Recording adds slight overhead to automation
- Large recordings can consume significant disk space
- Some headless environments may have codec limitations
