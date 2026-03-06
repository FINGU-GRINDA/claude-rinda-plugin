---
description: Search for B2B buyers matching your export target profile.
---

# Buyer Search Skill

Search for B2B buyers matching your export target profile. Runs a lead discovery search, polls for results, scores and ranks leads, then offers to enrich contacts or create an email sequence.

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `industry` | string | Yes | — | Industry or product category. Example: `"cosmetics"`, `"industrial machinery"`. |
| `country` | string | Yes | — | Target country or region. Example: `"US"`, `"Germany"`, `"Southeast Asia"`. |
| `buyer_type` | string[] | No | `["importer", "distributor", "wholesaler"]` | Types of buyers to target. |
| `min_size` | number | No | `1000000` | Minimum annual revenue in USD. |
| `quantity` | number | No | `50` | Number of results to fetch. |

## Implementation

This skill delegates to the **rinda-agent** sub-agent, which executes the **Workflow 1 — Buyer Search** workflow. See `agents/rinda-agent.md` for full implementation details including API endpoints, polling logic, lead scoring, and workflow chaining.

## Related Skills

- **export-sales** — Used for lead scoring rules, buyer type weighting, and exclusion logic.
