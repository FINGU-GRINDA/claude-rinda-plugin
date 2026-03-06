# RINDA AI — Claude Cowork Custom Plugin Development Guide

> Complete guide for integrating RINDA's buyer search, enrichment, email sequence, and reply management features as a Claude Cowork plugin.

**GRINDA AI | March 2026 | Internal Technical Document**

---

## 1. Plugin Architecture Overview

The Claude Cowork plugin uses a simple file-based structure written in Markdown + JSON, connecting to the RINDA API via an MCP server.

### 1.1 Directory Structure

```
rinda-ai/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata
├── .mcp.json                # MCP server connection config
├── commands/                # Slash commands
│   ├── buyer-search.md      # /rinda:buyer-search
│   ├── enrich.md            # /rinda:enrich
│   ├── sequence-create.md   # /rinda:sequence-create
│   ├── reply-check.md       # /rinda:reply-check
│   └── campaign-report.md   # /rinda:campaign-report
├── skills/                  # Domain knowledge auto-referenced by Claude
│   ├── export-sales/SKILL.md
│   ├── email-writing/SKILL.md
│   └── buyer-qualification/SKILL.md
└── agents/
    └── enrichment-agent.md  # Dedicated enrichment sub-agent
```

### 1.2 Core Concepts: 4 Components

| Component | Description |
|-----------|-------------|
| **Skills** | Expert knowledge Claude references contextually. Buyer search logic, email rules, and validation criteria written in Markdown. |
| **Commands** | Slash commands like `/rinda:buyer-search`. Each command is defined in a single Markdown file with prompts, parameters, and rules. |
| **Connectors** | Configuration to connect the RINDA API via MCP. Server URL and authentication defined in `.mcp.json`. |
| **Sub-agents** | Auxiliary agents for parallel processing of complex tasks. Example: 50 simultaneous enrichment operations. |

---

## 2. Step-by-Step Development

### Step 1: plugin.json

Define the plugin metadata.

```json
{
  "name": "rinda-ai",
  "version": "1.0.0",
  "description": "RINDA AI - International sales automation. Buyer search, enrichment, email sequences, reply management.",
  "author": "GRINDA AI",
  "homepage": "https://rinda.ai",
  "tags": ["sales", "export", "b2b", "outreach"]
}
```

### Step 2: MCP Server Connection (.mcp.json)

Connect the RINDA API server via MCP. This is the core configuration that enables Claude to use RINDA functions as Tools.

```json
{
  "mcpServers": {
    "rinda": {
      "type": "sse",
      "url": "https://api.rinda.ai/mcp/sse",
      "headers": { "Authorization": "Bearer <RINDA_API_KEY>" },
      "tools": [
        "rinda_buyer_search", "rinda_buyer_enrich",
        "rinda_sequence_create", "rinda_sequence_add_contact",
        "rinda_send_email", "rinda_check_replies",
        "rinda_get_campaign_stats", "rinda_get_order_history"
      ]
    }
  }
}
```

> ⚠️ **MCP Server Development Required**
> The above configuration assumes an MCP server exists in the RINDA backend. A separate server that wraps the REST API as MCP must be developed. This is covered in Section 3.

### Step 3: Writing Skills

3 core skills for the RINDA plugin:

#### `skills/export-sales/SKILL.md`

```markdown
# Export Sales Agent Guide

## Buyer Search Rules
- Always exclude previously contacted buyers
- Types: Importer, Distributor, Wholesaler, Retailer
- Default filters: Annual revenue $1M+, 10+ employees

## Email Writing Rules
- First email: 3 sentences max / Include company name in subject
- CTA: Specific action (meeting / sample / catalog)
- Tone: Professional but friendly

## Follow-up Rules
- No reply 3 days → First follow-up / 7 days → Second / 14 days → Last
- After that, 30-day wait → Move to reactivation queue

## Buyer Response Classification
- INTERESTED → Respond immediately / CURIOUS → Catalog + follow-up
- NOT_NOW → Reactivate after 90 days / REJECTED → Tag update only
```

#### `skills/email-writing/SKILL.md`

```markdown
# Email Writing Guide

## Personalization Required: Name + title, company product mention, recent news reference

## Subject Line Formula (open rate optimization)
- [Company name] + [value proposition]
- Include numbers / Question format

## Body: Hook (need) → Value (offer) → CTA (next step)

## Prohibited: Dear Sir/Madam, attachments in first email, spam trigger words
```

### Step 4: Writing Commands

Example: `commands/buyer-search.md`

