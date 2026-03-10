# RINDA AI — B2B Export Sales Automation

> A Claude Code **plugin marketplace** for B2B export sales automation — find overseas buyers, enrich contacts, run email campaigns, manage replies, and track performance.

**[한국어 README 보기 (Korean)](./README.ko.md)**

---

## Quick Start

```bash
# 1. Install the marketplace
/plugin marketplace add FINGU-GRINDA/claude-rinda-plugin

# 2. Connect your RINDA account
/rinda-ai:rinda connect my account

# 3. Start using it
/rinda-ai:rinda find cosmetics importers in the US
```

The CLI binary installs automatically on first session.

---

## What It Does

Tell the AI what you need in plain language. It uses the `rinda-cli` tool behind the scenes.

| Workflow | Example |
|----------|---------|
| **Buyer Search** | "Find 50 cosmetics importers in the US with revenue over $5M" |
| **Contact Enrichment** | "Enrich the contacts for those buyers" |
| **Email Campaigns** | "Create a 6-step outreach sequence called 'US Cosmetics Q1'" |
| **Reply Management** | "Check my replies and tell me what to do" |
| **Campaign Reports** | "Show campaign performance for the last 30 days" |

Workflows chain naturally — after each step, the AI suggests the next action and carries forward all relevant data (lead IDs, sequence IDs, etc.).

---

## How It Works

### Buyer Search

Specify industry, country, buyer type, and minimum revenue. The AI searches, scores each lead using weighted criteria (revenue, employee count, import history, product relevance), and presents a ranked table.

### Contact Enrichment

Collects emails, phone numbers, LinkedIn profiles, and product info. Classifies leads by priority:

- **High**: Decision-maker email + strong product match
- **Medium**: Email found, unclear job title
- **Low**: Generic email only (info@, contact@)

### Email Campaigns

Creates multi-step email sequences with AI-generated subject lines and bodies. Follows built-in guidelines:

- Subject lines under 60 characters, includes company name
- Body: Hook → Value → CTA
- Follow-ups add new value, progressively shorter

### Reply Management

Classifies replies by intent and urgency:

| Intent | Action |
|--------|--------|
| Meeting request | Respond immediately |
| Positive interest | Respond today |
| Question | Answer + follow up in 3 days |
| Not now | Reactivate in 90 days |
| Rejected | Update tags, remove from sequence |
| Unsubscribe | Process immediately |

### Campaign Reports

Funnel analysis, email metrics, hot leads list, and AI-generated insights. Compares against B2B export benchmarks (open rate >35% good, reply rate >10% good).

---

## Architecture

This repo is a **plugin marketplace** — it can host multiple plugins under `plugins/`.

```
.claude-plugin/
  marketplace.json          # Central plugin registry
  plugin.json               # Marketplace metadata
plugins/
  rinda-ai/                 # B2B export sales plugin
    .claude-plugin/plugin.json
    hooks/hooks.json        # Auto-installs CLI on session start
    skills/
      rinda/
        SKILL.md            # Main skill (CLI commands + workflows)
        references/
          buyer-qualification.md
          email-writing.md
          export-sales.md
bin/
  install.sh                # CLI installer (cross-platform)
crates/
  cli/                      # rinda-cli (Rust, handles auth + API)
  sdk/                      # Auto-generated from OpenAPI spec
```

### CLI

The `rinda-cli` binary handles authentication and all API calls. It installs to `~/.rinda/bin/rinda-cli` automatically via a SessionStart hook.

| Command | Description |
|---------|-------------|
| `rinda-cli auth ensure-valid` | Refresh token if expired |
| `rinda-cli buyer search` | Search for buyers |
| `rinda-cli buyer enrich` | Enrich a lead |
| `rinda-cli reply check` | Check email replies |
| `rinda-cli campaign stats` | Get campaign stats |
| `rinda-cli sequence create` | Create email sequence |
| `rinda-cli email send` | Send an email |

---

## FAQ

**Do I need programming knowledge?**
No. Just describe what you want in plain language.

**What industries/countries does it support?**
Any B2B export industry (cosmetics, food, machinery, electronics, etc.) and most countries worldwide.

**Is my data secure?**
Auth tokens are stored locally at `~/.rinda/credentials.json` (permissions 600). All communication uses HTTPS. Tokens auto-refresh and are deleted on logout.

**What if my login expires?**
Tokens refresh automatically. Only re-authenticate if unused for 14+ days.

---

## Technical Details

| Item | Details |
|------|---------|
| Plugin type | Claude Code Marketplace |
| Authentication | Google OAuth 2.0 |
| CLI | Rust (cross-platform: Linux, macOS, Windows) |
| SDK | Auto-generated from OpenAPI spec via progenitor |
| Token validity | Access: 1 hour (auto-refresh), Refresh: 14 days |
| Credentials | `~/.rinda/credentials.json` |
| License | MIT |

---

## Support

- Website: [rinda.ai](https://rinda.ai)
- Email: support@grinda.ai
- GitHub: [FINGU-GRINDA/claude-rinda-plugin](https://github.com/FINGU-GRINDA/claude-rinda-plugin)
