---
description: Domain knowledge for B2B export sales workflows including buyer search rules, follow-up cadence, response classification, and reactivation rules. Use when handling export sales tasks, managing buyer outreach sequences, or classifying buyer responses.
---

# Export Sales Agent Guide

## Buyer Search Rules

- Always exclude buyers previously contacted (check workspace history before searching).
- Supported buyer types: Importer, Distributor, Wholesaler, Retailer.
- Default filters:
  - Annual revenue >= $1M (minimum revenue threshold)
  - Employee count >= 10 (minimum employee count)
- Default search quantity: 50 buyers per run.

## Follow-up Cadence

Default follow-up cadence (adjust as needed for your outreach strategy):

| Day | Action |
|-----|--------|
| +3  | First follow-up (no reply) |
| +7  | Second follow-up (no reply) |
| +14 | Final follow-up (no reply) |
| +30 | Move to reactivation queue |

## Buyer Response Classification

Used by `/rinda:reply-check` to categorize inbound replies.

| Label | Meaning | Action |
|-------|---------|--------|
| `INTERESTED` | Positive, wants to proceed | Respond immediately; prioritize personal reply |
| `CURIOUS` | Asking questions, not committed | Send product catalog + schedule follow-up in 3 days |
| `NOT_NOW` | Timing issue, not a rejection | Tag as dormant; reactivate after 90 days |
| `REJECTED` | Explicit refusal | Update tags only; do not re-contact |

## Reactivation Rules

- `NOT_NOW` leads: reactivate after **90 days** with a new angle or updated offer.
- Dormant leads (no opens after full sequence): apply a **6-month cooldown** before any retry.
