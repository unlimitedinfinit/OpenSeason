# Purpose

## What is this?
Open Season is a local-first, zero-trust desktop application that empowers citizens to investigate, track, and draft disclosure statements for government fraud claims. Built with Tauri and Svelte, it acts as a secure "hunting blind" where users can compile evidence and generate professional legal reports locally on their machine.

## Who is it for?
* **Whistleblowers & Insiders**: Citizens or employees with direct knowledge of government contractor fraud who need to securely organize evidence.
* **Pro Se Investigators**: Individuals drafting Qui Tam (whistleblower) complaints under the False Claims Act.
* **Advocacy Groups & Journalists**: Organizations investigating government spending irregularities.

## What problem does it solve?
Before Open Season, whistleblowers faced high risks of accidental leaks, metadata traces (like EXIF data in photo evidence), and security compromises. Cloud storage services can be subpoenaed or breached, which can ruin a whistleblower's claim under the False Claims Act's "first-to-file" rule and compromise their anonymity. Open Season solves this by offering a local-only, metadata-scrubbed, cryptographically locked workstation.

## What does success look like?
A whistleblower launches the app, passes the Legal Airlock, enters their master password to unlock their session, checks a contractor's awards via the USAspending API, imports evidence images (which are automatically stripped of EXIF metadata and encrypted), logs timeline events, drafts a complaint, and compiles a professional PDF disclosure statement. They then export a single encrypted `.osb` backup to a flash drive.

## What does it explicitly NOT do?
- **No Cloud Storage or Syncing**: Open Season does not connect to the cloud or synchronize vaults over the internet.
- **No Legal Representation or Advice**: The application does not provide legal advice, representation, or guarantee False Claims Act rewards.
- **No Private Key Recovery**: Since keys are derived from the master password and never stored, there is no way to recover a vault if the password is lost.
