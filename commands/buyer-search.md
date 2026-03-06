# /rinda-ai:buyer-search

Search for B2B buyers matching your export target profile. Runs a lead discovery search, polls for results, scores and ranks leads, then offers to create an email sequence.

## Parameters

- `industry` (string, **required**) — Industry or product category to search for. Examples: "cosmetics", "industrial machinery", "organic food".
- `country` (string, **required**) — Target country or region. Examples: "US", "Germany", "Southeast Asia".
- `buyer_type` (string[], optional, default: `["importer", "distributor", "wholesaler"]` from `settings.json`) — Types of buyers to target.
- `min_size` (number, optional, default: `1000000` from `settings.json` `targetMarket.minRevenue`) — Minimum annual revenue in USD.
- `quantity` (number, optional, default: `50` from `settings.json` `search.defaultQuantity`) — Number of results to fetch.

Before calling the API, confirm parameters with the user if `industry` or `country` are missing.

## Credentials

Read `~/.rinda/credentials.json`:

```json
{
  "accessToken": "eyJ...",
  "workspaceId": "uuid"
}
```

Use `Authorization: Bearer ${accessToken}` on every request.

If the file is missing or `accessToken` is absent, tell the user: "Please run `/rinda-ai:connect` first to authenticate."

## Workflow

Base URL: `https://app.rinda.ai/api/v1`

### Step 1 — Read credentials

Read `~/.rinda/credentials.json` and extract `accessToken` and `workspaceId`.

### Step 2 — Start the search

```
POST https://app.rinda.ai/api/v1/lead-discovery/search
Authorization: Bearer ${accessToken}
Content-Type: application/json

{
  "query": "${industry} ${buyer_type.join(' OR ')} in ${country}",
  "targetCountries": ["${country}"],
  "buyerTypes": ${buyer_type},
  "minRevenue": ${min_size},
  "quantity": ${quantity},
  "workspaceId": "${workspaceId}"
}
```

Expected response:

```json
{ "sessionId": "sess_abc123" }
```

Save `sessionId` for polling.

### Step 3 — Poll until complete

```
GET https://app.rinda.ai/api/v1/lead-discovery/session/${sessionId}
Authorization: Bearer ${accessToken}
```

Poll every 3 seconds. Expected response shape:

```json
{
  "status": "processing" | "completed" | "failed",
  "progress": 65
}
```

Show the user a progress indicator while waiting: "Searching for ${industry} buyers in ${country}... (${progress}%)"

Stop polling when `status === "completed"` or `status === "failed"`.

If `status === "failed"`, report the error and stop.

### Step 4 — Fetch results

```
GET https://app.rinda.ai/api/v1/lead-discovery/db/sessions/${sessionId}/results
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "leads": [
    {
      "id": "lead_xyz",
      "companyName": "Acme Importers LLC",
      "country": "US",
      "industry": "cosmetics",
      "estimatedRevenue": 5000000,
      "employeeCount": 45,
      "buyerType": "importer",
      "relevanceScore": 0.92,
      "website": "https://acme.com",
      "description": "..."
    }
  ]
}
```

### Step 5 — Score and rank leads

Apply the **export-sales** skill rules to rank the leads:

1. Exclude any leads already in the contacted list (if available from context).
2. Score each lead using the weighted formula from the export-sales skill:
   - Revenue match vs `min_size`
   - Buyer type relevance
   - Relevance score from API
3. Sort descending by score.
4. Take the top results up to `quantity`.

### Step 6 — Present results

Display a ranked table:

```
Found ${leads.length} ${industry} buyers in ${country}

Rank | Company                | Country | Type        | Revenue   | Score
-----|------------------------|---------|-------------|-----------|------
  1  | Acme Importers LLC     | US      | importer    | $5M       | 0.92
  2  | Global Beauty Dist.    | US      | distributor | $12M      | 0.88
  ...
```

### Step 7 — Offer next steps

Ask the user:

```
Would you like to:
1. Enrich contacts for these leads? (/rinda-ai:enrich)
2. Create an email sequence for these leads? (/rinda-ai:sequence-create)
3. Export this list?
```

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| 401 Unauthorized | Token expired | Tell user: "Session expired. Please run `/rinda-ai:connect` to re-authenticate." |
| 429 Too Many Requests | Rate limit hit | Wait 30 seconds, then retry the same request once. If still failing, ask user to try again in a few minutes. |
| `status === "failed"` on poll | Search failed server-side | Report the error message and suggest rephrasing the query. |
| Empty `leads` array | No matching companies found | Suggest broadening the search: try a wider `country`, different `buyer_type`, or lower `min_size`. |
| Network error | Connectivity issue | Inform the user and ask them to retry. |

## Output Format

Present results as a markdown table sorted by score. After the table, include a brief summary:

```
Summary: ${leads.length} leads found. Top lead is ${topLead.companyName} (${topLead.country}, score: ${topLead.score}).
```

## Related Skills

- **export-sales** — Used in Step 5 for lead scoring rules, buyer type weighting, and exclusion logic.
