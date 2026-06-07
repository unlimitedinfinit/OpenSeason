# Risks

## Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Database Corruption | Low | High | Standardized exports to `.osb` files so users can restore case backups easily. |
| Typst version mismatch | Medium | Medium | Pinning the `typst` crate version in `Cargo.toml`. |
| Svelte 5 Rune API updates | Low | Low | Keep dependencies updated and run local frontend test suites. |

## Legal / Compliance Risks
- **Defamation & Defective Disclosures**: Users might file false or defamatory disclosure statements, thinking they are legally protected. The app enforces a mandatory **Legal Airlock** screen to make sure they know it is not legal advice and that they are solely responsible.

## Market Risks
- **Platform Dependency (Tauri/Vite)**: Open Season relies on the Tauri v2 framework. Any major security issues or abandonment of Tauri could force a rewrite of the shell in Swift/C#.

## Operational Risks
- **Key Loss Lockout**: Because keys are derived in-memory and never stored, if a user forgets their master password, they are locked out of their database forever. The UI must clearly emphasize this limitation.

## Security Risks
- **Memory Dump Leaks**: If the OS swaps memory containing raw passphrase bytes to disk, the key could be recovered. We mitigate this by using the `Zeroize` crate on the session key struct to wipe memory immediately upon drop.
