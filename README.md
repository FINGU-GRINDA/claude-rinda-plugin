# RINDA AI — B2B Export Sales Automation Plugin

> **RINDA AI** is a Claude Code plugin that **automates the entire B2B export sales process** — from finding overseas buyers, to collecting their contact details, to running email campaigns and managing replies.

It eliminates repetitive sales tasks so that **your sales team can focus on closing deals**.

**[한국어 README 보기 (Korean)](./README.ko.md)**

---

## Who Is This For?

- **Export sales reps** who need to find overseas buyers manually
- **Trading companies** running email outreach to international prospects
- **Startups & SMBs** looking to automate their B2B sales pipeline
- **Sales managers** who want a clear view of campaign performance

---

## Features

### 1. Overseas Buyer Search (`/rinda-ai:buyer-search`)

**Tell the AI your target industry and country — it automatically finds qualified buyers for you.**

- Specify industry (e.g., cosmetics, machinery) and target country (e.g., US, Germany)
- Filter by buyer type: Importer, Distributor, Wholesaler
- Set minimum revenue thresholds and result quantity
- AI **scores and ranks every lead** so you see the best prospects first
- Seamlessly continue to contact enrichment or email campaigns

> **Example**: "Find 50 cosmetics importers in the US with annual revenue over $5M"

---

### 2. Contact Enrichment (`/rinda-ai:enrich`)

**Automatically collect emails, phone numbers, LinkedIn profiles, and product info for your leads.**

- Enrich up to 100 leads at once from your search results
- Collects: email addresses, phone numbers, LinkedIn profiles, product portfolios
- **Automatically identifies decision-makers** (Purchasing Managers, Directors, etc.) and classifies leads:
  - **High priority**: Decision-maker email found + strong product match
  - **Medium priority**: Email found but unclear job title
  - **Low priority**: Only generic email found (info@, contact@)
- Batch processing for reliable, fast data collection

> **Example**: "Enrich the contacts for the buyers we just found"

---

### 3. AI Email Campaign Creation (`/rinda-ai:sequence-create`)

**AI writes a multi-step sales email sequence and sends it to your leads automatically.**

- Set campaign name, tone (professional/friendly/formal/casual), language, and number of steps
- AI **generates subject lines and email bodies** with personalization tokens (recipient name, company name, etc.)
- **Preview before sending** — approve, edit individual steps, or regenerate the entire sequence
- Auto-enroll selected leads with scheduled sending
- Quality assured by built-in email writing guidelines:
  - Subject lines: under 60 characters, includes company name
  - Body structure: Hook (address a need) → Value (your offer) → CTA (specific next step)
  - Follow-ups: each adds new value, progressively shorter

> **Example**: "Create a 6-step outreach sequence called 'US Cosmetics Q1 Outreach' in a professional tone"

---

### 4. Reply Management & Prioritization (`/rinda-ai:reply-check`)

**AI analyzes incoming replies and tells you exactly what to do with each one.**

- Fetches unread replies and classifies each by intent:
  - **Meeting request** → Respond immediately (propose meeting times)
  - **Positive interest** → Respond today (send catalog, offer a call)
  - **Question** → Answer the question + schedule follow-up in 3 days
  - **Not now** → Reactivate in 90 days
  - **Rejected** → Update tags, remove from active sequences
  - **Out of office** → Resume follow-up when they return
  - **Unsubscribe** → Process immediately, do not contact again
- **Sorted by urgency** so you handle the most important replies first

> **Example**: "Check my replies and tell me what I need to do"

---

### 5. Campaign Performance Report (`/rinda-ai:campaign-report`)

**Get a complete overview of your campaign performance with actionable insights.**

- **Funnel analysis**: Leads discovered → Enriched → Contacted → Replied → Meetings booked
- **Email metrics**: Sent, open rate, click rate, reply rate, bounce rate
- **Hot leads list**: High-priority prospects that need immediate attention
- **Industry benchmark comparison**:
  - Open rate: >35% good, >50% excellent
  - Reply rate: >10% good, >20% excellent
  - Bounce rate: <5% healthy
- **AI-generated insights**: Automatic recommendations based on your metrics
- Drill down into specific sequences (7-day / 30-day / 90-day periods)

> **Example**: "Show me the campaign performance for the last 30 days"

---

### 6. Account Connection (`/rinda-ai:connect`)

**Sign in to RINDA AI with your Google account — one-time setup.**

- Run once; you're good to go
- Secure authentication via Google OAuth
- Automatic token refresh — no need to log in again (valid for 14 days)
- Check connection status or log out anytime

---

## Automated Workflow Chaining

Each feature works independently, but they're **designed to flow naturally into the next step**:

```
Buyer Search → Contact Enrichment → Email Campaign → Reply Management → Campaign Report
```

