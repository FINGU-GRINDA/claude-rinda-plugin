# RINDA Cowork Plugin Architecture

> Claude Cowork plugin for RINDA AI. Uses a lightweight CLI for OAuth/token management. Claude calls the REST API directly — no MCP server needed.

---

## API Reference

- **OpenAPI JSON spec**: https://alpha.rinda.ai/openapi/json
- **Interactive docs (Scalar UI)**: https://alpha.rinda.ai/openapi

---

## High-Level Overview

```
+-------------------+     hook      +----------------+
|                   |   (pre-call)  |                |
|  Claude Cowork    |-------------->| rinda-cli      |
|  + RINDA Plugin   |               | auth ensure-   |
|  (commands/,      |               | valid          |
|   skills/)        |               +-------+--------+
|                   |                       |
|                   |                reads/writes
|                   |                       v
|                   |               ~/.rinda/
|                   |               credentials.json
|                   |                       ^
|                   |                reads  |
|                   |-----------------------+
|                   |
|                   |    REST (with valid accessToken)
|                   |-------------------------------------->  Elysia Backend
|                   |              HTTPS                     api.rinda.ai
+-------------------+
```

### What Each Piece Does

```
+------------------------------------------------------------------+
|  Component          | Responsibility                              |
|------------------------------------------------------------------+
|  rinda-cli          | OAuth login, token refresh, credential      |
|  (~100 lines Bun)   | storage. Claude never sees auth logic.      |
|                     |                                             |
|  Plugin commands/   | Markdown files that teach Claude HOW to     |
|  (5 .md files)      | call REST endpoints, what params to use,    |
|                     | and how to interpret responses.             |
|                     |                                             |
|  Plugin skills/     | Domain knowledge (email rules, buyer        |
|  (3 .md files)      | qualification, follow-up logic). Claude     |
|                     | references these contextually.              |
|                     |                                             |
|  Hook               | Pre-tool-call hook runs                     |
|  (.claude/hooks)    | `rinda-cli auth ensure-valid` to keep       |
|                     | tokens fresh. Invisible to Claude.          |
|                     |                                             |
|  Elysia Backend     | Existing REST API. No changes needed.       |
|  (api.rinda.ai)     | Same endpoints the dashboard uses.          |
+------------------------------------------------------------------+
```

---

## Directory Structure

```
rinda-ai/
├── .claude-plugin/
│   ├── plugin.json               # Plugin manifest (name, version, description)
│   └── marketplace.json          # Marketplace catalog (for distribution)
├── hooks/
│   └── hooks.json                # Pre-call hook: auto-refresh tokens
├── commands/
│   ├── connect.md                # /rinda-ai:connect — run OAuth login
│   ├── buyer-search.md           # /rinda-ai:buyer-search
│   ├── enrich.md                 # /rinda-ai:enrich
│   ├── sequence-create.md        # /rinda-ai:sequence-create
│   ├── reply-check.md            # /rinda-ai:reply-check
│   └── campaign-report.md        # /rinda-ai:campaign-report
├── skills/
│   ├── export-sales/SKILL.md     # Buyer search rules, follow-up logic
│   ├── email-writing/SKILL.md    # Email personalization, subject lines
│   └── buyer-qualification/SKILL.md  # Scoring, classification
├── agents/
│   └── enrichment-agent.md       # Sub-agent for parallel enrichment
├── .mcp.json                     # MCP server config (optional, for connectors)
├── settings.json                 # Default plugin settings
├── crates/
│   ├── sdk/                      # rinda-sdk: progenitor-generated from doc/openapi-patched.json
│   │   ├── Cargo.toml
│   │   ├── build.rs              # Code generation via progenitor at build time
│   │   └── src/lib.rs
│   └── cli/                      # rinda-cli: CLI binary (name: "rinda")
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # CLI entry (clap subcommands)
│           ├── config.rs         # Paths and BASE_URL (inlined from former rinda-common)
│           ├── error.rs          # RindaError enum (inlined from former rinda-common)
│           ├── credentials.rs    # Read/write ~/.rinda/credentials.json
│           ├── oauth.rs          # Google OAuth flow (localhost callback)
│           └── commands/
│               ├── auth.rs       # auth login/logout/status/ensure-valid
│               └── config.rs     # config show
└── doc/
    ├── openapi.json              # Raw OpenAPI spec (synced from upstream)
    └── openapi-patched.json      # Patched spec used for SDK generation
```

