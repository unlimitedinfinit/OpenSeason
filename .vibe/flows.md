# Flows

## 1. Onboarding & Hunt Setup
User launches Open Season
  ↓
Sees Legal Airlock modal → must type "I UNDERSTAND" to unlock
  ↓
Prompted for master password on the Vault Launcher screen
  ↓
Enters password → click "Unlock Vault" → derives session key in memory
  ↓
Sees Active Operations dashboard → click "New Operation"
  ↓
Wizard opens prompting for target contractor name

## 2. Target Scouting & Evidence Lockup
User enters target name in the Hunt Wizard
  ↓
Click "Scout Target" → app queries USAspending API
  ↓
Displays list of contracts/grants matching target name
  ↓
User selects target → saves target to the active operation
  ↓
Navigates to Case Vault → drags & drops files into Evidence section
  ↓
Tauri strips EXIF/chunk metadata from files, encrypts them via XChaCha20Poly1305, and saves them to the hunt vault
  ↓
User adds Timeline events, Parties, and drafts Complaint Sections

## 3. Disclosure Statement Generation & Bug Out Export
User goes to Case Vault → click "Report"
  ↓
Tauri compiles the local database facts, events, and evidence hashes using Typst PDF engine
  ↓
Generates a PDF disclosure statement and saves it locally in the hunt folder and user's Downloads directory
  ↓
Click "Export" → enters export destination path (e.g. flash drive)
  ↓
Tauri compresses the encrypted vault and sqlite db into a `.osb` file
  ↓
User copies the `.osb` file for secure transport or backup

## 4. Key Recovery & Lockout Error
User enters incorrect password on Vault Launcher
  ↓
Vault Launcher shows error "Invalid Key"
  ↓
If password is lost, user must click "New Vault" which purges cache and resets the local directories
  ↓
Since keys are derived in-memory and never stored, data recovery is impossible without the correct password
