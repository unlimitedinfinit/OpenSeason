# JustLegal Cloud Sync & Airgap Architecture

This document specifies the technical design, security rules, and interface reference for the **JustLegal Cloud Sync Hub** connecting local OpenSeason desktop and CLI instances to the `justlegal.me` platform (`justlegalapp1` Firebase infrastructure).

---

## 1. Core Principles & Privacy Guarantees

### Strict Local Autonomy
- **Zero Account Prerequisite:** OpenSeason is 100% functional offline without an account. All drafting, AcroForm filling, Bates stamping, court checks, and PDF exports run strictly on the local machine.
- **Additive-Only, Non-Destructive Merge:** Cloud sync never overwrites local drafts, notes, or case configurations. If a remote document matches an existing local file's SHA-256 hash, it is safely skipped. If a case already exists locally, remote cloud matter metadata (`cloud_matter_id`, updated dates) is attached non-destructively.

### Hard Confidentiality Airgap
- **The Rule:** Any case whose `mode` is `confidential`, or which has a `.sealed` marker, is **strictly airgapped**.
- **Enforcement:** `src-tauri/src/sync.rs` unconditionally checks `case_profile.mode == CaseMode::Confidential` and `crate::seal::is_dir_sealed(case_dir)`. If detected, sync aborts immediately with exit code 1 or an IPC error:
  ```json
  {
    "ok": false,
    "issues": [{ "field": "confidentiality", "message": "Confidential or sealed cases are strictly airgapped and cannot sync to the cloud." }]
  }
  ```
- **Integrity Guarantee:** Deleting `.sealed` or tampering with `case.json` does not bypass the airgap if either marker persists.

### Firebase Web API key (not in source)

The Firebase Web API key is **not** stored in this repository. Resolve it at runtime, first match wins:

1. `OPENSEASON_FIREBASE_API_KEY` environment variable
2. User config file outside the repo: `%LOCALAPPDATA%/JustLegal/config.json` (or the equivalent Application Support / `.local/share` path) with `{ "firebase_api_key": "..." }`
3. Compile-time `option_env!("OPENSEASON_FIREBASE_API_KEY")` from a gitignored `.env` (see `.env.example` and `src-tauri/build.rs`)

If none of those are set, login and sync are disabled with a clear message. Offline features keep working.

---

## 2. Authentication & Token Management

OpenSeason authenticates directly with Firebase Auth REST endpoints using HTTPS:

1. **Login:** Authenticates user email and password against `identitytoolkit.googleapis.com/v1/accounts:signInWithPassword`.
2. **Session Storage:** Returns an encrypted session containing `id_token`, `refresh_token`, `uid`, `email`, and `expires_at`.
3. **Session Cache:** Persisted locally in `$LOCALAPPDATA/JustLegal/cloud_session.json` (on Windows: `C:\Users\<User>\AppData\Local\JustLegal\cloud_session.json`).
4. **Token Refresh:** If the ID token expires (tokens are typically valid for 3600 seconds), OpenSeason automatically calls `securetoken.googleapis.com/v1/token` with the refresh token to obtain a fresh ID token without prompting the user.
5. **Logout:** Clears the local session cache file immediately.

---

## 3. Subfolder Routing & Schema Mapping

OpenSeason maps unified `justlegal.me` record collections into local deterministic folders:

| justlegal.me Record `type` | Local OpenSeason Subfolder | Example Artifacts |
|---|---|---|
| `Order` | `orders/` | Signed Court Orders, Fee Waiver Approvals, Scheduling Orders |
| `Motion`, `Pleading`, `Complaint` | `filings/` | Notice of Motion, Proposed Orders, Briefs, Complaints |
| `Evidence`, `Exhibit`, `Transcript`, `Other` | `evidence/` | Hearing Transcripts, Exhibits A-Z, Photos, Contracts |
| Court-issued AcroForms | `forms/` | Official court templates (e.g. Colorado JDF 205, JDF 1911) |
| System exports | `exports/` | Generated court PDFs, Bates-stamped packets, .docx files |

### Firestore Collections Queried
- `/cases/{caseId}`: Root case collection matching `userId == session.uid` or cases shared with user.
- `/cases/{caseId}/records`: Subcollection containing filing records, uploaded attachments, and metadata.
- `/users/{uid}/cases/{caseId}`: Fallback lookup for nested user matter hierarchies.

---

## 4. Desktop UI (Human Interface)

### Sync Hub Modal
Accessible via the **☁ JustLegal Cloud** button in the app navigation and on any standard Case Detail header:
- **Authentication Card:** Sign in with existing `justlegal.me` credentials or view connected account status.
- **Pull All Cloud Matters:** One-click pull that queries all matters from Firestore, merges them into `Documents/JustLegal/Cases/`, and routes attachments to `orders/`, `filings/`, and `evidence/`.
- **Push Active Case:** Syncs the currently opened case profile and its local files to JustLegal Cloud.
- **Airgap Notice:** On confidential cases, the cloud button is disabled and displays an "Airgapped (Confidential)" security shield.

### Status Indicators
- **Case Dashboard:** Cases synced with the cloud display an indigo `☁ Synced` pill with the shortened cloud matter ID.
- **Case Detail Header:** Displays the synchronized remote matter ID and one-click cloud controls.

---

## 5. Agent Interface (CLI & MCP)

The CLI and MCP stdio server expose full cloud management to automated scripts and AI subagents.

### CLI Commands

```bash
# Check cloud connection status
openseason --json cloud status

# Authenticate with JustLegal Cloud
openseason --json cloud login --email user@example.com --password secret

# Pull all cases or a single case from JustLegal Cloud
openseason --json cloud pull
openseason --json cloud pull --case IPn624wPtianEjZDcSFD

# Push local case to JustLegal Cloud
openseason --json cloud push Documents/JustLegal/Cases/anderson-coa

# Log out and invalidate local session token
openseason --json cloud logout
```

### MCP Tools
- `cloud_pull`: `{ case_id?: string }` — Pulls cloud matters and downloads associated documents.
- `cloud_push`: `{ case_dir: string }` — Uploads local profile and artifacts to JustLegal Cloud.

---

## 6. Troubleshooting & FAQ

**Q: What happens if the network drops during sync?**  
A: Downloads and uploads are transactionally idempotent. Any partially downloaded files are not committed until verified. Re-running `cloud pull` resumes non-destructively.

**Q: Will cloud sync overwrite my edited Markdown drafts or motion text?**  
A: Never. Local `drafts/`, `case.json` user motion fields, and local custom files are preserved. Only newly downloaded cloud orders/evidence are added to their respective subfolders.

**Q: Can I use OpenSeason on multiple computers with cloud sync?**  
A: Yes. Log into the same JustLegal account on both machines and run `cloud pull`. The additive merge synchronizes your matters and evidence across devices.
