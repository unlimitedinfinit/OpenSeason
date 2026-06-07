# Open Issues

## 🔴 Critical
- None.

## 🟡 Medium
- **Hardcoded Mock PDF Metrics**: When generating reports from the main dashboard, the arguments for evidence counts and valuations are hardcoded to mock values (`count: 12`, `value: 1540000.0`). The Svelte case uploader must calculate these values dynamically from the SQLite db.
- **Corrupted Bundle Imports**: The `import_hunt_cmd` does not perform validation checks on the ZIP archive layout before extraction. If the `.osb` file is corrupted or not a zip file, it throws a standard Rust panic error rather than a user-friendly error.

## 🟢 Low / Tech Debt
- **Svelte navigation warnings**: Navigation uses `goto` directly without a router context wrapper if ran inside pure Tauri dev containers, producing minor console warnings.

## Backlog
- **In-App PDF Viewer**: Add a native PDF viewer modal in Svelte so users do not have to leave the application to review their generated files.