> **Note**: Commands, agents, skills, and hooks go at the **plugin root**, NOT inside `.claude-plugin/`. Only `plugin.json` and `marketplace.json` belong in `.claude-plugin/`.

### Plugin Manifest

```json
// .claude-plugin/plugin.json
{
  "name": "rinda-ai",
  "description": "RINDA AI - B2B export sales automation. Buyer search, enrichment, email sequences, reply management.",
  "version": "1.0.0",
  "author": {
    "name": "GRINDA AI",
    "email": "support@grinda.ai"
  },
  "homepage": "https://rinda.ai",
  "repository": "https://github.com/grinda-ai/rinda-cowork-plugin",
  "license": "MIT",
  "keywords": ["sales", "export", "b2b", "outreach", "email"]
}
```

---

## Distribution

### Distribution Options

```
+------------------------------------------------------------------+
| Method           | How                      | Best for            |
|------------------------------------------------------------------|
| ZIP upload       | Upload .zip via Cowork   | Quick iteration,    |
|                  | admin UI (<50MB)         | internal testing    |
|                  |                          |                     |
| GitHub sync      | Connect private repo,    | Team collaboration, |
|                  | Cowork auto-syncs        | version control     |
|                  |                          |                     |
| npm package      | `npm install` source     | Public distribution,|
|                  | in marketplace.json      | semantic versioning |
|                  |                          |                     |
| Marketplace      | Users run /plugin        | Community sharing,  |
|                  | marketplace add org/repo | discoverability     |
|                  |                          |                     |
| Official submit  | Submit at claude.ai/     | Maximum reach       |
|                  | settings/plugins/submit  |                     |
+------------------------------------------------------------------+
```

### Marketplace File

```json
// .claude-plugin/marketplace.json
{
  "name": "rinda-tools",
  "owner": {
    "name": "GRINDA AI",
    "email": "support@grinda.ai"
  },
  "metadata": {
    "description": "B2B export sales automation plugins"
  },
  "plugins": [
    {
      "name": "rinda-ai",
      "source": "./",
      "description": "Buyer search, enrichment, email sequences, reply management",
      "version": "1.0.0",
      "category": "sales",
      "tags": ["b2b", "export", "outreach"]
    }
  ]
}
```

### Installation Flow

```
Option A: From marketplace (recommended)
=========================================
/plugin marketplace add grinda-ai/rinda-cowork-plugin
/plugin install rinda-ai@rinda-tools

Option B: From GitHub directly
==============================
/plugin marketplace add https://github.com/grinda-ai/rinda-cowork-plugin.git

Option C: Local testing
=======================
claude --plugin-dir ./rinda-ai

Option D: Team auto-install (via project .claude/settings.json)
===============================================================
{
  "extraKnownMarketplaces": {
    "rinda-tools": {
      "source": {
        "source": "github",
        "repo": "grinda-ai/rinda-cowork-plugin"
      }
    }
  },
  "enabledPlugins": {
    "rinda-ai@rinda-tools": true
  }
}
```

---

## Authentication: CLI-Managed OAuth

### One-Time Login

