# Changelog

All notable changes to the RINDA AI plugin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-03-06

### Added

- Plugin manifest (`plugin.json`) and marketplace metadata (`marketplace.json`)
- 6 plugin commands:
  - `buyer-search` — Search for B2B buyers matching target criteria
  - `campaign-report` — Generate campaign performance reports
  - `connect` — Authenticate with the RINDA AI platform (OAuth via Google)
  - `enrich` — Enrich buyer profiles with additional contact and company data
  - `reply-check` — Check and manage email replies from outreach campaigns
  - `sequence-create` — Create and launch email outreach sequences
- 3 domain knowledge skills:
  - `buyer-qualification` — Frameworks for qualifying B2B export buyers
  - `email-writing` — Best practices for writing effective outreach emails
  - `export-sales` — Domain knowledge for B2B export sales workflows
- 1 enrichment sub-agent for automated buyer data enrichment
- Rust SDK crate (`rinda-sdk`) with full RINDA API coverage
- Rust CLI crate (`rinda-cli`) providing the `rinda` binary for authentication
- Shared types crate (`rinda-common`)