```markdown
# /rinda:buyer-search

## Parameters (confirm with user)
- Industry / Country / Buyer type / Minimum size / Quantity (default 50)

## Workflow
1. Search for buyers matching criteria via rinda_buyer_search
2. Automatically exclude previously contacted buyers
3. Collect detailed information via rinda_buyer_enrich
4. Calculate quality score (refer to export-sales skill)
5. Sort by score and present the list
6. Ask whether to create an email sequence
```

---

## 3. MCP Server Development (Python FastMCP)

Wrap the RINDA REST API as MCP Tools. This is the most technically involved part.

```python
# pip install fastmcp httpx
from fastmcp import FastMCP
import httpx, os

mcp = FastMCP("RINDA AI")
BASE = "https://api.rinda.ai/v1"
def headers(): return {"Authorization": f"Bearer {os.environ['RINDA_API_KEY']}"}

@mcp.tool()
async def rinda_buyer_search(industry="cosmetics", countries=["US"],
        buyer_type="importer", min_revenue=1000000, limit=50) -> dict:
    """Search for overseas buyers matching target criteria."""
    async with httpx.AsyncClient() as c:
        r = await c.post(f"{BASE}/buyers/search", headers=headers(),
            json={"industry": industry, "countries": countries,
                  "buyer_type": buyer_type, "min_revenue": min_revenue, "limit": limit})
        return r.json()

@mcp.tool()
async def rinda_buyer_enrich(buyer_id: str) -> dict:
    """Collect detailed buyer info (contacts, email, import history)."""
    async with httpx.AsyncClient() as c:
        return (await c.get(f"{BASE}/buyers/{buyer_id}/enrich", headers=headers())).json()

@mcp.tool()
async def rinda_sequence_create(name: str, seq_type="cold_outreach",
        steps: list = None) -> dict:
    """Create an email sequence."""
    async with httpx.AsyncClient() as c:
        return (await c.post(f"{BASE}/sequences", headers=headers(),
            json={"name": name, "type": seq_type, "steps": steps or []})).json()

@mcp.tool()
async def rinda_check_replies(hours: int = 2) -> dict:
    """Check buyer replies within the last N hours."""
    async with httpx.AsyncClient() as c:
        return (await c.get(f"{BASE}/replies?hours={hours}", headers=headers())).json()

@mcp.tool()
async def rinda_get_campaign_stats(period="7d") -> dict:
    """Email campaign performance statistics."""
    async with httpx.AsyncClient() as c:
        return (await c.get(f"{BASE}/campaigns/stats?period={period}", headers=headers())).json()

@mcp.tool()
async def rinda_get_order_history(buyer_id=None, days_inactive=None) -> dict:
    """Buyer order history (for repurchase prediction)."""
    p = {}
    if buyer_id: p["buyer_id"] = buyer_id
    if days_inactive: p["days_inactive"] = days_inactive
    async with httpx.AsyncClient() as c:
        return (await c.get(f"{BASE}/orders", headers=headers(), params=p)).json()

if __name__ == "__main__": mcp.run(transport="sse", port=8080)
```

---

## 4. Deployment & Customer Customization

### 4.1 Deployment Options

| Option | Description | Notes |
|--------|-------------|-------|
| **Cloud (Recommended)** | Deploy MCP server to AWS/Vercel, set cloud URL in `.mcp.json` | Best for scalability and managed uptime |
| **Local** | Run via `uvx/npx` on user's machine | Data stays local — better for security |

```bash
# Deploy to GitHub
git push origin main

# User installation
claude plugin marketplace add grinda-ai/rinda-cowork-plugin
claude plugin install rinda-ai@rinda-cowork-plugin
```

### 4.2 Customer Customizable Items

After installing the RINDA plugin and clicking **Customize**, Claude walks through interactive setup:

| Item | Details |
|------|---------|
| **Company Info** | Company name, products, USP, certifications (FDA/CE, etc.) |
| **Target Market** | Countries, buyer types, minimum size |
| **Pricing** | FOB basis, currency, MOQ, payment terms, sample policy |
| **Email Tone** | Formal/casual, brand voice, prohibited words |
| **Integrations** | Slack channel, Drive folder, Calendly URL |

### Development Roadmap

| Phase | Duration | Scope |
|-------|----------|-------|
| **Phase 1** | 2 weeks | MCP server + `buyer_search`, `buyer_enrich`, `sequence_create` implementation |
| **Phase 2** | 1 week | Plugin structure + 3 Skills + 4 Commands |
| **Phase 3** | 1 week | Testing + prompt optimization + customization guide |
| **Phase 4** | 1 week | GitHub deployment + documentation + onboarding process |

**Total: ~5 weeks. Requires 1 Python FastMCP developer.**

---

*GRINDA AI — Internal Technical Document — 2026.03*
