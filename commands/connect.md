# /rinda-ai:connect

Connect your RINDA AI account via Google OAuth. Run this once before using any other RINDA commands.

## Parameters

None.

## Credentials

After login, credentials are stored at `~/.rinda/credentials.json`:

```json
{
  "accessToken":  "eyJ...",
  "refreshToken": "a8f3...",
  "expiresAt":    1709726400000,
  "workspaceId":  "uuid",
  "userId":       "uuid",
  "email":        "kim@company.com"
}
```

## Workflow

### Step 1 — Run the CLI login command

Tell the user to run the following command in their terminal:

```
${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth login
```

Explain what happens:
1. The CLI starts a local HTTP server on port 9876.
2. It opens the browser to `https://app.rinda.ai/api/v1/auth/google`.
3. The user signs in with their Google account.
4. Google redirects back to `http://localhost:9876/callback?code=xxx`.
5. The CLI exchanges the code for tokens from the backend.
6. Tokens are written to `~/.rinda/credentials.json` with file permissions 600.
7. The terminal prints: `Logged in as <email>`.

### Step 2 — Verify the connection

After the user confirms login, read `~/.rinda/credentials.json` and display the connection status:

```
RINDA AI Connection Status
==========================
Email:       kim@company.com
Workspace:   <workspaceId>
Token valid: yes (expires in 58 min)
Session:     active (refresh token expires in 14 days)
```

Calculate token expiry from `expiresAt` (milliseconds timestamp):
- If `expiresAt - Date.now() > 0`: token is valid, show minutes remaining.
- If expired: inform the user the hook will auto-refresh before each command.

### Step 3 — Confirm readiness

Tell the user they are now ready to use all RINDA commands:
- `/rinda-ai:buyer-search` — discover leads
- `/rinda-ai:enrich` — enrich contacts
- `/rinda-ai:sequence-create` — create email sequences
- `/rinda-ai:reply-check` — check email replies
- `/rinda-ai:campaign-report` — view campaign reports

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| Port 9876 already in use | Another process is using the port | Kill the conflicting process or restart the terminal |
| Browser did not open | Headless environment | Copy the URL printed in the terminal and open it manually |
| `~/.rinda/credentials.json` not found after login | Login was interrupted | Run `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth login` again |
| Login timed out | User took too long to authenticate | Run the login command again |

## Output Format

Display a clear confirmation message after verifying credentials:

```
Connected to RINDA AI
  Account: kim@company.com
  Workspace: <workspaceId>

You're ready to start finding buyers. Try:
  /rinda-ai:buyer-search
```

## Related Skills

None. This command handles authentication only via the CLI binary — no API calls are made by Claude.

## Notes

- Tokens are auto-refreshed before every tool call by the plugin hook (`rinda-cli auth ensure-valid`). You do not need to re-run this command unless your session has been inactive for more than 14 days.
- To check your current status at any time, run: `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth status`
- To log out and delete local credentials: `${CLAUDE_PLUGIN_ROOT}/bin/rinda-cli auth logout`
