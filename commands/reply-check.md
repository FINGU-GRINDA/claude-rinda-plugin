# /rinda-ai:reply-check

Check unread email replies from leads, classify them by intent, and get a prioritized action list telling you exactly what to do with each reply.

## Parameters

- `hours` (number, optional, default: `2`) — How far back to look for replies, in hours. Use `24` to check the last full day, `168` for the past week.

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

### Step 2 — Fetch intent summary

```
GET https://app.rinda.ai/api/v1/email-replies/stats/by-intent?workspaceId=${workspaceId}
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "meeting_request": 2,
  "positive_interest": 5,
  "question": 3,
  "not_now": 4,
  "rejected": 1,
  "out_of_office": 2,
  "unsubscribe": 1
}
```

Use this to give the user an upfront overview before showing individual replies.

### Step 3 — Fetch individual unread replies

```
GET https://app.rinda.ai/api/v1/email-replies?limit=50&isRead=false&workspaceId=${workspaceId}
Authorization: Bearer ${accessToken}
```

Expected response:

```json
{
  "replies": [
    {
      "id": "reply_xyz",
      "from": "jane@acme.com",
      "companyName": "Acme Importers LLC",
      "subject": "Re: Introducing our cosmetics line",
      "body": "Hi, we'd be interested in learning more. Can we schedule a call?",
      "intent": "meeting_request",
      "sentiment": "positive",
      "receivedAt": "2026-03-06T08:30:00Z",
      "leadId": "lead_abc123",
      "sequenceName": "US Cosmetics Q1 Outreach"
    }
  ],
  "total": 17
}
```

### Step 4 — Classify and determine actions

Using the **export-sales** skill classification rules, map each reply's `intent` to a recommended action:

| Intent | Classification | Recommended Action |
|--------|---------------|-------------------|
| `meeting_request` | INTERESTED | Respond immediately — propose 2-3 meeting times. High priority. |
| `positive_interest` | INTERESTED | Respond today — send product catalog, offer a call. High priority. |
| `question` | CURIOUS | Answer the question + attach catalog. Schedule follow-up in 3 days. |
| `not_now` | NOT_NOW | Mark for reactivation in 90 days. No immediate action needed. |
| `rejected` | REJECTED | Update lead tag to "rejected". Remove from active sequences. |
| `out_of_office` | WAIT | Note return date if mentioned. Resume follow-up when they are back. |
| `unsubscribe` | REJECTED | Unsubscribe immediately. Do not contact again. |

Prioritize output order: INTERESTED replies first, then CURIOUS, then WAIT, then NOT_NOW, then REJECTED.

### Step 5 — Present prioritized action list

First, show the intent overview:

```
Email Reply Summary
===================
Total unread: 17 replies

By intent:
  Meeting requests:   2  [RESPOND NOW]
  Positive interest:  5  [RESPOND TODAY]
  Questions:          3  [ANSWER + FOLLOW UP]
  Not now:            4  [REACTIVATE IN 90 DAYS]
  Rejected:           1  [TAG UPDATE ONLY]
  Out of office:      2  [WAIT FOR RETURN]
  Unsubscribe:        1  [UNSUBSCRIBE]
```

Then list individual replies grouped by priority:

```
RESPOND NOW (2)
---------------
1. Acme Importers LLC — jane@acme.com
   Subject: Re: Introducing our cosmetics line
   "Hi, we'd be interested. Can we schedule a call?"
   Action: Reply with 2-3 meeting time options (suggest calendar link if available)
   Sequence: US Cosmetics Q1 Outreach

2. [next meeting_request reply...]

RESPOND TODAY (5)
-----------------
[positive_interest replies...]

ANSWER + SCHEDULE FOLLOW-UP (3)
--------------------------------
[question replies...]

NO ACTION NEEDED
-----------------
NOT NOW (4): Mark for 90-day reactivation
REJECTED (1): Update tags, remove from sequences
UNSUBSCRIBE (1): Process unsubscribe request
```

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| 401 Unauthorized | Token expired | Tell user: "Session expired. Please run `/rinda-ai:connect` to re-authenticate." |
| 429 Too Many Requests | Rate limit hit | Wait 30 seconds, then retry. |
| Empty `replies` array | No unread replies in the period | Report: "No unread replies in the last ${hours} hours." Suggest checking back later or increasing the `hours` parameter. |
| Stats endpoint fails | Temporary backend issue | Skip the summary step and proceed to individual replies. Report the stats error separately. |
| Network error | Connectivity issue | Inform the user and ask them to retry. |

## Output Format

Start with the intent summary table for a quick overview. Follow with replies grouped by priority tier. For each reply include: company name, sender email, subject line, a one-line preview of the body, and the specific recommended action. Keep descriptions concise — one action per reply.

End with:

```
${INTERESTED.length} replies need your attention today.
Track your campaign performance: /rinda-ai:campaign-report
```

## Related Skills

- **export-sales** — Used in Step 4 for intent-to-action classification, follow-up timing rules, and reactivation logic.
