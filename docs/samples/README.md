# Sample court PDFs (fixture data only)

These files are **fake**. Names are Jordan Example and Sample County Clerk. Docket numbers are made up. Do not copy them into a real filing.

| File | What it shows |
|---|---|
| `nj-appellate-motion-jordan-example.pdf` | New Jersey Appellate Division boxed caption: appellate docket `A-01000-24`, trial-court docket `L-01000-24`, `COURT USE ONLY`, filer address and email, R. 1:4-4(b) certification above the signature, certificate of service on page 2 |
| `co-appeals-motion-jordan-example.pdf` | Colorado Court of Appeals boxed caption: case no. `24CA1000`, lower court Denver District Court `23CV1000`, `COURT USE ONLY`, user-editable conferral and declaration (`needs_verification`), signature, certificate of service on page 2 |
| `before-centered-stack.png` | What the old centered export looked like (raw markdown leaked; Arial; no page-1 footer) |
| `nj-page-1.png` / `nj-page-2.png` | Raster of the NJ sample. Page 2 is the certificate of service |
| `co-page-1.png` / `co-page-2.png` | Raster of the CO sample. Page 2 is the certificate of service |
| `ui-dashboard.png` | Calm dashboard: one Next-up card, no invented deadline |
| `ui-draft-preview.png` | Three-click Draft to PDF with preview (NJ sample) |
| `ui-draft-exported.png` | After Export Word and PDF |
| `ui-forms-packs.png` | Forms packs list with Install when a local fixture exists |

Source Markdown:

- `fixtures/sample-reconsideration.md`
- `fixtures/sample-co-reconsideration.md`

Rebuild:

```bash
npm run cli -- --json case create --path /tmp/os-nj --title "Jordan Example v. Sample County Clerk" --kind reconsideration --pack nj-app-div
npm run cli -- --json draft import /tmp/os-nj --md fixtures/sample-reconsideration.md
npm run cli -- --json export court-md /tmp/os-nj --out docs/samples
```

OpenSeason never files or serves. Review every line before filing.
