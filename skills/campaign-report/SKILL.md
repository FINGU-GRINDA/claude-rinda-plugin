---
description: Generate a campaign performance report with funnel metrics, email statistics, hot leads, and actionable insights. Optionally drill down into a specific sequence.
---

# Campaign Report Skill

Generate a campaign performance report with funnel metrics, email statistics, hot leads, and actionable insights. Optionally drill down into a specific sequence.

## Parameters

- `period` (`"7d"` | `"30d"` | `"90d"`, optional, default: `"7d"`) — Reporting period. Use `"7d"` for weekly, `"30d"` for monthly, `"90d"` for quarterly review.
- `sequence_id` (string, optional) — Sequence ID for per-sequence metrics drill-down. If omitted, only workspace-wide stats are shown.

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

### Step 2 — Fetch unified dashboard data

```
GET https://app.rinda.ai/api/v1/dashboard/unified?workspaceId=${workspaceId}
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "funnel": {
    "leads": 150,
    "enriched": 120,
    "contacted": 95,
    "replied": 23,
    "meetings": 4
  },
  "hotLeads": [
    {
      "leadId": "lead_abc123",
      "companyName": "Acme Importers LLC",
      "score": 0.95,
      "lastActivity": "2026-03-06T08:30:00Z",
      "intent": "meeting_request"
    }
  ],
  "activity": {
    "emailsSent": 95,
    "emailsOpened": 43,
    "linksClicked": 11,
    "repliesReceived": 23
  },
  "subscription": {
    "plan": "growth",
    "leadsUsed": 150,
    "leadsLimit": 500
  }
}
```

### Step 3 — Fetch overall sequence stats

```
GET https://app.rinda.ai/api/v1/sequences/stats/overall?workspaceId=${workspaceId}
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "totalSent": 95,
  "opened": 43,
  "clicked": 11,
  "replied": 23,
  "bounced": 3,
  "unsubscribed": 2
}
```

Calculate rates:
- Open rate = `opened / totalSent * 100`
- Click rate = `clicked / totalSent * 100`
- Reply rate = `replied / totalSent * 100`
- Bounce rate = `bounced / totalSent * 100`

### Step 4 — Fetch per-sequence metrics (if `sequence_id` provided)

```
GET https://app.rinda.ai/api/v1/sequences/${sequence_id}/metrics
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "sequenceId": "seq_abc123",
  "sequenceName": "US Cosmetics Q1 Outreach",
  "openRate": 0.45,
  "clickRate": 0.12,
  "replyRate": 0.24,
  "byStep": [
    {
      "stepNumber": 1,
      "sent": 50,
      "opened": 28,
      "clicked": 6,
      "replied": 8
    },
    {
      "stepNumber": 2,
      "sent": 45,
      "opened": 15,
      "clicked": 5,
      "replied": 15
    }
  ]
}
```

### Step 5 — Format report with insights

Compose the report with three sections: Overview, Email Performance, and Insights.

**Benchmarks for export sales** (use these to evaluate performance):
- Open rate: >35% is good, >50% is excellent
- Reply rate: >10% is good, >20% is excellent
- Click rate: >5% is good, >15% is excellent
- Bounce rate: <5% is healthy

Generate insights by comparing metrics to benchmarks and funnel shape:
- If reply rate > 20%: "Strong engagement — consider scaling outreach volume."
- If reply rate < 5%: "Low reply rate — review email copy with `/rinda-ai:sequence-create` and apply email-writing skill tips."
- If meetings / replied < 0.2: "Replies are not converting to meetings — review your meeting CTA wording."
- If bounce rate > 5%: "High bounce rate — clean your lead list or improve email verification during enrichment."
- If `hotLeads` array is non-empty: "You have ${hotLeads.length} hot lead(s) ready for personal outreach."

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| 401 Unauthorized | Token expired | Tell user: "Session expired. Please run `/rinda-ai:connect` to re-authenticate." |
| 429 Too Many Requests | Rate limit hit | Wait 30 seconds, then retry. |
| Dashboard endpoint returns empty funnel | No activity yet | Report "No campaign activity found for this workspace yet." Suggest starting with `/rinda-ai:buyer-search`. |
| `sequence_id` not found (404) | Invalid or deleted sequence | Inform the user the sequence was not found. List available sequences if possible (omit `sequence_id` and re-run for workspace totals). |
| One endpoint fails | Partial data | Report what data was successfully fetched. Display a partial report with a note about the failed section. |

## Output Format

```
RINDA Campaign Report — Last ${period}
=======================================
Workspace: ${workspaceId}

FUNNEL
  Leads discovered:  150
  Enriched:          120  (80%)
  Contacted:          95  (63%)
  Replied:            23  (24% reply rate)
  Meetings booked:     4  (17% meeting conversion)

EMAIL PERFORMANCE
  Sent:        95
  Opened:      43  (45.3% open rate)    [EXCELLENT]
  Clicked:     11  (11.6% click rate)   [GOOD]
  Replied:     23  (24.2% reply rate)   [EXCELLENT]
  Bounced:      3   (3.2% bounce rate)  [HEALTHY]
  Unsubscribed: 2

HOT LEADS (act now)
  1. Acme Importers LLC — meeting_request (score: 0.95)
  2. ...

[IF sequence_id provided:]
SEQUENCE BREAKDOWN: US Cosmetics Q1 Outreach
  Step 1: 50 sent | 56% open | 16% click | 16% reply
  Step 2: 45 sent | 33% open | 11% click | 33% reply
  ...

INSIGHTS & RECOMMENDATIONS
  - Strong engagement — consider scaling outreach volume.
  - ${hotLeads.length} hot lead(s) ready for personal outreach — check /rinda-ai:reply-check.
  - Plan usage: 150 / 500 leads used (30%).

SUBSCRIPTION
  Plan: growth | Leads: 150 / 500 used
```

## Related Skills

None. This skill is purely analytical and uses built-in benchmark thresholds for insights.
