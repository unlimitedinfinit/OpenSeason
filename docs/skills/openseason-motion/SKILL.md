# Skill: draft a motion with OpenSeason

Use this when the user wants a local, offline motion, appeal paper, or check/export through OpenSeason.

## Rules

- Read AGENTS.md. Never invent facts, arguments, or citations.
- Call `playbook` (MCP) or `openseason --json help playbook` first.
- If a tool returns `ok: false`, follow `issues[].message`. It says how to fix the field. Do not invent a cite to clear an issue.
- Never file, e-file, email, mail, or serve.

## Tools

Prefer MCP `openseason mcp` or `npm run cli -- --json ...`. Manifest: `templates/mcp.json`.

If the user does not know what to file: `guide_questions` then `guide_find`. Use only pack forms and cites. Mark `needs_verification` items. `help_links` is data; never open a browser.

Full motion path: case_create or case_seed -> draft_sections -> draft_fill (user words only) -> evidence_add -> evidence_index -> check_fact -> evidence_link -> check_authority -> check_deadline -> export_packet.

Fake sample: Lucille Example v. Sample County Board. Do not use real litigants in fixtures.
