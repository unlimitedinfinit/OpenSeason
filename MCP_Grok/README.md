# JustLegal Assistant: GrokBot Blueprints & Connectors

This directory contains the public templates, system prompts, skills, and bridge configurations for running **JustLegal Assistant** on x.ai GrokBot and connecting it to **JustLegal Desktop** (`OpenSeason`).

---

## The Two GrokBot Personas

### 1. JustLegal Navigator (General Civil Self-Help)
- **Target Audience:** Pro se litigants, tenants, consumers, and families navigating civil court cases, appeals, and routine legal instruments.
- **Recipe & Prompt:** `bot_civil_recipe.json` & `system_prompt_civil.md`
- **Capabilities:**
  - Plain-language intake and time-sensitive obligation detector.
  - Sourced forms discovery across 22,000+ official court forms on [justlegal.me](https://justlegal.me).
  - Pasted court order analysis and rule-based deadline calculation.
  - Working document outlines, certificates of service, and attorney referral guides.

### 2. JustLegal Fraud & Whistleblower Navigator (EO 14395 & Qui Tam)
- **Target Audience:** Citizen investigators, healthcare whistleblowers, and relators investigating government program fraud under **Executive Order 14395** and the **False Claims Act (31 U.S.C. § 3729)**.
- **Recipe & Prompt:** `bot_fraud_recipe.json` & `system_prompt_fraud.md`
- **Capabilities:**
  - Instant NPI & USAspending entity verification (identifying ghost clinics and shell companies).
  - FRCP Rule 9(b) 6-element viability scoring (0–100 score).
  - Strict statutory seal warnings (31 U.S.C. § 3730(b)(2)) prohibiting premature social media publication.
  - HIPAA Whistleblower Safe Harbor guidance (45 C.F.R. § 164.502(j)).
  - Direct routing to The Anti-Fraud Coalition (TAF), State Medicaid Fraud Control Units (MFCUs), and DOJ Fraud Section.

---

## How to Create and Share on x.ai GrokBot

1. Log into your x.ai account and navigate to **Grok Bot** (`https://x.ai/bot/create`).
2. Copy the name, description, and instructions from `MCP_Grok/system_prompt_fraud.md` (or `system_prompt_civil.md`).
3. Under Bot Settings, select **Share as Template**. This creates an unpublished draft.
4. Review the draft to ensure no private credentials or internal URLs are included.
5. Click **Publish as Public Link** and copy the resulting `https://x.ai/bot/<id>` link.
6. Share this link on X, GitHub, or embed it on [justlegal.me/fraud-triage](https://justlegal.me/fraud-triage).

---

## How GrokBot Interacts with JustLegal Desktop

When a user has **JustLegal Desktop** (`OpenSeason.exe` / `openseason-cli.exe`) installed on Windows or macOS:
1. GrokBot can request authorization to execute read-only diagnostic commands:
   ```bash
   .\openseason-cli.exe --json doctor
   .\openseason-cli.exe --json case context <path>
   .\openseason-cli.exe --json qui-tam verify-entity --name "<target>"
   .\openseason-cli.exe --json qui-tam audit-9b <path>
   ```
2. **Explicit User Approval:** Grok requires the user to approve local command execution on their machine.
3. **Airgap Enforcement:** The local engine strictly forbids remote inspection of `.sealed` or confidential case folders, preserving total protection for sensitive qui tam dossiers.