After each step, the AI suggests the next action. If you agree, it automatically carries forward all relevant data (lead IDs, sequence IDs, etc.) so you can keep going without re-entering anything.

---

## Getting Started

### Step 1: Install the Plugin

```bash
# In Claude Code
/plugin marketplace add FINGU-GRINDA/claude-rinda-plugin
```

### Step 2: Connect Your Account

```
/rinda-ai:connect
```

Sign in with your Google account and you're ready!

### Step 3: Start Finding Buyers

```
/rinda-ai:buyer-search
```

Enter your target industry and country, and the AI takes you through the entire sales process — from finding buyers, to enriching contacts, to sending emails.

---

## Frequently Asked Questions (FAQ)

### General

**Q: Do I need programming knowledge to use RINDA AI?**
> Not at all. Just type what you want in plain language. For example: "Find cosmetics importers in the US" and the AI handles everything.

**Q: Is RINDA AI free?**
> RINDA AI operates on a subscription plan. Each plan has a limit on the number of leads you can search. You can check your current usage in the campaign report.

**Q: What industries/products does it support?**
> Any industry suitable for B2B export. Cosmetics, food, machinery, electronics, chemicals, textiles — just specify your target industry.

**Q: Which countries can I search for buyers in?**
> Most countries worldwide. You can target the US, Europe, Southeast Asia, Middle East, and more.

---

### Account & Authentication

**Q: Can I use RINDA AI without a Google account?**
> Currently, only Google OAuth is supported. You need a Google account to sign in.

**Q: What happens when my login expires?**
> Under normal use, tokens are refreshed automatically. You only need to run `/rinda-ai:connect` again if you haven't used the plugin for more than 14 days.

**Q: Is my authentication data secure?**
> Yes. Auth tokens are stored only on your local machine (file permissions 600), and all communication uses HTTPS encryption. Tokens auto-refresh every hour and are deleted immediately on logout.

---

### Buyer Search

**Q: What if I get too few results?**
> Try broadening your search: lower the minimum revenue threshold, add more buyer types, or expand to adjacent industries.

**Q: Will previously contacted buyers show up again?**
> No. Buyers you've already contacted are automatically excluded from search results.

**Q: How many buyers can I search for at once?**
> The default is 50 per search. You can adjust this number as needed.

---

### Contact Enrichment

**Q: How accurate are the collected contacts?**
> The AI collects and verifies information from multiple sources. Leads classified as "high priority" have a confirmed decision-maker email.

**Q: How many contacts can I enrich at once?**
> Up to 100 leads per enrichment run.

---

### Email Campaigns

**Q: Can I edit the AI-generated emails?**
> Yes. After the AI generates the emails, it shows you a preview. You can approve, edit individual steps, or regenerate the entire sequence.

**Q: Can I send emails in languages other than English?**
> Yes. You can specify the language when creating a sequence — English (`en`), Korean (`ko`), German (`de`), French (`fr`), and more.

**Q: Will my emails be flagged as spam?**
> RINDA AI's email guidelines automatically avoid spam trigger words (e.g., "free", "act now", "limited time"). Personalized content and proper sending intervals further minimize spam risk.

**Q: How are follow-up emails spaced?**
> Default intervals are +3 days, +7 days, +14 days, and +30 days. You can adjust the timing when creating a campaign.

---

### Reply Management

**Q: How accurate is the reply classification?**
> The AI automatically analyzes reply intent (meeting request, interest, question, rejection, etc.). It provides specific next-action recommendations alongside each classification, so the final decision is always yours.

**Q: What happens to buyers who say "not now"?**
> They're automatically tagged for reactivation in 90 days with a fresh angle or updated offer.

---

### Reports

**Q: Can I compare my results to industry averages?**
> Yes. Campaign reports automatically compare your metrics against B2B export sales benchmarks (e.g., 35% open rate, 10% reply rate) and tell you whether performance is on track or needs improvement.

**Q: Can I view results for a specific campaign only?**
> Yes. Specify a sequence ID to see step-by-step performance for that particular campaign.

---

## Technical Specifications

| Item | Details |
|------|---------|
| Plugin type | Claude Code Plugin |
| Authentication | Google OAuth 2.0 |
| API communication | HTTPS REST API |
| Token validity | Access: 1 hour (auto-refresh), Refresh: 14 days |
| Credential storage | Local (`~/.rinda/credentials.json`, permissions 600) |
| Built by | [GRINDA AI](https://grinda.ai) |
| License | MIT |

---

## Support & Contact

- Website: [https://rinda.ai](https://rinda.ai)
- Email: support@grinda.ai
- GitHub: [FINGU-GRINDA/claude-rinda-plugin](https://github.com/FINGU-GRINDA/claude-rinda-plugin)
