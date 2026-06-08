# Decisions

## Why Tauri v2 instead of Electron?

- **Context**: The app requires strict filesystem access to manage local vaults under user local app data. It also needs native Rust libraries for cryptography (chacha20poly1305, argon2) and PDF compilation (typst). Electron is extremely bloated, increases binary size by ~100MB, and has a larger security attack surface.
- **Alternatives Considered**: Electron, Native OS apps (Swift/Kotlin/C#), Web-only application (rejected due to zero-cloud constraint).
- **Tradeoffs**: Rust compiler toolchain required for developer setup; debugging native bridge can be more complex than pure web.
- **Depends On**: Tauri's security model and active maintenance.
- **Threatened By**: Heavy dependencies or OS sandboxing changes that break local directories access.
- **Related Decisions**: [Why Svelte 5](#why-svelte-5)
- **Revisit When**: If Tauri fails to support crucial operating system APIs.

---

## Why Svelte 5 instead of React or Vue?

- **Context**: The UI needs to handle wizard flows, timeline lists, and drafting sections reactively. Svelte 5's compilation model and `$state` runes make state management highly predictable without the runtime overhead of Virtual DOM (React) or complex wrappers.
- **Alternatives Considered**: React (used in JustLegal, rejected here to minimize bundle footprint and leverage runes), Vue.
- **Tradeoffs**: Svelte 5 has a smaller ecosystem than React, but the core features are mature and sufficient.
- **Depends On**: Svelte compiler and Vite integration.
- **Threatened By**: Major breaking changes in Svelte ecosystem.
- **Related Decisions**: [Why Tauri v2](#why-tauri-v2-instead-of-electron)
- **Revisit When**: If we need to share complex component libraries directly with the React-based JustLegal client.

---

## Why SQLite Multi-Vault instead of a Single Central Database?

- **Context**: Under the "Bug Out Bag" protocol, users must be able to export an entire case (database + encrypted files) into a single `.osb` file, move it to another machine, and import it. Using a single central database makes exporting individual cases highly complex and risks leaking metadata of other cases.
- **Alternatives Considered**: Centralized SQLite db, Postgres, JSON files.
- **Tradeoffs**: Opening multiple connections if the user wants to search across all cases, though they are isolated by design.
- **Depends On**: `rusqlite` compatibility with target operating systems.
- **Threatened By**: Database corruption in a single hunt folder, though isolated to that case.
- **Related Decisions**: None.
- **Revisit When**: If cross-case indexing or global search becomes a primary requirement.

---

## Why Typst instead of genpdf or headless Chrome?

- **Context**: Generating professional legal disclosure statements needs a high-quality typesetting engine. Headless Chrome is too heavy and requires browser binaries. Other Rust PDF crates like `genpdf` or `lopdf` are effectively unmaintained or too low-level.
- **Alternatives Considered**: Headless browser printing, `genpdf` crate, `lopdf` crate, Typst.
- **Tradeoffs**: Typst syntax must be compiled dynamically via the Rust crate, but it yields LaTeX-grade output.
- **Depends On**: Maintainability of the `typst` library in Rust.
- **Threatened By**: Drastic changes to the Typst compiler interface.
- **Related Decisions**: None.
- **Revisit When**: If Typst compiling in Tauri exceeds target resource limits.

---

## Why Argon2id + XChaCha20Poly1305 + Zeroize?

- **Context**: Whistleblower safety is paramount. Any plaintext leak or recovery of keys from memory dumps ruins the claim and safety of the user. Keys must be session-only and Derived Key Material (DKM) zeroed out immediately after use.
- **Alternatives Considered**: AES-GCM, PBKDF2 (rejected as less secure against GPU-based cracking).
- **Tradeoffs**: Password hash derivation (Argon2id) takes a noticeable fraction of a second on launch, but is highly secure.
- **Depends On**: Security of the RustCrypto crates.
- **Threatened By**: Side-channel memory analysis or insecure key storage on disk.
- **Related Decisions**: None.
- **Revisit When:** If a newer NIST standard supersedes ChaCha20/Argon2.

---

## Why browser-native SVG displacement maps (GlassOS) instead of legacy CSS blurs or Chromium-only Canvas APIs?

- **Context**: We want our interfaces (JustLegal and OpenSeason) to look state-of-the-art and "wow" the user at first glance. Standard CSS `backdrop-filter: blur()` is passive and doesn't bend light or distort shapes.
- **Alternatives Considered**: Chromium-only SVG `backdrop-filter` or `HTML-in-Canvas` (which are flags-only and break on Safari/Firefox), or Canvas rendering of DOM nodes (which breaks accessibility and text selection).
- **Tradeoffs**: Generating displacement maps at runtime requires rendering quadrant canvases (symmetry optimizations) and cache-bypassing dynamically generated filter IDs (which increases repaint calls but stays under the 60fps budget).
- **Depends On**: SVG filter and Canvas API support.
- **Threatened By**: Heavy layout repaint costs in legacy Safari engines.
- **Revisit When**: CSS specifications adopt native light-refraction backdrop properties.
