# GrokFirebase-HELP: OpenSeason <-> justlegal.me cloud sync reference

Written 2026-10-15 from read-only inspection of Firebase (via MCP) and this repo.
"Observed" = seen in live data or code. "Inferred" = not verified; confirm before relying on it.
No legal record contents are reproduced. Example values are redacted.

## 1. Which Firebase project

| Project id | Display name | Role |
|---|---|---|
| `justlegalapp1` | Just Legal | **Hosts justlegal.me cases** (observed: root `cases` collection, case `IPn624wPtianEjZDcSFD` lives here). Labelled production. |
| `newjosh-consent` | NewJosh Consent | newjosh.com contact/newsletter forms. Not cases. |
| `voteready-me` | VoteReady | Unrelated. |

Firestore database: `(default)`. Web SDK config (apiKey, authDomain, storageBucket) lives in the justlegal.me web app source / Firebase console > Project settings > Your apps. apiKey is intentionally omitted here.

## 2. Collection paths (observed)

Cases are **root-level**: `/cases/{caseId}`, owner field `userId` (string, Firebase Auth uid, 28 chars, e.g. `Yxl9...redacted`).
Confirmed with real doc `cases/IPn624wPtianEjZDcSFD`. Not nested under users.

Root collections in `justlegalapp1`: `attorneys, bounties, cases, deadlines, extractedText, fancut_comments, fancut_submissions, users`.
`/users/{uid}` subcollections: `goals, presets, valueAssessments` (no cases there).
Root `deadlines` and `extractedText` were not schema-inspected (inferred: likely keyed by case or doc id; check before using).

### /cases/{caseId} fields (observed)

| Field | Firestore type | Example / values seen |
|---|---|---|
| userId | string | owner uid |
| title | string | "Plaintiff v. Defendant" |
| description | string | free text |
| caseType | string | Civil, Criminal, Employment, Family, Other |
| status | string | Open, Active, active, Pending, Pending Docket, Drafting, About to File, closed (**mixed case**) |
| caseNumber | string | "L-000000-25" |
| venue | string | "Middlesex Superior" |
| jurisdiction | string | "Middlesex Superior Court" |
| state | string | "New Jersey" |
| tags | array<string> | |
| progress | integer | 0-100 (inferred range) |
| createdAt | timestamp | `2025-12-05T01:27:52.511Z` (RFC3339 UTC, ms) |
| updatedAt | timestamp | same format |

No confidential / sealed / visibility field exists on any of the 24 case docs checked (masked query). Fields may be absent on some docs; treat all as optional.

### Subcollections under /cases/{caseId} (observed)

`documents`, `events`, `parties`, `counts`.

**/cases/{caseId}/documents/{docId}** (the "records")

| Field | Type | Notes |
|---|---|---|
| title | string | |
| description | string | |
| docType | string | **the record type**, see section 4 |
| originalFilename | string | e.g. `XXXX.pdf` |
| contentType | string | all seen: `application/pdf` |
| sizeBytes | integer (Firestore int64, JSON string in REST) | |
| relevanceDateText | string | free-text date, not a timestamp |
| tags | array<string> | |
| uploadedBy | string | uid |
| createdAt, updatedAt | timestamp | |

**No storage path, download URL, or hash field was found** on any document record (14 docs in one case fully read; mask queries on two more cases). See section 5.

**/cases/{caseId}/events/{id}**: title (string), description (string), eventType (string; seen "Original Incident", "Motion Filed"), dateText (string, free text), createdAt, updatedAt (timestamp).

**/cases/{caseId}/counts/{id}**: countType (string), customType (string), description (string), reliefSought (string), createdAt, updatedAt.

**/cases/{caseId}/parties/{id}**: not fully read (MCP list failed on null values). Fields seen in the error: `email`, `phone`, `notes` (nullable string). Others unknown. Inferred: name/role fields exist.

## 3. Mapping cloud <-> case.json

