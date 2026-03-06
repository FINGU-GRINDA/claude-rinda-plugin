---
description: Enrich a list of leads with contact details, social media profiles, and product information. Use after buyer-search to find decision-maker emails and LinkedIn profiles.
---

# Enrich Leads Skill

Enrich a list of leads with contact details, social media profiles, and product information. Use after buyer-search to find decision-maker emails and LinkedIn profiles.

## Parameters

- `lead_ids` (string[], **required**) — Array of lead IDs to enrich. Obtain these from a prior buyer-search result. Example: `["lead_abc123", "lead_def456"]`.

If the user says "enrich these leads" without specifying IDs, use the lead IDs from the most recent buyer-search results in the conversation.

Maximum recommended batch: 50 leads per call.

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

### Step 2 — Submit enrichment request

```
POST https://app.rinda.ai/api/v1/contact-enrichment/enrich-leads
Authorization: Bearer ${accessToken}
Content-Type: application/json

{
  "leadIds": ["lead_abc123", "lead_def456"]
}
```

Expected response:

```json
{
  "results": [
    {
      "leadId": "lead_abc123",
      "status": "enriched" | "not_found" | "failed",
      "contactsFound": 3
    }
  ]
}
```

Note the enrichment status for each lead. Leads with `status === "enriched"` have contact data available for retrieval.

### Step 3 — Fetch detailed data for each enriched lead

For each lead where `status === "enriched"`, fetch its full details:

```
GET https://app.rinda.ai/api/v1/leads/${leadId}/details
Authorization: Bearer ${accessToken}
```

Expected response shape:

```json
{
  "id": "lead_abc123",
  "companyName": "Acme Importers LLC",
  "contacts": [
    {
      "name": "Jane Smith",
      "title": "Purchasing Manager",
      "email": "jane@acme.com",
      "linkedin": "https://linkedin.com/in/janesmith",
      "phone": "+1-555-0100"
    }
  ],
  "socialMedia": {
    "linkedin": "https://linkedin.com/company/acme",
    "website": "https://acme.com"
  },
  "products": ["skincare", "haircare"],
  "annualRevenue": 5000000,
  "employeeCount": 45
}
```

Make these requests sequentially or in small parallel batches to avoid rate limits.

### Step 4 — Apply buyer qualification scoring

Using the **buyer-qualification** skill, evaluate each enriched lead:

1. Does the company have a purchasing decision-maker contact?
2. Does the product range match your export offer?
3. Is the revenue and employee count above your minimum thresholds?

Classify each lead:
- **High priority**: decision-maker email found, product match strong
- **Medium priority**: email found but title is unclear, or product match partial
- **Low priority**: only generic email found (info@, contact@), or no product match

### Step 5 — Present enriched results

Display a summary for each lead:

```
Enrichment Results (${enriched.length} of ${lead_ids.length} leads enriched)
===========================================================================

Acme Importers LLC [HIGH PRIORITY]
  Contact: Jane Smith, Purchasing Manager
  Email:   jane@acme.com
  LinkedIn: linkedin.com/in/janesmith
  Products: skincare, haircare
  Revenue: $5M | Employees: 45

Global Beauty Dist. [MEDIUM PRIORITY]
  Contact: info@globalbeauty.com (generic)
  LinkedIn: linkedin.com/company/globalbeauty
  Products: cosmetics
  Revenue: $12M | Employees: 120
  Note: No direct decision-maker contact found

Not enriched (${notFound.length}): <list company names>
```

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| 401 Unauthorized | Token expired | Tell user: "Session expired. Please run `/rinda-ai:connect` to re-authenticate." |
| 429 Too Many Requests | Rate limit hit | Wait 30 seconds, then retry. For large batches, split into groups of 10. |
| Empty `results` array | No leads processed | Verify the lead IDs are valid (from a recent buyer-search). |
| All leads `status === "not_found"` | Leads not in enrichment database | Inform the user that enrichment data is unavailable for these companies. Suggest trying a new search. |
| Network error on detail fetch | Intermittent failure | Retry the individual failed `GET /leads/:id/details` request once before reporting failure. |

## Output Format

Present results grouped by priority (High, Medium, Low). For each lead show: company name, best contact found, their title and email, LinkedIn if available, and any product alignment notes.

End with a summary line:

```
Enrichment complete: ${highPriority.length} high-priority, ${medPriority.length} medium, ${lowPriority.length} low-priority leads.
Ready to create a sequence? Try /rinda-ai:sequence-create
```

## Related Skills

- **buyer-qualification** — Used in Step 4 for contact scoring, decision-maker identification, and priority classification.
