# Topology

```
OpenSeason/
├── AGENTS.md                 Instructions for any AI using the builder
├── README.md
├── fixtures/sample-appeal-case/   Fake Notice of Appeal case
├── templates/court-rules/    Formatting data (generic default)
├── docs/                     Human documentation
├── src/                      Svelte 5 UI
│   ├── lib/components/       Airlock, vault, notice banner, glass panels
│   ├── lib/types/case.ts
│   └── routes/
│       ├── +page.svelte      Home (document types + Confidential)
│       ├── case/             Standard case profile and appeal export
│       ├── confidential/     Open Season vault door
│       └── hunt/[id]/        Confidential hunt workspace
└── src-tauri/                Rust
    ├── src/bin/openseason.rs CLI
    ├── src/case_profile.rs
    ├── src/sync.rs
    ├── src/documents.rs
    ├── src/court_rules.rs
    ├── src/case_commands.rs
    └── src/crypto.rs, db.rs, pdf.rs, bundle.rs, usaspending.rs
```