Local `case.json` is `CaseProfile` in `src-tauri/src/case_profile.rs`. It has **no cloud id field today**; add `cloud_matter_id: Option<String>` with `#[serde(default)]` (and optionally `cloud_project`, `cloud_synced_at`).

| Cloud (`cases/{id}`) | case.json | Direction / note |
|---|---|---|
| document id | `cloud_matter_id` (new) | key for matching |
| userId | (none) | set from signed-in uid on push |
| title | `title` | fill if local empty |
| caseNumber | `docket_number` | fill if local empty |
| venue / jurisdiction | `court.name` | jurisdiction preferred; never overwrite |
| state | (none; inferred: use for `rule_pack_id` hint, NJ -> `nj-app-div`) | |
| caseType | `document_kind`? (inferred, weak) | do not auto-map; store in notes |
| description, tags, status, progress | (no field) | keep cloud-only or add `cloud_meta` object |
| createdAt / updatedAt | `created_at` / `updated_at` | RFC3339 both sides |
| events[] | `timeline[]` (TimelineEvent: id,title,date,notes) | id=`cloud:{eventId}`, title, date=dateText, notes=description |
| documents[] | files in subfolders (section 4) | |
| (none) | `mode`, `sealed_at` | local only; gate sync |

## 4. docType -> local subfolder routing (real values)

Values observed across 3 of Josh's cases (49 docs): `Order, Motion, Answer, Notice, Evidence, Petition, Complaint, Pleading, Brief, Court Filing, Service of Process, Receipt, Lease`, and empty string `""`.
It is a free string, not an enforced enum (inferred from the variety and empty value). Match case-insensitively and trim.

| docType | Local folder (CASE_SUBDIRS) |
|---|---|
| Order | `orders/` |
| Motion, Answer, Notice, Petition, Complaint, Pleading, Brief, Court Filing, Service of Process | `filings/` |
| Evidence, Receipt, Lease | `evidence/` |
| "" or anything unknown | `evidence/_unsorted/` (inferred choice; never guess filings) |

Reverse (push): `orders/` -> Order, `filings/` -> Court Filing (unless the user tagged a type), `evidence/` -> Evidence. Never push `drafts/`, `court-rules/`, `exports/`.

## 5. Cloud Storage

Not observed. The documents carry no path field, and the MCP Storage tool only returns download URLs for a known path, so the pattern could not be enumerated read-only.
- Bucket (inferred): default bucket of `justlegalapp1`, label `firebase/storage-default-bucket: created`. Usually `justlegalapp1.firebasestorage.app` (newer) or `justlegalapp1.appspot.com`. Read `storageBucket` from the web app's SDK config.
- Path (inferred, unverified): web code likely builds it from ids, e.g. `cases/{caseId}/documents/{docId}/{originalFilename}` or `users/{uid}/...`. **Grep the justlegal.me web source for `ref(storage,` / `uploadBytes` to confirm before coding.**
- Hashes: none stored. Compute SHA-256 locally after download; store mapping in a local sidecar (below). Storage object metadata has `md5Hash` (base64) which can be used as a cheap remote change check.

## 6. Merge rules (additive only)

1. Gate first: call `sync::request_sync_for_dir`. If `.sealed` marker exists, `mode == confidential`, or `sealed_at` is set, stop. Never send anything. No cloud flag exists for confidential, so also skip cloud cases whose tags contain "sealed"/"confidential" (inferred safeguard).
2. Match a case by `cloud_matter_id` in case.json. No id: offer link-or-create; never auto-link by title.
3. Match files by SHA-256. Keep `.openseason/sync.json` per case: `{ docId, sha256, local_path, cloud_updateTime, md5Hash }`.
4. Pull: new docId -> download into routed folder. If filename exists with a different hash, write `name (cloud YYYY-MM-DD).pdf` beside it.
5. Push: new local hash not in sync.json -> upload object + create a new `documents` doc. Never update or delete existing cloud docs.
6. Same hash anywhere = already synced; just record mapping.
7. Never delete or overwrite on either side. Field conflicts in case.json: keep local, record cloud value in `cloud_meta`.
8. Firestore writes use `currentDocument.exists=false` precondition so a create can never clobber.

