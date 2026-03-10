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

All flags optional. If 422 error, fall back to `rinda-cli order history`.

### Buyer Enrich

```bash
rinda-cli buyer enrich --buyer-id "lead_abc123"
```

One lead at a time. Loop with 1s delay for batch.

### Reply Check

```bash
rinda-cli reply check --limit 50
```

### Campaign Stats

```bash
rinda-cli campaign stats --period "30d"
```

Accepts: `7d`, `30d`, `90d`, `2w`, `3m`, or bare number.

### Sequence Create

```bash
rinda-cli sequence create --name "Campaign Name"
```

Optional: `--type "email"`, `--steps '[{"delay":1,"template":"intro"}]'`

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

### 1. Buyer Search

1. `rinda-cli auth ensure-valid`
2. `rinda-cli buyer search --industry "X" --countries "Y" --limit 50`
3. Score leads using **buyer-qualification** rules (see references)
4. Present ranked table, offer: enrich? create sequence?

### 2. Contact Enrichment

1. `rinda-cli auth ensure-valid`
2. For each lead: `rinda-cli buyer enrich --buyer-id "ID"` (1s delay)
3. Classify: High/Medium/Low priority per **buyer-qualification** rules
4. Present grouped results, offer: create sequence?

### 3. Email Sequence

1. `rinda-cli auth ensure-valid`
2. `rinda-cli sequence create --name "Name"`
3. For each lead: `rinda-cli sequence add-contact --sequence-id "ID" --buyer-id "ID"`
4. Present summary, suggest: check replies later

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

`buyer-search → enrich → sequence-create → reply-check → campaign-report`

Carry forward IDs between steps. Always ask before proceeding to next.

## Error Handling

| Condition | Action |
|-----------|--------|
| Auth failure | "Run `/rinda-ai:connect`" |
| 422 on buyer search | Fall back to `rinda-cli order history` |
| 429 rate limit | Wait 30s, retry once |
| 5xx | Wait 3s, retry once |
| Empty results | Suggest broadening criteria |
