# Documentation Index & System Map

OpenSeason is a local, privacy-first **Legal Operating System & Court Document Compiler** for self-represented filers, paralegals, and AI coding/legal assistants. It supports everyday legal instruments (**Tier 1 Transactional**: Wills, Leases, Contractor Agreements, Powers of Attorney), pre-suit administrative demands (**Tier 2 Administrative**: Cure Notices, Security Deposit Demands, OPRA/CORA/FOIA Requests), and full court litigation & appeals (**Tier 3 Litigation**: Motions, Pre-Filing Complaints, Reconsideration, Notices of Appeal, Appellate Briefs, Mandamus, and SCOTUS Certiorari), plus an airgapped **Confidential Vault** (encrypted False Claims / whistleblower vault).

Both the desktop window (`OpenSeason.exe`) and the headless CLI/MCP server (`openseason-cli.exe`) call the exact same deterministic Rust engine (`src-tauri/src/`). Neither ever invents facts/citations, practices law, or files/serves documents.

---

## Documentation Map

| Document | Purpose & Audience |
|---|---|
| [EXECUTIVE-SUMMARY.md](EXECUTIVE-SUMMARY.md) | **Start Here:** Full architectural executive summary, 6 Legal OS subsystems, 4 Workbench Pillars, and module inventory |
| [AGENT-INTERFACE.md](AGENT-INTERFACE.md) | **AI Agents & CLI Users:** Complete `--json` CLI reference and `openseason mcp` stdio tool catalog |
| [architecture.md](architecture.md) | **System Architects:** Module-by-module map (<800 lines per file rule), 3-Tier data flow, Multi-Venue Matrix, and Tolling Engine |
| [troubleshooting.md](troubleshooting.md) | **Troubleshooting & Operations:** Step-by-step runbook for diagnosing & fixing build, export, venue validation, PDF, or cloud sync issues |
| [developer_guide.md](developer_guide.md) | **Contributors & Scaling:** How to add new state venues, document kinds, rule packs, Tauri commands, and Svelte 5 components |
| [SYNC-AND-CLOUD.md](SYNC-AND-CLOUD.md) | **Cloud Integration:** JustLegal Cloud (`justlegal.me`) Firebase Auth, additive-only sync, subfolder routing, and confidential airgap |
| [FORMS-PACKS.md](FORMS-PACKS.md) | **Court Forms:** Forms catalog, SHA-256 verification, AcroForm auto-filler (`JDF` / state forms) |
| [RULE-PACKS.md](RULE-PACKS.md) | **Deadline Math:** Holiday-aware deadline packs (`nj-app-div`, `co-mandamus`, `generic-us-appellate`) |
| [cases.md](cases.md) | **Case Folder Layout:** Portable `Documents/JustLegal/Cases/{caseId}/` structure and `case.json` schema |
| [api.md](api.md) | **IPC Reference:** Tauri `invoke` commands and Rust function bindings |
| [TESTING-ON-WINDOWS.md](TESTING-ON-WINDOWS.md) | **Windows Build & QA:** Visual Studio C++ toolchain, `cargo test`, and release binary checklist |
| [../AGENTS.md](../AGENTS.md) | **Mandatory AI Rules:** Hard guardrails for any AI touching a case folder |
| [skills/openseason-motion/SKILL.md](skills/openseason-motion/SKILL.md) | **Agent Skill File:** Fast onboarding flow (`case context` -> `intake` -> `next-steps` -> `check` -> `export`) |
