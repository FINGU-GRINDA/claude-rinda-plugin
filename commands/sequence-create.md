# /rinda-ai:sequence-create

Create an AI-generated multi-step email sequence and enroll leads into it. Covers sequence creation, AI content generation, user approval, and bulk enrollment with scheduling.

## Parameters

- `name` (string, **required**) — Name for the email sequence. Example: "US Cosmetics Q1 Outreach".
- `customer_group_id` (string, optional) — ID of the customer group (target segment) to associate with the sequence. If omitted, the sequence is created without a group association.
- `tone` (string, optional, default: `"professional"` from `settings.json` `email.tone`) — Email tone. Options: `"professional"`, `"friendly"`, `"formal"`, `"casual"`.
- `language` (string, optional, default: `"en"` from `settings.json` `email.language`) — Language code for generated emails. Example: `"en"`, `"de"`, `"fr"`.
- `step_count` (number, optional, default: `6`) — Number of email steps in the sequence.
- `lead_ids` (string[], optional) — Lead IDs to enroll after generation. If not provided, ask the user whether to enroll leads now or later.
- `user_email_account_id` (string, optional) — Email account ID to send from. Ask the user if not provided and lead enrollment is requested.

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

### Step 2 — Create the sequence

```
POST https://app.rinda.ai/api/v1/sequences
Authorization: Bearer ${accessToken}
Content-Type: application/json

{
  "name": "${name}",
  "workspaceId": "${workspaceId}",
  "customerGroupId": "${customer_group_id}"
}
```

Expected response:

```json
{
  "id": "seq_abc123",
  "name": "US Cosmetics Q1 Outreach",
  "status": "draft",
  "workspaceId": "uuid"
}
```

Save `sequenceId` (`id` from the response) for subsequent calls.

### Step 3 — Generate email steps with AI

```
POST https://app.rinda.ai/api/v1/sequences/${sequenceId}/generate
Authorization: Bearer ${accessToken}
Content-Type: application/json

{
  "model": "gpt-4o",
  "tone": "${tone}",
  "language": "${language}",
  "stepCount": ${step_count},
  "workspaceId": "${workspaceId}"
}
```

Expected response:

```json
{
  "steps": [
    {
      "stepNumber": 1,
      "subject": "Introducing [Company] to your product line",
      "body": "Dear {{firstName}},\n\nI came across...",
      "delayDays": 0
    },
    {
      "stepNumber": 2,
      "subject": "Following up — [Product] for your market",
      "body": "Hi {{firstName}},\n\nI wanted to follow up...",
      "delayDays": 3
    }
  ]
}
```

Apply **email-writing** skill guidelines when reviewing generated content for:
- Personalization token usage (`{{firstName}}`, `{{companyName}}`)
- Subject line quality (length, curiosity, relevance)
- Body structure (opening hook, value prop, CTA)

### Step 4 — Show steps to user for approval

Present each generated step clearly:

```
Generated Sequence: "${name}" (${step_count} steps)
====================================================

Step 1 (Day 0) — Initial outreach
  Subject: Introducing [Company] to your product line
  Preview: Dear {{firstName}}, I came across your company while researching...

Step 2 (Day 3) — First follow-up
  Subject: Following up — [Product] for your market
  Preview: Hi {{firstName}}, I wanted to follow up on my previous email...

[... remaining steps ...]

Do you approve these steps, or would you like to edit any of them?
Options: (A) Approve and proceed to enrollment | (E) Edit step N | (R) Regenerate all
```

Wait for user confirmation before proceeding to enrollment. If the user requests edits, apply them and re-display the updated step before continuing.

### Step 5 — Enroll leads (if approved and lead_ids provided)

Only proceed if the user has approved the sequence and `lead_ids` are available.

```
POST https://app.rinda.ai/api/v1/admin/sequences/${sequenceId}/enrollments/bulk-with-scheduling
Authorization: Bearer ${accessToken}
Content-Type: application/json

{
  "leadIds": ["lead_abc123", "lead_def456"],
  "userEmailAccountId": "${user_email_account_id}"
}
```

Expected response:

```json
{
  "enrolled": 8,
  "skipped": 2,
  "scheduledStart": "2026-03-07T09:00:00Z"
}
```

### Step 6 — Report enrollment results

```
Sequence Created Successfully
==============================
Name:      US Cosmetics Q1 Outreach
Sequence:  seq_abc123
Steps:     6 emails over 14 days

Enrollment:
  Enrolled:  8 leads
  Skipped:   2 (already in sequence or unsubscribed)
  First email scheduled: Fri Mar 7, 9:00 AM (recipient timezone)

Next step: Monitor replies with /rinda-ai:reply-check
```

If `lead_ids` were not provided, report sequence creation only and prompt:

```
Sequence "${name}" created (seq_abc123) with ${step_count} steps.
To enroll leads, run /rinda-ai:sequence-create with lead_ids, or add leads from the RINDA dashboard.
```

## Error Handling

| Error | Cause | Resolution |
|-------|-------|------------|
| 401 Unauthorized | Token expired | Tell user: "Session expired. Please run `/rinda-ai:connect` to re-authenticate." |
| 429 Too Many Requests | Rate limit hit | Wait 30 seconds, then retry the failed request once. |
| Sequence creation 400 | Missing required field or invalid `customerGroupId` | Report the validation error and ask the user to correct the parameter. |
| Generation timeout | AI generation taking too long | Inform the user the request is taking longer than expected. Wait up to 60 seconds before reporting failure. |
| Enrollment 400 | Invalid `userEmailAccountId` | Ask the user to provide a valid email account ID from the RINDA dashboard. |
| `enrolled: 0` | All leads skipped | Inform the user that all leads were already enrolled or unsubscribed. Suggest trying a different lead list. |

## Output Format

Present each step with step number, delay (days from sequence start), subject line, and a preview of the first two lines of the body. Use clear section headers. End with enrollment summary and next-step suggestion.

## Related Skills

- **email-writing** — Used in Step 3/4 to evaluate generated content quality, personalization, subject lines, and CTA effectiveness.
