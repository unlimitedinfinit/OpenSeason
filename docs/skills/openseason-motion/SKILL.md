# Skill: OpenSeason Legal Operating System (Motions, Appeals, Contracts & Wills)

Use this skill whenever a user asks you to inspect a case folder, onboard onto an existing matter, draft or assemble a motion/appeal/complaint/will/lease/demand letter, compute deadlines, or export court-ready Word/PDF packets with OpenSeason.

## Hard Rules

1. **Read `AGENTS.md` first.** Never invent facts, dates, party names, docket numbers, or citations.
2. **Never file or serve.** Remind the user to review every line before filing or executing.
3. **Confidential cases stay offline.** Never call `cloud_push` or `cloud_pull` on a `.sealed` or `confidential` case.

## Fast 5-Step Agent Workflow

1. **1-Call Orientation (`case_context`):**
   - CLI: `.\openseason-cli.exe --json case context <case-dir>`
   - MCP: `case_context`
   - Returns the entire case profile, 3-tier classification (`transactional`, `administrative`, `litigation`), active venue profile (`NJ`, `CO`, `FEDERAL`, `SCOTUS`), timeline events, exhibits, Guided Intake Wizard progress, and Predictive Next Steps in a single call.
2. **Offline PDF Auto-Discovery (`intake_scan`):**
   - CLI: `.\openseason-cli.exe --json intake scan <case-dir>`
   - MCP: `intake_scan`
   - Extracts text from PDFs in `orders/` and `evidence/` and surfaces candidate docket numbers, dates, and court names with source filenames.Confirm candidates with the user before saving.
3. **1-Question-at-a-Time Guided Intake (`intake_status` / `intake_answer`):**
   - CLI: `.\openseason-cli.exe --json intake status <case-dir>`
   - CLI: `.\openseason-cli.exe --json intake answer <case-dir> --id <question_id> --value "<user's answer>"`
   - Walks through missing profile fields, party roles, venue conferral requirements, and guided draft sections one question at a time.
4. **Predictive Procedural Graph & Venue Competence (`case_next_steps` / `check_venue`):**
   - CLI: `.\openseason-cli.exe --json case next-steps <case-dir>`
   - CLI: `.\openseason-cli.exe --json check venue <case-dir>`
   - Computes downstream legal branches only from a user-entered order/trigger date with a source. If that date is missing, `computed_deadline` is null and status is `needs_trigger_date`. Never invent a deadline from today. Holiday-aware math, **NJ Pause/Toll (`pause_remaining` under R. 2:4-3(e))** vs. **CO/Federal Full Reset (`reset_full`)**, and **+3 day mail service offsets**. Verifies venue-specific oaths where the pack is the source (NJ R. 1:4-4(b), 28 U.S.C. § 1746, SCOTUS Rule 14.1, Will witnesses). Colorado C.R.C.P. 121 conferral and C.R.S. § 13-27-106 are `needs_verification` — appellate motions may follow C.A.R., and a UUDA declaration often needs a signing date and place.
5. **Pre-Flight Check, Packet Assembly & Cross-Venue Escalation:**
   - CLI: `.\openseason-cli.exe --json check preflight <case-dir>`
   - CLI: `.\openseason-cli.exe --json export packet <case-dir>` (or `export pleading <case-dir> --kind <kind>`)
   - CLI: `.\openseason-cli.exe --json case escalate <case-dir> --target trial_lawsuit|appellate_appeal|mandamus|scotus_cert` (graduates a Pre-Suit Notice to a Lawsuit or escalates a Trial case to Appeal/Mandamus/SCOTUS while copying lower-court orders/filings into the new case's `evidence/` appendix).
