---
description: Check unread email replies, classify by intent, and get a prioritized action list.
---

# Reply Check Skill

Check unread email replies from leads, classify them by intent, and get a prioritized action list telling you exactly what to do with each reply.

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `hours` | number | No | `2` | How far back to look for replies, in hours. Use `24` for the last full day, `168` for the past week. |

## Implementation

This skill delegates to the **rinda-agent** sub-agent, which executes the **Workflow 4 — Reply Management** workflow. See `agents/rinda-agent.md` for full implementation details including intent summary fetching, reply classification, prioritized action list presentation, and workflow chaining.

## Related Skills

- **export-sales** — Used for intent-to-action classification, follow-up timing rules, and reactivation logic.
