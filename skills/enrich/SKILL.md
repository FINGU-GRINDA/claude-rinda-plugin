---
description: Enrich leads with contact details, social profiles, and product information.
---

# Enrich Leads Skill

Enrich a list of leads with contact details, social media profiles, and product information. Use after buyer-search to find decision-maker emails and LinkedIn profiles.

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `lead_ids` | string[] | Yes | — | Lead IDs to enrich. Obtain from a prior buyer search. If omitted, uses lead IDs from the most recent search in the conversation. |
| `batch_size` | number | No | `10` | Leads per batch (max 25). |

Maximum 100 leads per enrichment run.

## Implementation

This skill delegates to the **rinda-agent** sub-agent, which executes the **Workflow 2 — Contact Enrichment** workflow. See `agents/rinda-agent.md` for full implementation details including batch processing logic, progress reporting, buyer qualification scoring, and workflow chaining.

## Related Skills

- **buyer-qualification** — Used for contact scoring, decision-maker identification, and priority classification.
