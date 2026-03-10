---
description: Connect your RINDA AI account. Run once before using other RINDA commands.
allowed-tools: ["Bash", "Read"]
argument-hint: "(no arguments needed)"
---

# /rinda-ai:connect

Connect your RINDA AI account. Run this once before using any other RINDA commands.

## Parameters

None.

## Workflow

### Step 1 — Ensure CLI is installed

Run the install script to download the CLI binary if it is not already present:

```
${CLAUDE_PLUGIN_ROOT}/bin/install.sh
```

### Step 2 — Check if already logged in

```
${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth status
```

If the user is already logged in with a valid token, skip to Step 5.

### Step 3 — Get the login URL

```
${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth url
```

Show the URL to the user and ask them to:
1. Open the URL in their browser.
2. Sign in with their Google account.
3. Copy the refresh token displayed on the page.
4. Paste the token back here.

### Step 4 — Authenticate with the token

Once the user provides the token, run:

```
${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth token <PASTED_TOKEN>
```

The CLI exchanges the refresh token for an access token and saves credentials to `~/.rinda/credentials.json`.

### Step 5 — Verify the connection

Run `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth status` and display:

```
Connected to RINDA AI
  Account:   kim@company.com
  Workspace: <workspaceId>
  Token:     valid (expires in N minutes)

You're ready to start finding buyers. Try:
  /rinda-ai:buyer-search
```

## Error Handling

| Error | Resolution |
|-------|------------|
| Install script fails | Check internet connection and retry |
| "Invalid or expired token" | Get a fresh token from the login URL |
| Token expires during session | The plugin hook auto-refreshes before each command |

## Notes

- Tokens are auto-refreshed before every tool call by the plugin hook (`rinda-cli auth ensure-valid`).
- To check status: `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth status`
- To log out: `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth logout`