```
$ npx rinda-cli auth login

  1. CLI starts local HTTP server on :9876
  2. Opens browser -> https://api.rinda.ai/api/v1/auth/google
  3. User signs in with Google
  4. Google redirects to localhost:9876/callback?code=xxx
  5. CLI exchanges code -> receives tokens from backend
  6. Writes to ~/.rinda/credentials.json
  7. "Logged in as kim@company.com"

~/.rinda/credentials.json
{
  "accessToken":  "eyJ...",           // 1hr TTL
  "refreshToken": "a8f3...",          // 14 day TTL
  "expiresAt":    1709726400000,      // ms timestamp
  "workspaceId":  "uuid",
  "userId":       "uuid",
  "email":        "kim@company.com"
}
```

### Auto Token Refresh (Hook)

Hooks live in `hooks/hooks.json` at the plugin root (not inside `.claude-plugin/`).
Use `${CLAUDE_PLUGIN_ROOT}` to reference files within the plugin's install directory.

```json
// hooks/hooks.json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/cli/src/index.ts ensure-valid"
          }
        ]
      }
    ]
  }
}
```

```
Every tool call:
================

User: /rinda:buyer-search cosmetics US

  1. Hook fires: `npx rinda-cli auth ensure-valid`
     |
     +-> Read ~/.rinda/credentials.json
     +-> expiresAt - now() > 5min?
     |     YES -> exit 0 (fast, <50ms)
     |     NO  -> POST /api/v1/auth/refresh { refreshToken }
     |            -> write new tokens to file
     |            -> exit 0
     |
  2. Claude reads ~/.rinda/credentials.json
     -> gets valid accessToken (always fresh)
     |
  3. Claude calls REST API directly:
     fetch("https://api.rinda.ai/api/v1/lead-discovery/search", {
       headers: { Authorization: `Bearer ${accessToken}` }
     })
     |
  4. Done. Claude never knew about OAuth.
```

### Token Lifecycle

```
Access Token (1hr)
==================
  0min          55min         60min
  |--- valid ---|-- refresh --|-- expired --|
                ^
                Hook refreshes here
                (5min buffer)

Refresh Token (14 days)
=======================
  Day 0                        Day 14
  |-------- valid --------------|-- expired --|
                                ^
                                Hook detects 401
                                -> CLI prints:
                                "Session expired. Run: npx rinda-cli auth login"
```

### Session Expiry

```
Scenario: User hasn't used plugin in 14+ days

User: /rinda:buyer-search cosmetics US

  1. Hook: `npx rinda-cli auth ensure-valid`
     -> accessToken expired
     -> POST /auth/refresh -> 401 (refresh token also expired)
     -> Hook exits with error code 1

  2. Claude receives hook error:
     "RINDA session expired. Please run: npx rinda-cli auth login"

  3. User runs login, gets fresh tokens, retries command.
```

---

## Command-to-Endpoint Mapping

Claude calls the REST API directly using `fetch`. Each command teaches Claude which endpoints to hit:

```
+------------------------------------------------------------------+
|  Command               | Backend Endpoints Called                 |
|------------------------------------------------------------------+
|                                                                   |
|  /rinda:connect        | CLI handles (no API call)                |
|                        |                                          |
|  /rinda:buyer-search   | POST /api/v1/lead-discovery/search       |
|                        | GET  /api/v1/lead-discovery/              |
|                        |   session/:id/stream (SSE poll)          |
|                        | GET  /api/v1/lead-discovery/              |
|                        |   db/sessions/:id/results                |
|                        |                                          |
|  /rinda:enrich         | POST /api/v1/contact-enrichment/          |
|                        |   enrich-leads                           |
|                        | GET  /api/v1/leads/:id/details            |
|                        |                                          |
|  /rinda:sequence-      | POST /api/v1/sequences                   |
|  create                | POST /api/v1/sequences/:id/steps          |
|                        | POST /api/v1/sequences/:id/generate       |
|                        | POST /api/v1/admin/sequences/:id/         |
|                        |   enrollments/bulk-with-scheduling        |
|                        |                                          |
|  /rinda:reply-check    | GET  /api/v1/email-replies                |
|                        | GET  /api/v1/email-replies/stats/         |
|                        |   by-intent                              |
|                        |                                          |
|  /rinda:campaign-      | GET  /api/v1/sequences/stats/overall      |
|  report                | GET  /api/v1/dashboard/unified            |
|                        | GET  /api/v1/sequences/:id/metrics        |
+------------------------------------------------------------------+
```

