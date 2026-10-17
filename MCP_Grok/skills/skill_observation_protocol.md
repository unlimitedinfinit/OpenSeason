# SKILL: LAWFUL FIELD OBSERVATION & CRYPTOGRAPHIC CHAIN-OF-CUSTODY

Use this skill when advising citizens or whistleblowers on how to lawfully document suspected ghost facilities, shuttered clinics, or physical anomalies without breaking laws or compromising evidence.

---

## 1. Golden Rules of Lawful Citizen Observation
Defense attorneys and federal judges scrutinize how evidence was obtained. A single unlawful act can taint the entire investigation.

1. **STAY IN PUBLIC PROPERTY:**
   - Always document from public sidewalks, public roadways, or public parking easements.
   - **Never trespass** past "No Trespassing" signs, gates, or private clinic corridors.
   - **Never enter locked areas, force doors, or peek through closed window blinds.**
2. **MULTI-DAY OBSERVATION DISCIPLINE:**
   - A single photo of a closed door proves only that the clinic was closed at that specific second (e.g. lunch break or holiday).
   - Solid evidence requires **3 to 5 separate visits** across multiple regular business days (e.g. Tuesday at 10 AM, Thursday at 2 PM) during mandatory operating hours posted on the state licensing application.
3. **WHAT TO PHOTOGRAPH LAWFULLY:**
   - Posted operating hours on the front exterior door.
   - Empty parking lots during peak operating hours.
   - Piled-up, untouched mail or notices overflowing in exterior mailboxes.
   - Dark interiors where no lights, staff, or patients are visible.
4. **DO NOT POST TO SOCIAL MEDIA:**
   - Remind the user: posting observation videos to YouTube, X, or TikTok forfeits the statutory qui tam bounty and can expose the investigator to retaliation.

---

## 2. Cryptographic Ingestion & EXIF Extraction
Digital photos taken on smartphones contain embedded EXIF metadata (exact GPS coordinates and timestamp).

When the user ingests a photo into OpenSeason Desktop:
```bash
.\openseason-cli.exe --json qui-tam ingest-observation ./my-case --file "C:/Photos/clinic_tuesday_1400.jpg" --notes "Doors locked, no staff visible at 2:00 PM on Tuesday."
```

### What the Engine Does:
1. Extracts EXIF `DateTimeOriginal`, camera model, and GPS latitude/longitude.
2. Computes SHA-256 and SHA-512 hashes.
3. Appends the entry to an immutable `provenance.json` Merkle chain.
4. Sets the file permissions to **read-only** on disk.

This ensures defense lawyers cannot claim the photo was taken on a Sunday, fabricated with AI, or doctored after the fact.
