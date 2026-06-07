# Troubleshooting

## Common Issues

### Svelte UI shows "Vault Locked" or commands return authorization error

**Cause:** The derived session key has been cleared from memory (either because the app was reloaded or the key dropped).

**Fix:**
Click the **LOCK** button in the top right to force the authorization screen, then input your Master Password again.

---

### USAspending API searches return empty or request fails

**Cause:** The application requires outbound internet access to query `api.usaspending.gov`. If you are offline, target verification will fail.

**Fix:**
Verify your network connection by trying to reach [api.usaspending.gov](https://api.usaspending.gov). If you are running under a VPN/proxy, make sure outbound calls are allowed.

---

### PDF Report Compilation fails or yields empty pages

**Cause:** The active case lacks essential timeline events or drafting sections, or the target name is empty.

**Fix:**
Make sure the operation has a target name set, at least one timeline event, and that the complaint draft sections are not completely blank before clicking **Report**.

---

### Exporting `.osb` fails with folder permission error

**Cause:** Tauri's filesystem capability scopes restrict exports to directories within `$HOME/.open-season/**` and the user's Downloads directory.

**Fix:**
Save your `.osb` file inside your **Downloads** directory or target a folder inside `C:\Users\<user>\.open-season\`.

---

## Diagnostic Commands

To check the build and compilation environments, run:

```bash
# Check Tauri dependencies and OS components status
npx tauri info

# Build the Rust backend directly to check compile errors
cd src-tauri
cargo check
```
