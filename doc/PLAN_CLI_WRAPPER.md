# Plan: CLI as API Wrapper

> Refactor the plugin so Claude never calls the REST API directly. Instead, `rinda-cli` wraps every API operation. Tokens stay out of Claude's context. Distribution via GitHub Releases + install script.

---

## Why

The current design has Claude construct `fetch()` calls with raw access tokens from `~/.rinda/credentials.json`. Problems:

1. **Security** — access tokens enter Claude's context window
2. **Reliability** — markdown instructions for HTTP calls are fragile (wrong headers, missing params, bad error handling)
3. **Complexity** — SSE polling, pagination, retry logic are hard to describe in markdown

With a CLI wrapper, each subcommand handles auth + API calls internally and returns structured JSON. Claude only sees the output.

---

## Phases

### Phase 1 — Regenerate SDK from OpenAPI spec

The hand-written SDK uses `serde_json::Value` extensively (loosely typed). The OpenAPI spec (`doc/openapi.json`, 12,800 lines) has full schema definitions.

- Parse `doc/openapi.json` and generate proper Rust request/response structs
- Replace `serde_json::Value` fields with typed structs where the spec defines them
- Keep the existing `RindaClient` HTTP helper pattern
- Validate generated models compile against existing endpoint methods

**Output**: Strongly-typed `rinda-sdk` crate with models matching the real API.

### Phase 2 — Add API subcommands to `rinda-cli`

Add subcommands that map 1:1 to plugin commands:

```
rinda auth login|logout|status|ensure-valid    # already exists

rinda buyer-search                             # new
    --query "cosmetics importers"
    --countries US,DE
    [--workspace-id <id>]                      # default from credentials

rinda enrich                                   # new
    --lead-ids id1,id2,...

rinda sequence-create                          # new
    --name "US Cosmetics Q1"
    --group-id <customer-group-id>

rinda sequence-generate                        # new
    --sequence-id <id>
    --tone professional
    [--language en]

rinda reply-check                              # new
    [--hours 2]

rinda campaign-report                          # new
    [--period 7d|30d|90d]
    [--sequence-id <id>]
```

Each subcommand:
1. Reads `~/.rinda/credentials.json`
2. Auto-refreshes token if expired (reuses `ensure-valid` logic)
3. Calls SDK methods
4. Outputs JSON to stdout
5. Exits with appropriate code (0 success, 1 auth error, 2 API error)

For `buyer-search`, the CLI handles the full flow internally:
- POST search -> poll status -> GET results -> print JSON

### Phase 3 — Rewrite markdown commands

Change each command markdown from "how to construct fetch() calls" to "when to use this CLI subcommand and how to interpret the output".

Before (current):
```markdown
## Steps
1. Read ~/.rinda/credentials.json
2. POST /api/v1/lead-discovery/search with headers...
3. Poll GET /api/v1/lead-discovery/session/:id/status...
```

After:
```markdown
## Steps
1. Run: rinda buyer-search --query "<user query>" --countries <countries>
2. Parse the JSON output
3. Present results to user
```

Commands to rewrite:
- `commands/connect.md` — simplify to `rinda auth login`
- `commands/buyer-search.md` — `rinda buyer-search`
- `commands/enrich.md` — `rinda enrich`
- `commands/sequence-create.md` — `rinda sequence-create` + `rinda sequence-generate`
- `commands/reply-check.md` — `rinda reply-check`
- `commands/campaign-report.md` — `rinda campaign-report`

### Phase 4 — Install script + CI release pipeline

**Install script** (`install.sh`, ~30 lines):
- Detect OS (linux, darwin, windows) and arch (x86_64, aarch64)
- Download the matching binary from `https://github.com/FINGU-GRINDA/claude-rinda-plugin/releases/latest`
- Place it in `${CLAUDE_PLUGIN_ROOT}/bin/rinda`
- Make it executable

**GitHub Actions** (extends issue #9):
- On tag push (`v*`), build release binaries for:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- Upload binaries to GitHub Release
- Name format: `rinda-cli-<version>-<target>.tar.gz`

**Plugin integration**:
- `commands/connect.md` checks if binary exists, runs `install.sh` if not
- Or: post-install hook in plugin manifest triggers `install.sh`

### Phase 5 — Simplify hooks

Since each CLI subcommand handles auth internally (read credentials, refresh if needed), the pre-tool-call hook (`ensure-valid` on every tool call) is no longer needed.

- Remove or simplify `hooks/hooks.json`
- Auth refresh happens lazily inside each subcommand
- Fewer moving parts, faster tool calls (no hook overhead)

---

## File Changes Summary

```
Modified:
  crates/rinda-sdk/src/models/*.rs    — regenerated from OpenAPI
  crates/rinda-sdk/src/endpoints/*.rs  — updated to use typed models
  crates/rinda-cli/src/main.rs         — add new subcommands
  crates/rinda-cli/src/commands/mod.rs — register new command modules
  commands/*.md                        — rewrite to use CLI
  hooks/hooks.json                     — simplify or remove

New:
  crates/rinda-cli/src/commands/buyer_search.rs
  crates/rinda-cli/src/commands/enrich.rs
  crates/rinda-cli/src/commands/sequence.rs
  crates/rinda-cli/src/commands/replies.rs
  crates/rinda-cli/src/commands/report.rs
  install.sh
  .github/workflows/release.yml
```

---

## Order of Execution

```
Phase 1 ──> Phase 2 ──> Phase 3 ──> Phase 4 ──> Phase 5
  SDK         CLI         Docs       Release     Cleanup
 types     subcommands   rewrite    pipeline     hooks
```

Phases 4 and 5 are independent and can be done in parallel after Phase 3.