---

## Detailed Command Flows

### `/rinda:buyer-search`

```
Parameters (confirmed with user):
  - industry, country, buyer_type, min_size, quantity (default 50)

Claude:                                              Elysia Backend
  |                                                        |
  | 1. Read ~/.rinda/credentials.json                      |
  |    (accessToken, workspaceId)                          |
  |                                                        |
  | 2. POST /api/v1/lead-discovery/search                  |
  |    { query, targetCountries, ... }                     |
  |------------------------------------------------------->|
  |                           sessionId                    |
  |<-------------------------------------------------------|
  |                                                        |
  | 3. Poll GET /api/v1/lead-discovery/session/:id         |
  |    until status = "completed"                          |
  |------------------------------------------------------->|
  |                           { status, progress }         |
  |<-------------------------------------------------------|
  |                                                        |
  | 4. GET /api/v1/lead-discovery/db/sessions/:id/results  |
  |------------------------------------------------------->|
  |                           { leads: [...] }             |
  |<-------------------------------------------------------|
  |                                                        |
  | 5. Apply export-sales SKILL rules:                     |
  |    - Exclude previously contacted                      |
  |    - Score by revenue, type, relevance                 |
  |    - Sort and present top results                      |
  |                                                        |
  | 6. Ask user: "Create email sequence for these?"        |
```

### `/rinda:enrich`

```
Parameters:
  - lead_ids: string[] (from buyer search results)

Claude:                                              Elysia Backend
  |                                                        |
  | 1. POST /api/v1/contact-enrichment/enrich-leads        |
  |    { leadIds: [...] }                                  |
  |------------------------------------------------------->|
  |    (backend runs enrichment pipeline)                  |
  |                           { results: [...] }           |
  |<-------------------------------------------------------|
  |                                                        |
  | 2. For each enriched lead:                             |
  |    GET /api/v1/leads/:id/details                       |
  |------------------------------------------------------->|
  |    { contacts, socialMedia, products, ... }            |
  |<-------------------------------------------------------|
  |                                                        |
  | 3. Present enriched data to user                       |
  |    (contacts found, emails, LinkedIn, etc.)            |
```

### `/rinda:sequence-create`

```
Parameters:
  - name, target buyer group, email tone, step count

Claude:                                              Elysia Backend
  |                                                        |
  | 1. POST /api/v1/sequences                              |
  |    { name, workspaceId, customerGroupId }              |
  |------------------------------------------------------->|
  |                           { id, status: "draft" }      |
  |<-------------------------------------------------------|
  |                                                        |
  | 2. POST /api/v1/sequences/:id/generate                 |
  |    { model, tone, language, ... }                      |
  |------------------------------------------------------->|
  |    (AI generates 6 email steps)                        |
  |                           { steps: [...] }             |
  |<-------------------------------------------------------|
  |                                                        |
  | 3. Show generated steps to user for approval           |
  |                                                        |
  | 4. POST /api/v1/admin/sequences/:id/                   |
  |    enrollments/bulk-with-scheduling                    |
  |    { leadIds, userEmailAccountId }                     |
  |------------------------------------------------------->|
  |                           { enrolled: N }              |
  |<-------------------------------------------------------|
```

### `/rinda:reply-check`

