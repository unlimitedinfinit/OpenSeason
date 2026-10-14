# Roadmap

## This run (foundation)

- [x] Audit OpenSeason and record that JustLegalAid source was not reachable
- [x] Standard and Confidential modes enforced in Rust, with sync-refusal tests
- [x] Portable case folder and fill-once `case.json`
- [x] Notice of Appeal to Word and PDF
- [x] `AGENTS.md` and `openseason` CLI
- [x] Home screen with document types plus Confidential mode
- [x] Remove debug leftovers (`api_debug.log`, `src-tauri/ok`)

## Next run (planned)

These items are intentionally not built here:

1. New-complaint intake interview
2. Evidence intake that links each fact to a source file
3. Jurisdiction finder
4. Merit / readiness checklist
5. Court packet and local-rules fetch into `court-rules/` (store source URL and date)
6. Bundle output for a filing packet
7. Opt-in shared court-rules library
8. Real justlegal.me pull/publish for **standard** cases only, behind `sync::request_sync`

## Still open from the old vault roadmap

- [ ] Disclosure PDF should use live SQLite totals, not leftover mock values in one dashboard path
- [ ] Stronger `.osb` integrity checks
- [ ] Optional SQLCipher for `metadata.db`
- [ ] Produce and smoke-test Windows `.msi` / `.exe` and macOS `.dmg` on real builder machines
