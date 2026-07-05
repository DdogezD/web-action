#!/bin/bash
# Template: Form Automation Workflow
# Purpose: Fill and submit web forms with validation
# Usage: ./form-automation.sh <form-url>
#
# This template demonstrates the snapshot-interact-verify pattern:
# 1. Navigate to form
# 2. Snapshot to get element refs
# 3. Fill fields using refs
# 4. Submit and verify result
#
# Customize: Update the refs (@e1, @e2, etc.) based on your form's snapshot output

set -euo pipefail

FORM_URL="${1:?Usage: $0 <form-url>}"

echo "Form automation: $FORM_URL"

# Step 1: Navigate to form
web-action open "$FORM_URL"
web-action wait --load networkidle

# Step 2: Snapshot to discover form elements
echo ""
echo "Form structure:"
web-action snapshot -i

# Step 3: Fill form fields (customize these refs based on snapshot output)
#
# Common field types:
#   web-action fill @e1 "John Doe"           # Text input
#   web-action fill @e2 "user@example.com"   # Email input
#   web-action fill @e3 "SecureP@ss123"      # Password input
#   web-action select @e4 "Option Value"     # Dropdown
#   web-action check @e5                     # Checkbox
#   web-action click @e6                     # Radio button
#   web-action fill @e7 "Multi-line text"   # Textarea
#   web-action upload @e8 /path/to/file.pdf # File upload
#
# Uncomment and modify:
# web-action fill @e1 "Test User"
# web-action fill @e2 "test@example.com"
# web-action click @e3  # Submit button

# Step 4: Wait for submission
# web-action wait --load networkidle
# web-action wait --url "**/success"  # Or wait for redirect

# Step 5: Verify result
echo ""
echo "Result:"
web-action get url
web-action snapshot -i

# Optional: Capture evidence
web-action screenshot /tmp/form-result.png
echo "Screenshot saved: /tmp/form-result.png"

# Cleanup
web-action close
echo "Done"
