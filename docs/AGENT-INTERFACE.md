# Agent interface (first-class)

The Offline Case Builder is driven the same way by a person in the window and by an agent on the command line. Both call the Rust engine in `open_season_lib`. The app never files or serves.

Read [AGENTS.md](../AGENTS.md) first. Never invent facts, arguments, or citations.

There is no cloud API and no hosted model. Headless use is local: the `openseason` CLI (`--json`) and an optional stdio MCP server (`openseason mcp`).

## Contract

`--json` prints one envelope per command:

```json
{
  "ok": false,
  "never_files": true,
  "never_serves": true,
  "notice": "...",
  "issues": [{ "field": "fact.1", "message": "..." }],
  "data": {}
}
```

Rules:

- `never_files` and `never_serves` are always true.
- `ok` is true only when `issues` is empty.
- Exit code 0 means the command ran. Read `ok`. A missing file or bad argument is exit 1 plus an envelope whose `issues[0].field` is `cli` or `mcp`.
- An agent should stop and ask the user when a fact has no record cite or a cite is `unverified`. Do not fill those fields yourself.

## CLI

From the repository root (Rust toolchain required):

```bash
npm run cli -- --json case create --path ./tmp-case --title "Example v. Sample County Clerk" --kind reconsideration --pack nj-app-div
npm run cli -- --json draft import ./tmp-case --md fixtures/sample-reconsideration.md
npm run cli -- --json evidence add ./tmp-case --file fixtures/sample-appeal-case/evidence/.gitkeep --description "Dkt. 12 order"
npm run cli -- --json check fact ./tmp-case
npm run cli -- --json check authority ./tmp-case
npm run cli -- --json check deadline ./tmp-case --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09
npm run cli -- --json check all ./tmp-case --pack nj-app-div --rule reconsideration-appellate --trigger-date 2026-01-09
npm run cli -- --json export court-md ./tmp-case
npm run cli -- --json packs list
```

Equivalent without npm: `cargo run --manifest-path src-tauri/Cargo.toml --bin openseason -- --json ...`

### Commands

