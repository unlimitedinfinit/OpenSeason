# Forms packs and the filing finder

OpenSeason does **not** vendor Josh's ~7 GB JustLegal form tree. The app ships a catalog (an index) plus teaching text inside rule packs. Official PDFs download into `Documents/JustLegal/Packs/` only when the user installs a state or situation.

This is legal information, not legal advice. The app never files or serves.

## What we found on JustLegal

Public GitHub under `unlimitedinfinit` has no `*justlegal*` forms repo (JustLegalAid and the website repo were not readable). The live site documents the catalog in [justlegal.me/llms.txt](https://justlegal.me/llms.txt) and publishes a machine-readable registry:

| URL | What it is |
|---|---|
| `https://justlegal.me/registry/index.json` | Index: 22,936 catalogued forms, 16,023 mirrors, 51 states, 94 districts (generated 2026-09-24) |
| `https://justlegal.me/registry/states/NJ.json` | Per-state file: `forms[]`, `packets[]`, `gaps[]` |
| `https://justlegal.me/mirrors/files/{id}.pdf` | Hosted unmodified mirrors with `sha256` and `bytes` |
| `https://justlegal.me/court-forms` | Human directory (SPA) |

JustLegal form objects already have the fields this pack format uses: `id`, `title`, `formNumber`, `appliesToCourt`, `task` (situation), `sourcePageUrl`, `directFileUrl`, `mirror.{url,sha256,bytes}`, `verified.{status,checkedAt,bytes}`. Task ids include `appeal`, `fee-waiver`, `eviction`, `general-motion`, and the rest of the court-forms taxonomy.

OpenSeason adapts that JSON (`forms::justlegal_state_to_catalog`) and adds **teaching** (`what_it_is`, `when_used`, `common_mistakes`, `learn_more`) plus `citation` / `citation_status` (`pack_source` or `needs_verification`). Teaching lives in rule packs so a new state is data.

The fixture catalog uses `https://example.test/` URLs on purpose. Do not treat those as court downloads.

## Catalog shape (`openseason.forms-catalog/v1`)

```json
{
  "schema": "openseason.forms-catalog/v1",
  "packs": [
    {
      "id": "nj-appeal-fixture",
      "state": "NJ",
      "court": "…",
      "situations": ["appeal"],
      "title": "…",
      "size_bytes": 120,
      "download_url": "https://example.test/packs/nj-appeal-fixture.zip",
      "local_dir": "packs/nj-appeal",
      "sha256": "",
      "rule_pack_id": "nj-app-div"
    }
  ],
  "forms": [
    {
      "id": "fixture-nj-appeal-kit",
      "state": "NJ",
      "court": "…",
      "situation": "appeal",
      "title": "…",
      "size_bytes": 120,
      "download_url": "https://example.test/forms/…",
      "sha256": "…",
      "citation_status": "needs_verification"
    }
  ]
}
```

`sources[]` may be `http`, `github_release` (`repo` + `tag` → `https://github.com/{repo}/releases/download/{tag}/{asset}`), `s3` / R2 (`base_url` + key), or `dir` (local unpacked pack).

## Installer

```bash
npm run cli -- --json packs list
npm run cli -- --json packs install NJ --situation appeal --file fixtures/forms-catalog/packs/nj-appeal
npm run cli -- --json packs verify NJ
npm run cli -- --json packs remove NJ
```

List shows `size_label` before download. Install copies into `Documents/JustLegal/Packs/{id}/` (or `OPENSEASON_PACKS_ROOT`), writes `pack.json` with per-file SHA-256, and works offline afterward. Network installs of `example.test` URLs are refused. To cache a live JustLegal index (JSON only, not the PDFs):

```bash
npm run cli -- --json packs catalog --url https://justlegal.me/registry/index.json
# or a state file you already saved:
npm run cli -- --json packs catalog --file /path/to/NJ.json
```

## What do I need to file?

```bash
npm run cli -- --json guide questions
npm run cli -- --json guide find --state NJ --situation reconsideration --trigger-date 2026-01-09
```

MCP: `guide_questions`, `guide_find`. The result includes `forms`, `steps` (each with teaching), `deadline` only when the pack already names one, `unverified[]` when a cite is not `pack_source`, `help_links`, and `stuck` when nothing matches. The finder will not invent a notice-of-appeal day count for NJ; that deadline is not in the bundled pack.

## Get legal help / Find resources

Configurable file: [`templates/justlegal-links.json`](../templates/justlegal-links.json).

The window Help menu and the finder show **Get legal help**, **Find resources**, and the **online forms database**. A confirmation always appears first:

> This will open your web browser to JustLegal's online resources. You need an internet connection. Continue?

CLI/MCP (`docs links`, `help_links`) return the same URLs as data and **never** open a browser. `opens_browser` is always false.

Verified on 2026-10-14:

- `https://justlegal.me/find-legal-help`
- `https://justlegal.me/resources/knowledge-hub`
- `https://justlegal.me/resources/guides`
- `https://justlegal.me/court-forms`
- `https://justlegal.me/registry/index.json` (application/json)

## Hosting a real pack

1. Put official files in a folder with `pack.json` (rule pack + `forms` + `situations`).
2. Run `python3 scripts/build-forms-pack.py path/to/folder --out dist/nj-appeal.zip`.
3. Host the zip on GitHub Releases, Cloudflare R2, or any HTTPS CDN.
4. Publish a catalog JSON whose `download_url` and `sha256` match that zip.
5. Point the app at that catalog (`packs catalog --url …`). Do not commit the PDFs.

## Builder

```bash
python3 scripts/build-forms-pack.py fixtures/forms-catalog/packs/nj-appeal --out /tmp/nj-appeal.zip
```

Prints size, SHA-256, and a catalog snippet.
