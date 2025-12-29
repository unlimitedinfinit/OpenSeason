# Open Season
### The Zero-Trust Fraud Hunting Toolkit

**Open Season** is a local-first, cryptographically secure desktop application designed for citizens to track, verify, and report government fraud. Built on the **Tauri v2** framework, it ensures that your investigation data never leaves your machine until you choose to export it.

## 🛡️ Core Philosophy: The "Hunting Blind"
- **Zero-Trust Architecture**: No cloud syncing, no telemetry, no external database connections.
- **Session-Only Authentication**: Your master password is used to derive an encryption key (Argon2id) in memory. It is never stored on disk.
- **Legal Airlock**: Mandatory acknowledgment of legal boundaries before every session.
- **Kill Kit**: Integrated Typst engine to generate professional "Confidential -- Attorney Work Product" disclosure statements without external dependencies.

## 🏗️ Technology Stack
- **Frontend**: Svelte 5 + TypeScript + TailwindCSS (shadcn-ui)
- **Backend**: Rust (Tauri 2.0)
- **Database**: SQLite (One isolated DB per Hunt)
- **Cryptography**: XChaCha20Poly1305 (Encryption) + Argon2id (KDF) + Zeroize (Memory)
- **Reporting**: Typst (PDF Generation)

## 🚀 Getting Started

### Prerequisites
- **Node.js** (v18 or later)
- **Rust** (Stable toolchain via [rustup](https://rustup.rs/))
- **Build Tools**:
  - *Windows*: Visual Studio Build Tools (C++ Workload)
  - *macOS*: Xcode Command Line Tools
  - *Linux*: `build-essential`, `libwebkit2gtk-4.0-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

### Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-username/open-season.git
   cd open-season
   ```

2. **Install Frontend Dependencies**
   ```bash
   npm install
   ```

3. **Run in Development Mode**
   Hot-reloads frontend and compiles Rust backend on the fly.
   ```bash
   npm run tauri dev
   ```

### Building for Release
Create a highly optimized, native binary for your operating system.

```bash
# Windows (.msi/.exe)
npm run tauri build

# macOS (.app/.dmg)
npm run tauri build

# Linux (.deb/.AppImage)
npm run tauri build
```

## 📂 Project Structure
```
open-season/
├── src/                # Svelte 5 Frontend
│   ├── routes/         # UI Pages (Dashboard, Wizard)
│   └── lib/            # Components (VaultLauncher, LegalAirlock)
├── src-tauri/          # Rust Backend ("The Armory")
│   ├── src/
│   │   ├── crypto.rs       # Encryption & Key Management
│   │   ├── branding.rs     # Typst PDF Engine
│   │   ├── usaspending.rs  # Intelligence Gathering API
│   │   └── bundle.rs       # .osb Import/Export Logic
│   └── capabilities/   # Security Scopes
└── ...
```

## ⚠️ Disclaimer
**Open Season is a neutral tool.** It does not verify the legal validity of your claims. You are solely responsible for ensuring your "Disclosure Statements" comply with the False Claims Act (31 U.S.C. §§ 3729–3733) and do not contain classified or defamatory information.

---
*Built for the Public Domain.*
