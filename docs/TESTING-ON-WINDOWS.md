# Testing OpenSeason on Windows

This is the checklist for a local desktop run. The app is offline-first. It does not send telemetry.

## Prerequisites

1. **Node.js 18 or later**  
   Install from [https://nodejs.org](https://nodejs.org). In PowerShell: `node -v` should print `v18` or newer.

2. **Rust 1.85 or later**  
   Install from [https://rustup.rs](https://rustup.rs). In PowerShell: `rustc -V` should print `rustc 1.85` or newer. If it is older, run `rustup update stable`.

3. **Visual Studio Build Tools (C++) if you compile locally**  
   If PowerShell prints `link.exe not found` or `linker 'link.exe' not found`, the laptop does not have MSVC. Install [Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/), pick the **Desktop development with C++** workload, then open a **new** Developer PowerShell.  
   **You do not have to compile on the laptop.** GitHub Actions workflow `.github/workflows/windows-installer.yml` builds the `.msi` / `.exe` on `windows-latest` and uploads them as the `openseason-windows-installer` artifact. Download that artifact. It is not a GitHub Release. WebView2 is required to run the app. Current Windows 10/11 machines usually already have it.

4. **Git** if you are cloning the repository.

## Install and run (dev)

In PowerShell, from the repository root:

```powershell
git clone https://github.com/unlimitedinfinit/OpenSeason.git
cd OpenSeason
git checkout cursor/unified-legal-builder-5286
npm install --legacy-peer-deps
npm run tauri dev
```

The first run compiles the Rust backend and can take several minutes. A desktop window should open on the home screen.

If `npm run tauri dev` fails on missing WebView2, install the Evergreen WebView2 runtime from Microsoft and try again.

## Build an installer

```powershell
npm run tauri build
```

This needs the same C++ toolchain (`link.exe`). If that toolchain is missing, skip this step and download the `openseason-windows-installer` artifact from the Actions tab. Output from a local build lands under `src-tauri/target/release/bundle/` (`.msi` and/or `.exe`). The workflow does not create a GitHub Release.

### Agent / CLI on Windows

```powershell
npm run cli -- --json packs list
bash examples/agent-e2e.sh
```

If `bash` is missing, run the same `npm run cli -- --json ...` lines from `examples/agent-e2e.sh` in PowerShell.

## Manual test checklist

Work with fake names only (for example Jordan Example). Do not put real people, real docket numbers, or real exhibits into a test vault you might later share.

### Home

- [ ] Home lists every case together. Each card shows **standard** or **confidential**.
- [ ] Empty state appears if the cases folder is empty (no leftover sample except what you created).
- [ ] **Create a case**, **Open a case folder**, **Import .osb**, and **Unlock Confidential** are visible.
- [ ] Notice of Appeal is marked Ready. Complaint and Motion are marked **Basic template**, not "later release".
- [ ] The review / not-legal-advice notice is visible.

### Guided wizard

- [ ] Create a case walks through: mode, court, docket, caption, parties, filer, document.
- [ ] Parties can be added, edited, and removed. Role includes plaintiff, defendant, appellant, appellee. Counsel accepts "Pro se" or an attorney name.
- [ ] Choosing Confidential without unlocking shows a clear error.
- [ ] After finish, the case dashboard opens.

### Case dashboard (Standard)

- [ ] Overview edits court, docket, filer, and signature block. Save writes `case.json`.
- [ ] Parties tab uses the same editor as the wizard.
- [ ] Documents tab exports Notice of Appeal to Word and PDF under `exports/`.
- [ ] Complaint and Motion export Word and PDF. The UI says they are basic templates.
- [ ] Documents tab imports a Markdown draft and exports court Markdown Word/PDF.
- [ ] Check tab runs fact, authority, and deadline (NJ R. 2:11-6 from 2026-01-09 should due 2026-01-20). Issues list appears. The app never invents cites.
- [ ] Evidence copies a file into the case `evidence/` folder.
- [ ] Timeline add/remove persists after Save.
- [ ] Drafts and exports lists the files that were written.
- [ ] Validate reports missing fields instead of failing silently.
- [ ] Account backup shows the stub refusal (`not_implemented`).

### Confidential

- [ ] Unlock Confidential shows the Legal Airlock (`I UNDERSTAND`) then the vault password.
- [ ] A new Confidential case from the wizard appears on the home list with a confidential badge.
- [ ] Older vault hunts appear as Confidential cases after unlock (same id).
- [ ] Confidential tools (USAspending lookup, disclosure statement, `.osb` export) live on the case dashboard, not a second app home.
- [ ] Adding evidence to a **sealed** Confidential case refuses without a password, refuses a wrong password, and accepts the correct password.
- [ ] Adding the same exhibit file twice does not replace or delete the existing `.enc` (hash collision must be a no-op). The UI should say the sealed copy was not changed.
- [ ] Convert to Confidential on a Standard case asks for the vault password again and then seals.

### Import / open

- [ ] Open a case folder: paste a path that contains `case.json`. The case appears on the home list.
- [ ] Import `.osb` after unlock creates a Confidential case.

### Errors and notices

- [ ] Empty lists use dashed empty states, not blank screens.
- [ ] Failed saves and exports show the Rust error text.
- [ ] Every export path still includes the review / not-legal-advice notice.

## If something fails

- `link.exe not found`: install Visual Studio Build Tools (Desktop C++) or stop compiling and use the CI installer artifact instead.
- `npm run tauri dev` compile errors: confirm `rustc -V` is 1.85+ and the C++ workload is installed.
- Window opens but case list is empty: look at `Documents\JustLegal\Cases\` and the app data vault folder (`%LOCALAPPDATA%\com.openseason.app\vaults\`).
- PDF export fails: install a common system font (the generic court rules prefer Times New Roman / Liberation Serif / Arial).
- Lost vault password: there is no recovery. That is intentional.

See [AUDIT.md](AUDIT.md) for known gaps before you treat a failure as a regression.
