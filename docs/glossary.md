# Glossary

| Term | Definition |
|---|---|
| **Argon2id** | The state-of-the-art key derivation function used to convert the user's master password into a secure 32-byte session key. |
| **Bug Out Bag** | The protocol for compressing and packaging an entire case vault into a portable, encrypted `.osb` ZIP file. |
| **False Claims Act** | A federal law (31 U.S.C. §§ 3729-3733) that imposes liability on individuals and companies who defraud governmental programs, offering financial bounties to whistleblowers. |
| **Standard case** | An unencrypted, portable case folder under Documents/JustLegal/Cases. May use account sync later. |
| **Confidential mode** | The Open Season vault. Desktop seal encrypts case files after a password check (except `metadata.db` and `court-rules`). Legal Airlock is required in the app. Sync, backup, and publish are refused when `.sealed` exists. CLI seal writes the marker only and does not encrypt. |
| **Fill-once profile** | The `case.json` record of court, docket, parties, and filer contact reused on every document. |
| **Notice of Appeal** | The first finished document template. Formats the user's own notice text. |
| **Hunting Blind** | The core design philosophy of Open Season: local-first execution, metadata scrubbing, and zero cloud synchronization. |
| **Kill Kit** | The report compilation framework that takes local SQLite data and evidence and compiles a PDF disclosure statement using Typst. |
| **Qui Tam** | A legal provision allowing private citizens to file lawsuits on behalf of the government and receive a percentage of any recovered funds. |
| **Tauri** | A framework for building desktop applications using a Svelte web frontend and a Rust system backend. |
| **Typst** | A modern markup-based typesetting compiler used to generate professional PDF documents natively. |
| **XChaCha20Poly1305** | The symmetric authenticated encryption cipher used to encrypt case evidence files using a 192-bit nonce. |
| **Zeroize** | A security practice (and Rust crate) used to securely clear sensitive bytes and keys from computer memory after use. |
