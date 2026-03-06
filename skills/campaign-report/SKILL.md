---
description: Generate a campaign performance report with funnel metrics, email stats, and insights.
---

# Campaign Report Skill

Generate a campaign performance report with funnel metrics, email statistics, hot leads, and actionable insights. Optionally drill down into a specific sequence.

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `period` | `"7d"` \| `"30d"` \| `"90d"` | No | `"7d"` | Reporting period. Use `"7d"` for weekly, `"30d"` for monthly, `"90d"` for quarterly. |
| `sequence_id` | string | No | — | Sequence ID for per-sequence metrics drill-down. If omitted, shows workspace-wide stats only. |

## Implementation

This skill delegates to the **rinda-agent** sub-agent, which executes the **Workflow 5 — Campaign Reporting** workflow. See `agents/rinda-agent.md` for full implementation details including dashboard data fetching, rate calculations, benchmark comparisons, and insight generation.

## Related Skills

None. This skill is purely analytical and uses built-in benchmark thresholds defined in `agents/rinda-agent.md` for insights.
