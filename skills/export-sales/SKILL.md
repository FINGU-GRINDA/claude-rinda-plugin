# Export Sales Agent Guide

## Buyer Search Rules

- Always exclude buyers previously contacted (check workspace history before searching).
- Supported buyer types: Importer, Distributor, Wholesaler, Retailer.
- Default filters (override via `settings.json` → `targetMarket`):
  - Annual revenue >= $1M (`targetMarket.minRevenue`)
  - Employee count >= 10 (`targetMarket.minEmployees`)
- Default search quantity: 50 buyers per run (`targetMarket.searchQuantity`).

## Follow-up Cadence

Use `settings.json` → `email.followUpDays` to override defaults.

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
