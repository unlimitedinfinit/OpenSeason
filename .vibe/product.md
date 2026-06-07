# Product

## Product Vision
To empower any citizen to securely compile evidence and expose multi-million dollar government contractor fraud, securing False Claims Act bounties without fear of data leakage or exposure.

## Value Proposition
Unlike generic notes apps or cloud-connected legal software, Open Season provides a local-first, zero-trust workstation specifically optimized for Qui Tam (whistleblower) litigation. It strips tracking data, compiles evidence hashes, and formats professional disclosure statements locally.

## What Drives Signups
- **Whistleblower Paydays**: The False Claims Act rewards whistleblowers with 15% to 30% of recovered funds (often millions of dollars).
- **Fear of Retaliation**: Insiders need to gather evidence quietly without storing sensitive information on company servers or insecure personal cloud systems.
- **Complexity of Filings**: Designing a valid disclosure statement that meets strict legal standards is difficult without a structured, step-by-step checklist.

## What Drives Retention
- **Multi-Vault Cases**: Users can track several targets at once, keeping all case assets isolated and structured.
- **Export Portability**: The ease of packing up a case into a `.osb` file and carrying it on an encrypted USB drive.
- **Zero-Footprint**: Peace of mind knowing that closing the app leaves zero encryption keys on disk.

## What Nobody Uses
- **Direct PDF Previews**: Users typically just download and compile the PDF to view locally rather than attempting to view long PDFs inside the Tauri container.

## Where Users Get Confused
- **Passphrase Loss**: If a user forgets their master password, they lose access to their vaults forever, which can cause panic. The app needs clear warnings.
- **Downloads Directory**: Finding where the compiled reports go on Windows/Mac (they go to the system Downloads folder).

## Feature Priorities

| Priority | Feature | User Problem It Solves |
|---|---|---|
| High | Actual database reporting | Ensures that generated reports show real evidence counts and contract values instead of mock totals. |
| Medium | In-App PDF viewer | Allows users to verify their disclosure document formatting before exporting. |
| Low | Local target notes template | Provides users with standard interview templates for witnesses and targets. |