## 7. REST endpoints for a Rust client

Auth (Identity Toolkit, `key=` = web apiKey):
- Email/password: `POST https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key=API_KEY` body `{email,password,returnSecureToken:true}` -> idToken, refreshToken, localId (uid), expiresIn (3600).
- Google: desktop OAuth (loopback redirect + PKCE) to get a Google id_token, then `POST .../v1/accounts:signInWithIdp?key=API_KEY` body `{postBody:"id_token=...&providerId=google.com", requestUri:"http://localhost", returnSecureToken:true}`.
- Refresh: `POST https://securetoken.googleapis.com/v1/token?key=API_KEY` form `grant_type=refresh_token&refresh_token=...` -> id_token, refresh_token.
- Store refreshToken in OS keychain (keyring crate), never in case folders.

Firestore (header `Authorization: Bearer {idToken}`), base `https://firestore.googleapis.com/v1/projects/justlegalapp1/databases/(default)/documents`:
- List my cases: `POST {base}:runQuery` with structuredQuery from `cases`, where `userId == uid`, orderBy `createdAt desc` (matches the existing index).
- Get case: `GET {base}/cases/{caseId}`; list subs: `GET {base}/cases/{caseId}/documents?pageSize=100`.
- Create doc: `POST {base}/cases/{caseId}/documents?documentId={id}&currentDocument.exists=false`.
- REST values are typed wrappers (`stringValue`, `integerValue` as string, `timestampValue`, `arrayValue.values`, `nullValue`).

Storage (Firebase Storage REST, same Bearer token):
- Download: `GET https://firebasestorage.googleapis.com/v0/b/{bucket}/o/{urlencoded path}?alt=media`
- Metadata: same URL without `alt=media` (gives md5Hash, size, updated).
- Upload: `POST https://firebasestorage.googleapis.com/v0/b/{bucket}/o?name={urlencoded path}` with file bytes and Content-Type.

## 8. Security rules

**Not read for justlegalapp1.** The MCP rules tool only reads the CLI's active project, which is `newjosh-consent` (its rules only cover `inquiries` and `newsletter`, create-only). Switching the active project was not done to keep this read-only.
Inferred expectation (verify in console > Firestore > Rules and Storage > Rules): owner check `resource.data.userId == request.auth.uid` on `cases/{id}` and probably `get(/cases/$(caseId)).data.userId` for subcollections. A REST client using the user's ID token gets exactly the same permissions as the web app; no admin bypass. Queries must include `where userId == uid` or the list is denied even if every doc would pass.

## 9. Auth providers

Not directly observed (no read-only MCP tool lists enabled providers; auth_get_users was not needed). Uid shape: 28-char Firebase uid. Inferred: email/password and/or Google, check console > Authentication > Sign-in method.

## 10. Indexes (observed)

One composite index on `cases`: `userId ASC, createdAt DESC, __name__ DESC` (READY). That covers "my cases newest first". Subcollection reads by parent path need no index. A collection-group query on `documents` with filters would need a new index (none exists).

## 11. Gotchas

- `status` values are inconsistent in case (`Active` vs `active`, `closed`). Normalize when displaying; never write back.
- `docType` can be empty and is free text.
- Dates in `events.dateText` and `documents.relevanceDateText` are free text, not timestamps.
- `sizeBytes` arrives as a JSON string in REST (int64).
- `parties` contains null-valued fields; deserialize `nullValue` as Option.
- No file path or hash in Firestore: the storage path convention must be confirmed from web source first.
- Many other users' cases share the root `cases` collection: always filter by uid.
- No cloud confidential flag: the local seal is the only guard.
