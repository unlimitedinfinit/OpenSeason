# API Reference

## Base Connection
Open Season does not run a web server. Svelte communicates with the Rust backend ("The Armory") using **Tauri IPC Invokes**.

---

## Tauri Invoke Commands

### `unlock_vault`
Derive the master session encryption key from the user password.

- **Arguments**:
  ```json
  { "password": "user_passphrase" }
  ```
- **Returns**: `Result<(), String>` (throws error if key derivation fails)

---

### `lock_vault`
Wipe the active session key from memory.

- **Arguments**: None.
- **Returns**: `Result<(), String>`

---

### `is_locked`
Check if the cryptographic vault is currently locked.

- **Arguments**: None.
- **Returns**: `boolean`

---

### `list_hunts`
Retrieve a list of all active operations stored in the local vaults folder.

- **Arguments**: None.
- **Returns**:
  ```json
  [
    {
      "id": "vault_uuid",
      "name": "Operation Name",
      "created": "timestamp"
    }
  ]
  ```

---

### `create_new_hunt`
Initialize a new isolated vault directory and SQLite database.

- **Arguments**:
  ```json
  { "name": "Operation Name" }
  ```
- **Returns**:
  ```json
  {
    "id": "new_uuid",
    "name": "Operation Name",
    "created": "timestamp"
  }
  ```

---

### `verify_target_cmd`
Query the USAspending.gov public API to locate recipient contracts over $100k.

- **Arguments**:
  ```json
  { "name": "Contractor Name" }
  ```
- **Returns**:
  ```json
  [
    {
      "generated_internal_id": "award_id",
      "date_signed": "YYYY-MM-DD",
      "description": "Award Details description",
      "total_obligation": 1500000.0,
      "awarding_agency": "Agency Name",
      "recipient_name": "Contractor Name"
    }
  ]
  ```

---

### `save_disclosure_cmd`
Compile the database records into a PDF Disclosure Statement using the Typst engine.

- **Arguments**:
  ```json
  {
    "huntId": "vault_uuid",
    "target": "Contractor Name",
    "count": 12,
    "value": 1540000.0
  }
  ```
- **Returns**: `String` (path to the generated PDF saved in the system Downloads directory)

---

### `export_hunt_cmd`
Export a specific hunt vault (database + encrypted files) into a compressed `.osb` ZIP file.

- **Arguments**:
  ```json
  {
    "huntId": "vault_uuid",
    "targetPath": "C:/path/to/backup.osb"
  }
  ```
- **Returns**: `Result<String, String>`

---

### `import_hunt_cmd`
Unpack and import a `.osb` case ZIP bundle back into the local vaults database directory.

- **Arguments**:
  ```json
  {
    "osbPath": "C:/path/to/backup.osb"
  }
  ```
- **Returns**: `Result<(), String>`
