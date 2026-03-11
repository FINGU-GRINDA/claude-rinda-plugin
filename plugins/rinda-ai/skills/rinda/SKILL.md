---
name: rinda
description: RINDA AI B2B export sales automation. Handles buyer search, contact enrichment, email sequences, reply management, and campaign reporting. Use when the user wants to work with RINDA sales workflows.
argument-hint: "[workflow or question]"
allowed-tools: Bash
---

# RINDA AI Skill

All API interactions go through the CLI. Never use raw curl/HTTP calls.

## CLI Binary

The CLI installs automatically on session start. If missing, run:

```bash
bash ${CLAUDE_PLUGIN_ROOT}/skills/rinda/scripts/install.sh
```

Then use it. Prepend to PATH at the start of every bash block:

```bash
export PATH="$HOME/.rinda/bin:$PATH"
```

After that, just call `rinda-cli` directly. All commands output JSON.

---

## MCP Server

An MCP server (`rinda-mcp`) is registered in `plugin.json` and auto-discovered by Claude Code on session start. It exposes the same capabilities as the CLI as structured MCP tools, which Claude can call directly without shell commands.

The following MCP tools are available:

| Tool | Description |
|------|-------------|
| `rinda_auth_status` | Return current authentication status |
| `rinda_auth_login` | Return browser login URL and instructions |
| `rinda_workspace_list` | List workspaces the authenticated user belongs to |
| `rinda_buyer_search` | Start an async buyer search, returns sessionId |
| `rinda_buyer_status` | Poll status of an async search session |
| `rinda_buyer_results` | Get results of a completed search session |
| `rinda_buyer_select` | Save selected leads from a discovery session |
| `rinda_buyer_enrich` | Enrich a buyer/lead with contact and company data |
| `rinda_buyer_clarify` | Submit answers to clarification questions |
| `rinda_campaign_stats` | Get campaign dashboard statistics |
| `rinda_email_send` | Send an email via RINDA |
| `rinda_reply_check` | Get recent email replies |
| `rinda_sequence_create` | Create a new email sequence |
| `rinda_sequence_list` | List existing email sequences |
| `rinda_sequence_generate` | AI-generate email steps for a sequence |
| `rinda_sequence_add_contact` | Enroll a lead into an email sequence |
| `rinda_workspace_list` | List workspaces for the authenticated user |

The MCP server binary (`rinda-mcp`) is installed to `~/.rinda/bin/rinda-mcp` by the session-start install hook alongside `rinda-cli`. Auth is handled via `~/.rinda/credentials.json` — no additional configuration needed.

---

## Auth

Always run before any workflow:

```bash
rinda-cli auth ensure-valid
```

If it fails → "Run `/rinda-ai:connect` to authenticate."

| Action | Command |
|--------|---------|
| Get login URL | `rinda-cli auth url` |
| Login with token | `rinda-cli auth token <TOKEN>` |
| Check status | `rinda-cli auth status` |
| Logout | `rinda-cli auth logout` |

---

## Commands

### Buyer Search

```bash
rinda-cli buyer search --industry "cosmetics" --countries "US,DE" --buyer-type "importer" --min-revenue 1000000 --limit 50
```

All flags optional. Returns a `sessionId` — the search runs async. If 422 error, fall back to `rinda-cli order history`.

### Buyer Status

```bash
rinda-cli buyer status --session-id "uuid"
```

Check progress of an async search. Shows status, progress %, result count. Poll until `status: complete`.

### Buyer Results

```bash
rinda-cli buyer results --session-id "uuid"
```

View discovered leads from a completed search session. Returns JSON array of companies with names, countries, business types.

### Buyer Select

```bash
rinda-cli buyer select --session-id "uuid" --recommendation-id "rec_id"
```

Save selected leads from discovery results into the workspace.

### Buyer Messages

```bash
rinda-cli buyer messages --session-id "uuid"
```

Retrieve clarification questions for a session in `waiting_clarification` status.

### Buyer Clarify