```
Parameters:
  - hours (default: 2)

Claude:                                              Elysia Backend
  |                                                        |
  | 1. GET /api/v1/email-replies/stats/by-intent           |
  |    ?workspaceId=xxx                                    |
  |------------------------------------------------------->|
  |    { meeting_request: 2, positive_interest: 5, ... }   |
  |<-------------------------------------------------------|
  |                                                        |
  | 2. GET /api/v1/email-replies?limit=50&isRead=false     |
  |------------------------------------------------------->|
  |    { replies: [{ from, subject, intent, sentiment }] } |
  |<-------------------------------------------------------|
  |                                                        |
  | 3. Apply export-sales SKILL classification:            |
  |    INTERESTED -> "Respond immediately"                 |
  |    CURIOUS    -> "Send catalog + schedule follow-up"   |
  |    NOT_NOW    -> "Reactivate after 90 days"            |
  |    REJECTED   -> "Tag update only"                     |
  |                                                        |
  | 4. Present prioritized action list to user             |
```

### `/rinda:campaign-report`

```
Parameters:
  - period: "7d" | "30d" | "90d"
  - sequence_id (optional)

Claude:                                              Elysia Backend
  |                                                        |
  | 1. GET /api/v1/dashboard/unified                       |
  |    ?workspaceId=xxx                                    |
  |------------------------------------------------------->|
  |    { funnel, hotLeads, activity, subscription }        |
  |<-------------------------------------------------------|
  |                                                        |
  | 2. GET /api/v1/sequences/stats/overall                 |
  |    ?workspaceId=xxx                                    |
  |------------------------------------------------------->|
  |    { totalSent, opened, clicked, replied, bounced }    |
  |<-------------------------------------------------------|
  |                                                        |
  | 3. (if sequence_id)                                    |
  |    GET /api/v1/sequences/:id/metrics                   |
  |------------------------------------------------------->|
  |    { openRate, clickRate, replyRate, byStep: [...] }   |
  |<-------------------------------------------------------|
  |                                                        |
  | 4. Format report with insights                         |
```

---

## End-to-End Campaign Flow

```
                        Claude Cowork + RINDA Plugin
                        ============================

Step 1: Connect (one-time)
  $ npx rinda-cli auth login
  -> Google OAuth -> tokens saved to ~/.rinda/credentials.json

Step 2: Find Buyers
  /rinda:buyer-search
  -> POST lead-discovery/search -> poll -> get results
  -> "Found 50 cosmetics importers in US. Top 10 by score:"

Step 3: Enrich Top Picks
  (automatic or user-triggered)
  -> POST contact-enrichment/enrich-leads (parallel, up to 50)
  -> "Found email contacts for 8 of 10 buyers"

Step 4: Create Campaign
  /rinda:sequence-create
  -> POST sequences + generate AI steps
  -> "Created 'US Cosmetics Q1' with 6 email steps"

Step 5: Enroll & Send
  -> POST enrollments/bulk-with-scheduling
  -> "Enrolled 8 buyers. First emails scheduled for tomorrow 9am their timezone"

Step 6: Monitor Replies
  /rinda:reply-check
  -> GET email-replies + stats
  -> "3 new replies: 1 meeting request, 1 question, 1 not interested"
  -> Suggests actions per reply classification

Step 7: Report
  /rinda:campaign-report
  -> GET dashboard/unified + sequences/stats
  -> "Week 1: 45% open rate, 12% reply rate, 2 meetings booked"
```

---

## CLI Implementation

### `cli/src/index.ts` (Bun)

```
Commands:
  npx rinda-cli auth login          # Open browser, Google OAuth, save tokens
  npx rinda-cli auth logout         # Delete ~/.rinda/credentials.json
  npx rinda-cli auth status         # Show current user, token expiry
  npx rinda-cli auth ensure-valid   # Refresh if needed (called by hook)
```

### Core Logic (~100 lines)

