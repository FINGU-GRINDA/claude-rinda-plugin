---
description: Create an AI-generated multi-step email sequence and enroll leads into it.
---

# Sequence Create Skill

Create an AI-generated multi-step email sequence and enroll leads into it. Covers sequence creation, AI content generation, user approval, and bulk enrollment with scheduling.

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `name` | string | Yes | — | Name for the email sequence. Example: `"US Cosmetics Q1 Outreach"`. |
| `customer_group_id` | string | No | — | ID of the customer group to associate with the sequence. |
| `tone` | string | No | `"professional"` | Email tone. Options: `"professional"`, `"friendly"`, `"formal"`, `"casual"`. |
| `language` | string | No | `"en"` | Language code for generated emails. Example: `"en"`, `"de"`, `"fr"`. |
| `step_count` | number | No | `6` | Number of email steps in the sequence. |
| `lead_ids` | string[] | No | — | Lead IDs to enroll after generation. If not provided, ask the user whether to enroll leads now or later. |
| `user_email_account_id` | string | No | — | Email account ID to send from. Required if enrolling leads. |

## Implementation

This skill delegates to the **rinda-agent** sub-agent, which executes the **Workflow 3 — Email Sequence Creation** workflow. See `agents/rinda-agent.md` for full implementation details including AI generation, user approval flow, bulk enrollment, and workflow chaining.

## Related Skills

- **email-writing** — Used to evaluate generated content quality, personalization, subject lines, and CTA effectiveness.