| Command | What it does |
|---|---|
| `case create --path DIR --title TEXT [--kind KIND] [--pack PACK] [--mode standard]` | Create a portable case folder and `case.json` |
| `case validate DIR` | Caption and layout check |
| `case seal DIR` | Writes `.sealed`. Does not encrypt files |
| `draft import DIR --md FILE` | Import md2court subset into `drafts/draft.md` |
| `evidence add DIR --file FILE [--description TEXT]` | Copy an exhibit into `evidence/` |
| `check fact DIR` | Split the draft into sentences. Does not invent cites |
| `check fact DIR --id N --cite TEXT --exhibit NAME --verified` | Attach a user-supplied cite |
| `check authority DIR` | List cites already in the draft. Status starts `unverified` |
| `check authority DIR --id N --status verified --source URL` | Record a human/agent verification |
| `check deadline DIR --pack PACK --rule RULE --trigger-date YYYY-MM-DD` | Weekend/holiday roll |
| `check signoff DIR` / `--id ID --checked` | Sign-off checklist |
| `check all DIR [--pack --rule --trigger-date]` | Fact + authority + deadline + sign-off |
| `export court-md DIR [--out DIR]` | Court-formatted Word + PDF |
| `export pleading DIR --kind KIND` | Notice, motion, brief, mandamus, complaint |
| `export notice-of-appeal DIR` | Notice of Appeal Word + PDF |
| `draft sections DIR` | List guided prompts for this case kind |
| `draft fill DIR --id ID --text TEXT` | Fill one section with the user's words |
| `evidence index DIR` | Number exhibits Ex. A, Ex. B |
| `evidence link DIR --fact-id N --letter A` | Attach a fact to an exhibit |
| `case seed --path DIR --template lucille` | Fake Lucille Example sample |
| `export packet DIR` | Notice of motion, motion, certification, proposed order, certificate of service |
| `export ics DIR --pack --rule --trigger-date` | Deadline calendar (.ics) |
| `packs validate FILE` | Validate a rule-pack JSON file |
| `gaps add DIR --title TEXT` / `gaps export DIR` | Justice-gap notes |
| `docs playbook` / `docs fields` | Agent playbook and field help |
| `packs list` | Bundled rule packs plus form-pack catalog (sizes first) |
| `packs install STATE [--situation NAME] [--file ZIP]` | Download or copy a forms pack; verify SHA-256; offline afterward |
| `packs remove STATE` / `packs verify [STATE]` | Uninstall or re-check hashes |
| `packs catalog --url URL` | Cache a catalog JSON (JustLegal registry or your host) |
| `guide questions` / `guide find --state --situation [--court] [--trigger-date]` | What do I need to file? Teaching from packs. Never invents rules |
| `docs links` | JustLegal help/resource URLs as data. Never opens a browser |
| `check preflight DIR` | Live readiness audit across caption, docket, filer, deadline, facts, authorities, and packet parts |
| `check transcripts DIR` | Line-index hearing and deposition transcripts and verify draft record citations |
| `export exhibit-packet DIR [--prefix PREFIX] [--start N]` | Assemble Bates-stamped exhibit packet with SHA-256 Table of Exhibits |
| `form inspect FILE` | Inspect PDF AcroForm fields, types, and detect standard court forms |
| `form fill --case DIR FILE [--out OUT]` | Fill official PDF form with case profile data and export filled PDF + JSON |
| `cloud login --email EMAIL --password PASS` | Authenticate with JustLegal Cloud |
| `cloud status` | Check current JustLegal Cloud connection status |
| `cloud pull [--case CASE_ID]` | Pull cloud matters and download associated orders, filings, and evidence |
| `cloud push DIR` | Push local case profile and files to JustLegal Cloud (airgapped cases refused) |
| `cloud logout` | Clear local session tokens |
| `mcp` | Stdio MCP server (same tools) |
| `sync DIR --action backup` | Stub. Standard: `not_implemented`. Confidential: refused |

## MCP (stdio)

`openseason mcp` speaks JSON-RPC 2.0 with `Content-Length` framing. Tools include:
- `check_preflight`: Run comprehensive court readiness pre-flight checklist.
- `check_transcripts`: Index transcripts and cross-reference assertions against record lines.
- `export_exhibit_packet`: Generate Bates-stamped exhibit packet and Table of Exhibits.
- `form_inspect`: Inspect official court PDF forms and list AcroForm fields.
- `form_fill`: Auto-map `case.json` profile values into official court forms.
- `cloud_pull` & `cloud_push`: Autonomous additive sync with JustLegal Cloud (airgap protected).
- Motion path tools: `case_create`, `draft_import`, `check_fact`, `check_authority`, `check_deadline`, `check_signoff`, `export_pleading`, `export_court_md`.
- Form packs & guide: `packs_install`, `packs_verify`, `guide_find`, `help_links`.

`guide_find` and `help_links` return `help_links` as data and never open a browser.

Each `tools/call` result is a text block whose JSON is the same envelope as `--json`. The server is offline. It does not talk to a cloud model host.

## Example end-to-end run

```bash
bash examples/agent-e2e.sh
```

That script creates a fake NJ reconsideration case, imports Markdown, records an exhibit, runs every check stage, asserts the JSON envelope, and exports Word and PDF. It uses Jordan Example only.

Library tests that lock the same path: `agent_e2e_import_check_deadline` and `mcp_tools_create_import_check_export`.

## What the checker will not do

- Invent a record cite or a case citation
- Mark a cite `verified` unless a human (or an agent the user directed) set that status with a source
- File, e-file, email, mail, or serve anyone
- Open Josh's personal case folders unless the user points the CLI at a local path they own