```bash
rinda-cli buyer clarify --session-id "uuid" --answers '{"field": "value"}'
```

Submit answers to clarification questions and resume the search session.

### Buyer Enrich

```bash
rinda-cli buyer enrich --buyer-id "https://company-website.com"
```

Enrich a lead with additional data. Takes a website URL. One lead at a time. Loop with 1s delay for batch.

### Reply Check

```bash
rinda-cli reply check --limit 50
```

### Campaign Stats

```bash
rinda-cli campaign stats --period "30d"
```

Accepts: `7d`, `30d`, `90d`, `2w`, `3m`, or bare number.

### Sequence List

```bash
rinda-cli sequence list --limit 10
```

List existing sequences in the workspace. Optional: `--offset` for pagination.

### Sequence Create

```bash
rinda-cli sequence create --name "Campaign Name"
```

Optional: `--type "email"`, `--steps '[{"delay":1,"template":"intro"}]'`

### Sequence Generate

```bash
rinda-cli sequence generate --id "uuid"
```

AI-generate email steps for an existing sequence.

### Sequence Add Contact

```bash
rinda-cli sequence add-contact --sequence-id "uuid" --buyer-id "lead_abc123"
```

### Email Send

```bash
rinda-cli email send --to "buyer@co.com" --subject "Subject" --body "Body"
```

### Order History (fallback search)

```bash
rinda-cli order history --buyer-id "search term" --days-inactive 30
```

---

## Workflows

### 1. Buyer Search (end-to-end)

1. `rinda-cli auth ensure-valid`
2. `rinda-cli buyer search --industry "X" --countries "Y" --limit 50` → get `sessionId`
3. Poll: `rinda-cli buyer status --session-id "ID"`
   - If `waiting_clarification` → `rinda-cli buyer messages --session-id "ID"` → present questions to user → `rinda-cli buyer clarify --session-id "ID" --answers '{...}'` → resume polling
   - If `complete` → proceed to results
4. `rinda-cli buyer results --session-id "ID"` → view discovered leads
5. Score leads using **buyer-qualification** rules (see references)
6. Present ranked table, offer: select? enrich? create sequence?

### 2. Contact Enrichment

1. `rinda-cli auth ensure-valid`
2. For each lead: `rinda-cli buyer enrich --buyer-id "https://website.com"` (1s delay)
3. Classify: High/Medium/Low priority per **buyer-qualification** rules
4. Present grouped results, offer: create sequence?

### 3. Email Sequence

1. `rinda-cli auth ensure-valid`
2. `rinda-cli sequence list` → check existing sequences
3. `rinda-cli sequence create --name "Name"` → get sequence ID
4. `rinda-cli sequence generate --id "ID"` → AI-generate email steps
5. For each lead: `rinda-cli sequence add-contact --sequence-id "ID" --buyer-id "ID"`
6. Present summary, suggest: check replies later

### 4. Reply Management

1. `rinda-cli auth ensure-valid`
2. `rinda-cli reply check --limit 50`
3. Classify per **export-sales** rules: INTERESTED → CURIOUS → NOT_NOW → REJECTED
4. Present prioritized action list

### 5. Campaign Report

1. `rinda-cli auth ensure-valid`
2. `rinda-cli campaign stats --period "30d"`
3. Calculate rates, compare to benchmarks (open >35% good, reply >10% good)
4. Generate insights

## Chaining

`buyer-search → buyer-status → buyer-results → enrich → sequence-create → sequence-generate → add-contacts → reply-check → campaign-report`

Carry forward IDs between steps. Always ask before proceeding to next.

## Error Handling

| Condition | Action |
|-----------|--------|
| Auth failure | "Run `/rinda-ai:connect`" |
| 422 on buyer search | Fall back to `rinda-cli order history` |
| 429 rate limit | Wait 30s, retry once |
| 5xx | Wait 3s, retry once |
| Empty results | Suggest broadening criteria |
| waiting_clarification | Fetch messages, present to user, submit answers with `buyer clarify` |
