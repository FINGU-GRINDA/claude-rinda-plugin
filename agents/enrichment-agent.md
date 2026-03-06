---
name: enrichment-agent
description: Parallel lead enrichment sub-agent that processes batches of lead IDs to fetch contact details (email, phone, LinkedIn) via the RINDA API. Invoked by /rinda:enrich or after /rinda:buyer-search.
---

# Enrichment Sub-Agent

> Parallel enrichment of multiple leads with progress reporting and failure handling.

## When to Use

This agent is invoked by the `/rinda:enrich` command or automatically after `/rinda:buyer-search` when the user agrees to enrich results. It processes a list of lead IDs in parallel batches, respecting API rate limits.

## Inputs

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `leadIds` | string[] | Yes | Lead IDs from buyer search results |
| `batchSize` | number | No | Leads per batch (default: 10, max: 25) |

## Authentication

1. Read `~/.rinda/credentials.json`
2. Extract `accessToken` and `workspaceId`
3. Use `Authorization: Bearer <accessToken>` for all API calls

## Base URL

```
https://beta.rinda.ai/api/v1
```

## Workflow

### Step 1: Validate Inputs

- Confirm `leadIds` is a non-empty array
- Cap `batchSize` at 25 to respect rate limits
- Split `leadIds` into batches of `batchSize`

### Step 2: Batch Enrichment

For each batch, call the enrichment endpoint:

```
POST /api/v1/contact-enrichment/enrich-leads
Content-Type: application/json
Authorization: Bearer <accessToken>

{
  "leadIds": ["id1", "id2", "id3", ...]
}
```

- Process batches sequentially (not all at once) to respect rate limits
- Wait 1 second between batches if processing more than 2 batches
- Report progress after each batch: "Enriched X/Y leads..."

### Step 3: Fetch Lead Details

For each successfully enriched lead, fetch full details:

```
GET /api/v1/leads/<leadId>/details
Authorization: Bearer <accessToken>
```

These detail requests CAN be made in parallel within a batch (use multiple fetch calls).

### Step 4: Consolidate Results

Collect all results into three categories:

- **Enriched**: Leads with contact data found (emails, phone, LinkedIn)
- **Partial**: Leads with some data but missing key fields (e.g., no email)
- **Failed**: Leads that returned errors during enrichment

## Error Handling

| Error | Action |
|-------|--------|
| Single lead fails in batch | Log the failure, continue with remaining leads |
| Entire batch fails (5xx) | Retry the batch once after 3 seconds. If still failing, skip and continue |
| 401 Unauthorized | Stop processing. Report: "Session expired. Run `/rinda:connect` to re-authenticate." |
| 429 Too Many Requests | Wait for `Retry-After` header value (or 10 seconds), then retry the batch |
| Network error | Retry once after 2 seconds. If still failing, skip batch and continue |

Never stop the entire enrichment run because of a single lead or batch failure. Always continue processing remaining batches.

## Progress Reporting

Report progress to the user at these checkpoints:

1. **Start**: "Starting enrichment of {total} leads in {batchCount} batches..."
2. **Per batch**: "Batch {n}/{total}: enriched {success} leads, {failed} failed"
3. **Completion**: Final summary (see output format below)

## Output Format

Present results as a structured summary:

```
## Enrichment Results

**{enriched}/{total} leads enriched successfully**
{partial} partial | {failed} failed

### Contacts Found

| Lead | Company | Email | Phone | LinkedIn |
|------|---------|-------|-------|----------|
| ...  | ...     | ...   | ...   | ...      |

### Partial Results (missing contact info)
- Lead Name (Company) — missing: email

### Failed
- Lead Name (Company) — error: <reason>
```

## Constraints

- Maximum 100 leads per enrichment run. If more are provided, ask the user to narrow the list.
- Do not retry a failed lead more than once.
- Do not call enrichment on leads that are already enriched (check lead details first if unsure).
- Keep the user informed of progress — never go silent during a long batch run.
