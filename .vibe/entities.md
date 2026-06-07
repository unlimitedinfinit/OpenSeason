# Entities

## Hunt (Vault)
A container representing an active investigation into government contractor fraud.

- **What is it?** An isolated directory on disk containing its own SQLite database (`metadata.db`) and encrypted evidence files.
- **Who creates it?** The user, by clicking "New Operation" or "See Example Walkthrough" (demo).
- **Who modifies it?** The user, by editing the target name, adding events/evidence, or deleting the hunt.
- **What depends on it?** All Evidence, Events, Parties, and Complaint Sections belong to a Hunt. Deleting a Hunt deletes its directory and database cascade.

## Evidence File
An encrypted file that contains proof of fraudulent activities.

- **What is it?** A local document or image that is stripped of EXIF/metadata, encrypted via XChaCha20Poly1305, and stored inside the Hunt's directory, indexed in the `evidence` SQLite table.
- **Who creates it?** The user, by uploading/dropping files into the Case Vault.
- **Who modifies it?** The user can update the description or delete the file. The file content itself is immutable after encryption.
- **What depends on it?** The compiled Typst PDF Report relies on the presence and SHA-256 hashes of these evidence files.

## Event (Timeline)
A point-in-time record of fraudulent behavior or contractor actions.

- **What is it?** A database entry representing a specific event with a date, title, description, and type, stored in the `events` table.
- **Who creates it?** The user, by adding an event to the Case Timeline.
- **Who modifies it?** The user can edit the event date, title, description, or type, or delete it.
- **What depends on it?** The chronological timeline in the compiled PDF Report is generated directly from these events.

## Party
An individual or organization involved in the target fraud.

- **What is it?** A database entry representing a key witness, contact, or suspect with contact info and notes, stored in the `parties` table.
- **Who creates it?** The user, by adding a party to the Case Vault.
- **Who modifies it?** The user can edit the name, role, email, phone, notes, or delete the party.
- **What depends on it?** The "Parties Involved" section of the compiled PDF Report and drafts.

## Complaint Section
A drafting section of the Qui Tam complaint outline.

- **What is it?** A database entry mapping a specific section ID (e.g. `introduction`, `jurisdiction`, `facts`, `violations`) to draft text content, stored in the `complaint_sections` table.
- **Who creates it?** Generated automatically on Hunt initialization with blank content.
- **Who modifies it?** The user, by editing sections in the Hunt Drafting page.
- **What depends on it?** The body sections of the compiled PDF report.