```
auth login:
  1. Start HTTP server on localhost:9876
  2. Open browser to https://api.rinda.ai/api/v1/auth/google
     with redirect_uri=http://localhost:9876/callback
  3. Receive callback with auth code
  4. POST /api/v1/auth/google/callback { code }
     -> { token, refreshToken, user }
  5. Write credentials.json
  6. Stop server, print success

auth ensure-valid:
  1. Read credentials.json
  2. If (expiresAt - Date.now()) > 5 * 60 * 1000 -> exit 0
  3. POST /api/v1/auth/refresh { refreshToken }
     -> { token, refreshToken }
  4. Write updated credentials.json
  5. Exit 0
  (If refresh fails with 401 -> exit 1 with "Run: npx rinda-cli auth login")

auth status:
  1. Read credentials.json
  2. Print email, workspace, token expiry countdown
  3. Warn if refresh token expires within 3 days
```

---

## Security

```
+------------------------------------------------------------------+
|  Credential Storage                                              |
|  ~/.rinda/credentials.json                                       |
|  - File permissions: 600 (owner read/write only)                 |
|  - CLI sets permissions on write                                 |
|  - .gitignore'd by default                                       |
|                                                                  |
|  Token Lifecycle                                                 |
|  - Access token: 1hr (auto-refreshed by hook)                    |
|  - Refresh token: 14 days (user re-logins if expired)            |
|  - Tokens never enter Claude's context (read from file)          |
|                                                                  |
|  Transport                                                       |
|  - All API calls over HTTPS                                      |
|  - Same TLS as dashboard (api.rinda.ai)                          |
|                                                                  |
|  Revocation                                                      |
|  - User changes password -> refresh token invalidated            |
|  - Admin deactivates user -> next refresh fails                  |
|  - `npx rinda-cli auth logout` -> deletes local credentials      |
+------------------------------------------------------------------+
```

---

## Context Window Efficiency

```
What gets loaded into Claude's context:
=======================================

Always loaded:
  - Active command .md file           ~500 tokens
  - credentials.json (read once)      ~50 tokens
  - Referenced SKILL.md               ~300 tokens

NOT loaded (vs MCP approach):
  - No tool schemas (saves ~2,500 tokens)
  - No MCP protocol overhead
  - No unused tool definitions

Per API call overhead:
  - fetch() call + headers            ~100 tokens
  - Response parsing                  ~varies

Total per session:                    ~1,000 tokens
(vs ~4,000+ with MCP server)
```

---

## Comparison: Why CLI + Plugin Over MCP Server

```
                        MCP Server           CLI + Plugin
                        ──────────           ────────────
Infrastructure          ECS + ALB + Redis    $0 (runs locally)
Auth handling           Server-side Redis    CLI + local file
Context window          ~4,000 tok/session   ~1,000 tok/session
Token rotation          Server manages       Hook + CLI manages
Backend changes         None                 None
Maintenance             Server monitoring    ~100 lines of Bun
Deployment              Docker + AWS         npm publish
Time to build           3-4 weeks            1 week
Monthly cost            ~$30-50              $0
```

---

## Implementation Plan

```
Phase 1 (3 days)                     Phase 2 (3 days)
+--------------------------+         +--------------------------+
| CLI + Auth               |         | Commands                 |
| - rinda-cli auth login   |         | - buyer-search.md        |
| - rinda-cli auth refresh |         | - enrich.md              |
| - rinda-cli auth status  |         | - sequence-create.md     |
| - credentials.json R/W   |         | - reply-check.md         |
| - Hook setup             |         | - campaign-report.md     |
+--------------------------+         +--------------------------+
         |                                    |
         v                                    v
Phase 3 (2 days)                     Phase 4 (2 days)
+--------------------------+         +--------------------------+
| Skills + Plugin Meta     |         | Test + Publish           |
| - export-sales SKILL     |         | - End-to-end test        |
| - email-writing SKILL    |         | - Prompt optimization    |
| - buyer-qualification    |         | - npm publish rinda-cli  |
| - plugin.json            |         | - GitHub repo + README   |
| - enrichment sub-agent   |         | - User onboarding doc    |
+--------------------------+         +--------------------------+

Total: ~10 days. 1 developer.
```
