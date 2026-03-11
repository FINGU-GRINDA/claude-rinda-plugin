# CLI API Endpoint Status

Tested on 2026-03-10 against `alpha.rinda.ai`.

## Working

| CLI Command | API Endpoint | Method | Notes |
|---|---|---|---|
| `auth url` | — | — | Returns hardcoded URL |
| `auth token <token>` | `POST /api/v1/auth/refresh` | POST | Exchanges refresh token for access token. Response wrapped in `data` envelope. |
| `auth status` | — | — | Reads local credentials file |
| `auth logout` | — | — | Deletes local credentials file |
| `auth ensure-valid` | `POST /api/v1/auth/refresh` | POST | Auto-refreshes expired access token using stored refresh token |
| `campaign stats` | `GET /api/v1/dashboard/stats` | GET | Params: `startDate`, `endDate`. Returns lead/email/open-rate stats. |
| `reply check` | `GET /api/v1/email-replies` | GET | Params: `limit`, `offset`, `isRead`, `sentiment`, etc. Returns reply list. |
| `order history` | `GET /api/v1/leads/search` | GET | Many filter params. Returns paginated lead list. Works as order-history approximation. |
| `buyer sessions` | `GET /api/v1/lead-discovery/db/sessions` | GET | Lists past search sessions for a workspace. Params: `workspaceId` (required), `userId` (optional). |
| `buyer status` | `GET /api/v1/lead-discovery/db/sessions/{sessionId}` | GET | Returns session detail with status, progress, query. |
| `buyer results` | `GET /api/v1/lead-discovery/db/sessions/{sessionId}/results` | GET | Returns search results for a completed session. |
| `buyer messages` | `GET /api/v1/lead-discovery/db/sessions/{sessionId}/messages` | GET | Returns clarification messages for a session. |

## Broken

| CLI Command | API Endpoint | Method | Issue |
|---|---|---|---|
| `buyer search` | `POST /api/v1/lead-discovery/search` | POST | SSE-streaming endpoint. Now uses raw reqwest with `Accept: text/event-stream` to consume events and extract session_id. |
| `buyer enrich` | `POST /api/v1/lead-discovery/enrich` | POST | Untested but likely same issue — empty schema in spec. |

## Not Tested (Destructive)

| CLI Command | API Endpoint | Method | Notes |
|---|---|---|---|
| `email send` | `POST /api/v1/emails/send` | POST | Sends real email — skipped to avoid side effects |
| `sequence create` | `POST /api/v1/sequences` | POST | Creates real sequence |
| `sequence add-contact` | `POST /api/v1/sequences/{id}/enrollments` | POST | Enrolls lead in sequence |

## API Response Envelope

All RINDA API responses are wrapped in a standard envelope:

```json
{
  "success": true,
  "code": "S200",
  "message": "...",
  "data": { ... },
  "timestamp": "2026-03-10T09:50:06.155Z"
}
```

The CLI must read from `data`, not from the top-level response object.

## Alternative Endpoints for Buyer Search

The `lead-discovery/search` endpoint is designed for the web UI's conversational AI search flow (SSE streaming). For a CLI-friendly buyer search, consider:

- `GET /api/v1/leads/search` — simple paginated search with filters (already used by `order history`). Supports: `search`, `country`, `businessType`, `leadStatus`, `sortField`, `sortOrder`, `limit`, `offset`, etc.
- `POST /api/v1/admin/buyer-search/start` — admin buyer search (also returns 422, schema unknown).
