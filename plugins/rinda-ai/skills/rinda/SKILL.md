---
name: rinda
description: RINDA AI B2B export sales automation. Handles buyer search, contact enrichment, email sequences, reply management, and campaign reporting. Use when the user wants to work with RINDA sales workflows.
argument-hint: "[workflow or question]"
allowed-tools: Bash
---

# RINDA AI Skill

All API interactions go through the CLI. Never use raw curl/HTTP calls.

## CLI Binary

```bash
CLI="${CLAUDE_PLUGIN_ROOT}/../../bin/rinda-cli"
```

If missing: `${CLAUDE_PLUGIN_ROOT}/../../bin/install.sh`

All commands output JSON. Parse output to present results to the user.

---

## Auth

Always run before any workflow:

```bash
$CLI auth ensure-valid
```

If it fails → "Run `/rinda-ai:connect` to authenticate."

| Action | Command |
|--------|---------|
| Get login URL | `$CLI auth url` |
| Login with token | `$CLI auth token <TOKEN>` |
| Check status | `$CLI auth status` |
| Logout | `$CLI auth logout` |

---

## Commands

### Buyer Search

```bash
$CLI buyer search --industry "cosmetics" --countries "US,DE" --buyer-type "importer" --min-revenue 1000000 --limit 50
```

All flags optional. If 422 error, fall back to `$CLI order history`.

### Buyer Enrich

```bash
$CLI buyer enrich --buyer-id "lead_abc123"
```

One lead at a time. Loop with 1s delay for batch.

### Reply Check

```bash
$CLI reply check --limit 50
```

### Campaign Stats

```bash
$CLI campaign stats --period "30d"
```

Accepts: `7d`, `30d`, `90d`, `2w`, `3m`, or bare number.

### Sequence Create

```bash
$CLI sequence create --name "Campaign Name"
```

Optional: `--type "email"`, `--steps '[{"delay":1,"template":"intro"}]'`

### Sequence Add Contact

```bash
$CLI sequence add-contact --sequence-id "uuid" --buyer-id "lead_abc123"
```

### Email Send

```bash
$CLI email send --to "buyer@co.com" --subject "Subject" --body "Body"
```

### Order History (fallback search)

```bash
$CLI order history --buyer-id "search term" --days-inactive 30
```

---

## Workflows

### 1. Buyer Search

1. `$CLI auth ensure-valid`
2. `$CLI buyer search --industry "X" --countries "Y" --limit 50`
3. Score leads using **buyer-qualification** rules (see references)
4. Present ranked table, offer: enrich? create sequence?

### 2. Contact Enrichment

1. `$CLI auth ensure-valid`
2. For each lead: `$CLI buyer enrich --buyer-id "ID"` (1s delay)
3. Classify: High/Medium/Low priority per **buyer-qualification** rules
4. Present grouped results, offer: create sequence?

### 3. Email Sequence

1. `$CLI auth ensure-valid`
2. `$CLI sequence create --name "Name"`
3. For each lead: `$CLI sequence add-contact --sequence-id "ID" --buyer-id "ID"`
4. Present summary, suggest: check replies later

### 4. Reply Management

1. `$CLI auth ensure-valid`
2. `$CLI reply check --limit 50`
3. Classify per **export-sales** rules: INTERESTED → CURIOUS → NOT_NOW → REJECTED
4. Present prioritized action list

### 5. Campaign Report

1. `$CLI auth ensure-valid`
2. `$CLI campaign stats --period "30d"`
3. Calculate rates, compare to benchmarks (open >35% good, reply >10% good)
4. Generate insights

## Chaining

`buyer-search → enrich → sequence-create → reply-check → campaign-report`

Carry forward IDs between steps. Always ask before proceeding to next.

## Error Handling

| Condition | Action |
|-----------|--------|
| Auth failure | "Run `/rinda-ai:connect`" |
| 422 on buyer search | Fall back to `$CLI order history` |
| 429 rate limit | Wait 30s, retry once |
| 5xx | Wait 3s, retry once |
| Empty results | Suggest broadening criteria |
