# RINDA Plugin — API Flows

> Extracted from `RINDA_Cowork_Plugin_DevGuide_EN`. Each flow maps to a CLI method the SDK must expose.

---

## Flow 0: Authentication (Token)

**How it works**: User visits `https://alpha.rinda.ai/cli-auth` directly in their browser (not via CLI).

**Steps**:
1. User opens `https://alpha.rinda.ai/cli-auth` in browser
2. User authenticates (Google OAuth or credentials)
3. Frontend displays a token for the user to copy
4. User provides the token to the CLI (e.g. `rinda login <token>` or config file)
5. CLI stores token at `~/.rinda/credentials.json`
6. All subsequent API calls use `Authorization: Bearer <token>`

**Token lifecycle**:
- Stored in `~/.rinda/credentials.json`
- Attached to every request as `Bearer` header
- CLI checks token validity before each call; prompts re-auth if expired

---

## Flow 1: Buyer Search

**Command**: `/rinda:buyer-search`

**Parameters**:
- `industry` — target industry (e.g. "cosmetics")
- `countries` — list of target countries (e.g. ["US", "DE"])
- `buyer_type` — importer | distributor | wholesaler | retailer
- `min_revenue` — minimum annual revenue filter (default $1M)
- `limit` — max results (default 50)

**Steps**:
1. Call `POST /buyers/search` with filters
2. Exclude previously contacted buyers
3. Enrich each result (→ Flow 2)
4. Score buyers using qualification criteria
5. Sort by score, present list
6. Prompt user: create email sequence? (→ Flow 3)

**API endpoint**: `POST {BASE}/buyers/search`

---

## Flow 2: Buyer Enrichment

**Command**: `/rinda:enrich`

**Parameters**:
- `buyer_id` — single buyer ID to enrich

**Steps**:
1. Call `GET /buyers/{buyer_id}/enrich`
2. Return detailed info: contacts, emails, import history

**API endpoint**: `GET {BASE}/buyers/{buyer_id}/enrich`

**Note**: The enrichment sub-agent handles parallel enrichment (up to 50 concurrent).

---

## Flow 3: Sequence Create

**Command**: `/rinda:sequence-create`

**Parameters**:
- `name` — sequence name
- `seq_type` — sequence type (default "cold_outreach")
- `steps` — list of email steps (optional, auto-generated if empty)

**Steps**:
1. Call `POST /sequences` with name, type, steps
2. Return created sequence with ID

**API endpoint**: `POST {BASE}/sequences`

---

## Flow 4: Add Contact to Sequence

**Parameters**:
- `sequence_id` — target sequence
- `buyer_id` — buyer to add

**Steps**:
1. Call sequence add-contact endpoint

**API endpoint**: `POST {BASE}/sequences/{sequence_id}/contacts` (inferred)

---

## Flow 5: Send Email

**Parameters**:
- Target buyer/contact
- Email content (subject, body)

**Steps**:
1. Generate personalized email (using email-writing skill rules)
2. Call send endpoint

**API endpoint**: `POST {BASE}/emails/send` (inferred from `rinda_send_email` tool)

---

## Flow 6: Reply Check

**Command**: `/rinda:reply-check`

**Parameters**:
- `hours` — check window in hours (default 2)

**Steps**:
1. Call `GET /replies?hours={hours}`
2. Classify each reply: INTERESTED | CURIOUS | NOT_NOW | REJECTED
3. Recommend action per classification:
   - INTERESTED → respond immediately
   - CURIOUS → send catalog + schedule follow-up
   - NOT_NOW → reactivate after 90 days
   - REJECTED → tag update only

**API endpoint**: `GET {BASE}/replies?hours={hours}`

---

## Flow 7: Campaign Report

**Command**: `/rinda:campaign-report`

**Parameters**:
- `period` — reporting window (default "7d")

**Steps**:
1. Call `GET /campaigns/stats?period={period}`
2. Return performance metrics (open rate, reply rate, etc.)

**API endpoint**: `GET {BASE}/campaigns/stats?period={period}`

---

## Flow 8: Order History (Repurchase Prediction)

**Parameters**:
- `buyer_id` — specific buyer (optional)
- `days_inactive` — inactivity threshold (optional)

**Steps**:
1. Call `GET /orders` with optional filters
2. Return order history for repurchase analysis

**API endpoint**: `GET {BASE}/orders`

---

## Summary: MCP Tools → CLI Methods

| # | MCP Tool | REST Endpoint | CLI Method |
|---|----------|---------------|------------|
| 0 | — | User visits frontend `/cli-auth` | `login` (stores token) |
| 1 | `rinda_buyer_search` | `POST /buyers/search` | TBD |
| 2 | `rinda_buyer_enrich` | `GET /buyers/{id}/enrich` | TBD |
| 3 | `rinda_sequence_create` | `POST /sequences` | TBD |
| 4 | `rinda_sequence_add_contact` | `POST /sequences/{id}/contacts` | TBD |
| 5 | `rinda_send_email` | `POST /emails/send` | TBD |
| 6 | `rinda_check_replies` | `GET /replies?hours=N` | TBD |
| 7 | `rinda_get_campaign_stats` | `GET /campaigns/stats?period=X` | TBD |
| 8 | `rinda_get_order_history` | `GET /orders` | TBD |

---

## Follow-up Rules (Automated)

These are not separate API calls but logic the plugin orchestrates:

| Condition | Action |
|-----------|--------|
| No reply after 3 days | First follow-up |
| No reply after 7 days | Second follow-up |
| No reply after 14 days | Final follow-up |
| No reply after 14+ days | 30-day wait → reactivation queue |

---

## Open Questions

1. **Endpoint mapping**: The dev guide uses idealized paths (`/buyers/search`, `/sequences`, etc.). Need to map these to actual alpha API endpoints from the OpenAPI spec.
2. **Auth**: Token-based. User gets token from `https://alpha.rinda.ai/cli-auth` (frontend, direct browser access). CLI stores and uses it. ✅ Confirmed.
3. **MCP vs CLI**: Dev guide describes an MCP server wrapping REST. Our plugin uses a Rust CLI instead. The flows are the same but the transport differs.
