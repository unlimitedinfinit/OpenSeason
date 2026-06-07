# The .vibe Standard — A Complete Guide to Semantic Repository Layers

> **For AI agents and humans.** This document contains everything you need to create a `.vibe/` semantic layer for any software project, in any language, of any size. Read this entire document before starting.

---

## Table of Contents

1. [What is .vibe and Why Does It Exist?](#what-is-vibe-and-why-does-it-exist)
2. [The Core Principle](#the-core-principle)
3. [Folder Structure](#folder-structure)
4. [Before You Start — The Discovery Phase](#before-you-start--the-discovery-phase)
5. [File 1: purpose.md](#file-1-purposemd)
6. [File 2: architecture.md](#file-2-architecturemd)
7. [File 3: flows.md](#file-3-flowsmd)
8. [File 4: entities.md](#file-4-entitiesmd)
9. [File 5: decisions.md](#file-5-decisionsmd)
10. [File 6: state.json](#file-6-statejson)
11. [The Update Protocol](#the-update-protocol)
12. [Quality Checklist](#quality-checklist)
13. [Common Mistakes](#common-mistakes)
14. [FAQ](#faq)

---

## What is .vibe and Why Does It Exist?

Most repositories are impossible to understand without reading the source code. Documentation, when it exists, describes *how* things work (API routes, function signatures, config options) but never captures *why* things exist, *what* the system actually is, or *how* a user moves through it.

The `.vibe/` folder is a **semantic repository layer** — a small set of files that lets any human or AI understand a project without reading the source code. It describes **intent** rather than **implementation**.

### The Problem It Solves

When a new developer (or AI agent) encounters a codebase, they face these questions:

1. **What is this thing?** (Not "what framework does it use" — what *problem* does it solve?)
2. **How is it structured?** (Not "what files exist" — what *systems* exist and how do they talk?)
3. **What do users actually do with it?** (Not component descriptions — actual *journeys*)
4. **What are the important concepts?** (Not class hierarchies — the *nouns* of the domain)
5. **Why was it built this way?** (Not "how does X work" — why X instead of Y?)
6. **What's the current state?** (What works? What's broken? What are the constraints?)

If you can answer these 6 questions, you can understand any project. The 6 files in `.vibe/` answer exactly these 6 questions.

---

## The Core Principle

> **Describe intent, not implementation.**

Every line in `.vibe/` should pass this test: *"Would this still be true if we rewrote the entire codebase in a different language?"*

If yes → it belongs in `.vibe/`.
If no → it belongs in code comments, READMEs, or docs.

Examples:
- ✅ "We chose SQLite because the access pattern is <100 writes/hour" → Intent
- ❌ "The `DatabaseService` class has a `connect()` method" → Implementation
- ✅ "User uploads a document → AI analyzes it → results displayed" → Intent
- ❌ "POST /api/documents calls `processDocument()` in `services/ai.ts`" → Implementation

---

## Folder Structure

```
your-project/
└── .vibe/
    ├── purpose.md       — What this is, who it's for, what it doesn't do
    ├── architecture.md  — Systems and their connections
    ├── flows.md         — User journeys from start to finish
    ├── entities.md      — Important nouns and their relationships
    ├── decisions.md     — Why things exist the way they do
    └── state.json       — Machine-readable health snapshot
```

**Rules:**
- The `.vibe/` folder lives at the **root** of the repository.
- All 6 files are required. If a file would be empty (e.g., a library has no user flows), write "N/A — this is a library, not an application" rather than omitting it.
- Files should be concise. If a `.vibe/` file exceeds ~300 lines, you're probably describing implementation, not intent.
- Use Markdown for the 5 `.md` files. Use JSON for `state.json`.

---

## Before You Start — The Discovery Phase

Before writing any `.vibe/` files, you must understand the project. Here is your research protocol:

### Step 1: Scan the Repository Structure

List every directory and understand the top-level organization. Look for:
- Source code directories (src/, lib/, app/, pkg/)
- Configuration files (package.json, Cargo.toml, go.mod, pyproject.toml, *.csproj)
- Existing documentation (README.md, docs/, wiki/)
- Tests (tests/, __tests__/, *_test.go, *.spec.ts)
- Infrastructure (Dockerfile, docker-compose.yml, terraform/, .github/workflows/)
- Build artifacts (dist/, build/, out/, target/)

### Step 2: Read Existing Documentation

Read everything that exists:
- README.md (often contains the "what" and "how to run")
- CONTRIBUTING.md (often contains architectural decisions)
- CHANGELOG.md (reveals what's changed and why)
- Architecture Decision Records (ADRs) if they exist
- API docs, OpenAPI specs, or GraphQL schemas
- Comments in configuration files (these often explain "why")

### Step 3: Identify the Language and Framework Stack

Determine what you're working with:
- **Language:** Python, TypeScript, Go, Rust, C#, Java, C++, Swift, etc.
- **Frameworks:** React, Next.js, Django, FastAPI, Spring, Rails, Unity, Unreal, etc.
- **Database:** PostgreSQL, SQLite, MongoDB, Supabase, Firebase, none
- **Deployment:** Docker, Kubernetes, Vercel, AWS Lambda, bare metal, mobile app stores
- **AI/ML:** OpenAI, Anthropic, local models, Hugging Face, none

### Step 4: Read the Entry Points

Every project has entry points. Find and read them:
- **Web apps:** `index.html`, `App.tsx`, `main.py`, `app.py`, `server.ts`
- **CLI tools:** `main.go`, `__main__.py`, `cli.rs`, `Program.cs`
- **Libraries:** The main exported module and its public API
- **Games:** `GameManager`, `main.cpp`, `App.swift`
- **Mobile apps:** `MainActivity.kt`, `AppDelegate.swift`, `App.tsx`

### Step 5: Ask the User

Some information **cannot** be inferred from code. If any of the following are unclear after your research, ask the user directly:

**Questions you should ask:**

1. "What problem does this project solve, in one sentence?"
2. "Who uses this? Is it a tool for developers, an app for consumers, an internal service?"
3. "What are the things this project deliberately does NOT do?"
4. "Were there any major architectural decisions where you chose between alternatives? What were they and why did you choose what you chose?"
5. "What's currently broken, incomplete, or known to be problematic?"
6. "Are there any hard constraints? (e.g., must run offline, must support IE11, can't use GPL libraries, must stay under 50MB)"

**Do not ask** questions that you can answer by reading the code. Asking "what language is this in?" when there's a `package.json` in the root is insulting.

---

## File 1: purpose.md

### What It Answers
"What is this? Why does it exist?"

### Structure

```markdown
# Purpose

## What is this?
[1-3 sentences. No jargon. A non-technical person should understand.]

## Who is it for?
[Specific user personas. Not "developers" — what KIND of developers? Not "businesses" — what KIND of businesses?]

## What problem does it solve?
[The pain point that existed before this project. What was the user doing before? Why was that bad?]

## What does success look like?
[A concrete scenario. "A user does X, then Y, then Z, and it feels like W."]

## What does it explicitly NOT do?
[Hard boundaries. Things users might expect but will never get. This section prevents scope creep and misguided PRs.]
```

### How to Research It

1. Read the first paragraph of README.md — it usually has the "what"
2. Read the project's website or landing page if it exists
3. Check the package description in package.json / Cargo.toml / setup.py
4. If none of this is clear, **ask the user**

### Examples Across Different Project Types

**SaaS Web App (Legal Tech):**
```markdown
## What is this?
An AI-powered legal motion drafting platform that generates court filings from plain-English case descriptions.

## Who is it for?
Solo practitioners and small law firms (1-5 attorneys) who can't afford dedicated legal research staff.

## What does it explicitly NOT do?
- Not legal advice. The output requires attorney review.
- Does not handle federal court filings (only state courts).
- Does not store client data beyond the active session.
```

**Python CLI Tool:**
```markdown
## What is this?
A command-line tool that converts Figma design files into production-ready React components.

## Who is it for?
Frontend developers who receive designs from Figma and want to skip the manual conversion step.

## What does it explicitly NOT do?
- Does not handle animations or transitions.
- Does not generate tests.
- Requires a Figma API token — does not scrape the web interface.
```

**Game (Unity):**
```markdown
## What is this?
A roguelike deckbuilder where players navigate a procedurally generated dungeon using cards that represent spells, items, and movement.

## Who is it for?
Fans of Slay the Spire and Inscryption who want a more tactical, grid-based experience.

## What does it explicitly NOT do?
- No multiplayer.
- No microtransactions or loot boxes.
- No procedural narrative — story is hand-written.
```

### Anti-Patterns

- ❌ "This is a Next.js app with Tailwind and Supabase" — That's a tech stack, not a purpose.
- ❌ "This solves problems for users" — Too vague. WHAT problems? WHICH users?
- ❌ Omitting the "NOT do" section — This is the most valuable section. Every project has boundaries.

---

## File 2: architecture.md

### What It Answers
"What are the big pieces and how do they connect?"

### Structure

```markdown
# Architecture

## Systems
[A visual diagram showing the major systems — NOT files or classes, but conceptual systems like "Frontend", "API", "Database", "Auth", "Queue", "AI Layer"]

## How They Connect
[A table or description showing what protocol/method each system uses to talk to another]

## Trust Boundaries
[Where does trusted data end and untrusted data begin? Where are the security gates?]
```

### How to Research It

1. Look at the directory structure — top-level folders often map to systems
2. Read `docker-compose.yml` — each service is usually a system
3. Read import/dependency graphs — what modules depend on what?
4. Look for HTTP clients, database connections, queue publishers, API calls
5. Check environment variables — they usually point at external services

### The Diagram

Use ASCII art or Mermaid diagrams. The goal is a picture that shows all major systems and their connections **on one screen**. If your diagram doesn't fit on one screen, you're describing too many things.

**Good diagram (web app):**
```
┌────────────┐     HTTP      ┌────────────┐     SQL       ┌──────────┐
│  React SPA │ ───────────── │  FastAPI    │ ──────────── │ Postgres │
│  (Vercel)  │               │  (Railway)  │              │ (Supabase)│
└────────────┘               └──────┬─────┘              └──────────┘
                                    │ HTTP
                                    ▼
                             ┌──────────────┐
                             │  OpenAI API  │
                             │  (external)  │
                             └──────────────┘
```

**Good diagram (mobile app):**
```
┌──────────────┐    REST     ┌──────────────┐
│  Flutter App │ ──────────► │  Node.js API │
│  (iOS/Droid) │             │  (AWS Lambda)│
└──────────────┘             └──────┬───────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
             ┌──────────┐   ┌──────────┐   ┌──────────────┐
             │ DynamoDB  │   │ S3 Bucket│   │ Stripe API   │
             │ (users)   │   │ (uploads)│   │ (payments)   │
             └──────────┘   └──────────┘   └──────────────┘
```

### Trust Boundaries

This section is optional for simple projects but critical for anything with auth, payments, or user data:

```
UNTRUSTED              GATE                    TRUSTED
────────────────────────────────────────────────────────
Browser input    →   Zod validation      →   API handler
API request      →   JWT verification    →   Database query
File upload      →   MIME type check     →   S3 storage
Webhook payload  →   Signature verify    →   Event processing
```

### Anti-Patterns

- ❌ Listing every file in the project — That's a file tree, not an architecture.
- ❌ Describing internal class hierarchies — That's implementation.
- ❌ Including arrows without labels — Every connection should say *what* travels over it.

---

## File 3: flows.md

### What It Answers
"What do users actually DO with this system, step by step?"

### Structure

```markdown
# Flows

## 1. [Flow Name]
[Step-by-step journey using simple arrow notation]

## 2. [Flow Name]
...
```

### How to Research It

1. **Read the UI code** — pages, routes, screens, and components reveal user paths
2. **Read API routes** — they often follow CRUD patterns that map to user actions
3. **Read tests** — integration tests often describe user scenarios
4. **Look for onboarding/wizard flows** — these are always important
5. **Check for auth flows** — signup, login, password reset, OAuth
6. **If unclear, ask the user:** "Walk me through what a new user does from first visit to getting value."

### The Arrow Notation

Use simple step-by-step arrows. Each step should be something a user *does* or *sees*, not a technical operation:

```
User visits landing page
  ↓
Clicks "Sign Up" → enters email and password
  ↓
Receives verification email → clicks link
  ↓
Redirected to onboarding wizard
  ↓
Uploads first document
  ↓
AI processes document (loading spinner, ~10 seconds)
  ↓
Results displayed in dashboard with highlights
```

### What Flows to Include

Every project should have:
1. **The primary value flow** — The main thing users come to do
2. **The onboarding flow** — How a new user gets started
3. **The error/recovery flow** — What happens when something goes wrong

Optional but valuable:
- Admin/management flows
- Payment/billing flows
- Data export flows
- Integration/API flows (if it's a platform)

### Examples Across Different Project Types

**E-commerce:**
```
## 1. Purchase Flow
Customer browses catalog → filters by category
  ↓
Clicks product → views details, reviews, sizing
  ↓
Selects size and quantity → adds to cart
  ↓
Clicks checkout → enters shipping address
  ↓
Selects payment method → Stripe checkout
  ↓
Order confirmed → email receipt sent
  ↓
Warehouse notified → shipping label generated
```

**CLI Tool:**
```
## 1. First Run
User installs via pip: `pip install figma2react`
  ↓
Runs `figma2react init` → prompted for Figma API token
  ↓
Token stored in ~/.figma2react/config.json
  ↓
Runs `figma2react convert <figma-url>` → fetches design
  ↓
Components generated in ./output/ directory
  ↓
User reviews and copies into their project
```

**Library (no user flows):**
```
# Flows

N/A — This is a library, not an application. Developers import it and call functions.

The typical integration pattern is:

Developer installs the package
  ↓
Imports the client: `from omniauth import Client`
  ↓
Configures with API key: `client = Client(api_key="...")`
  ↓
Calls methods: `client.authenticate(user, password)`
  ↓
Handles response or error
```

### Anti-Patterns

- ❌ Describing API request/response payloads — That's API docs, not flows.
- ❌ Including technical middleware steps ("JWT decoded, rate limiter checked") — Users don't see those.
- ❌ Only one flow — Every project has at least 2-3 important journeys.

---

## File 4: entities.md

### What It Answers
"What are the important nouns in this system?"

### Structure

For each entity:

```markdown
## [Entity Name]

[One sentence: what is this thing?]

- **What is it?** [Detailed explanation]
- **Who creates it?** [What user action or system process brings it into existence]
- **Who modifies it?** [What can change it after creation]
- **What depends on it?** [What breaks or changes if this entity is deleted/modified]
```

### How to Research It

1. **Database schema** — Every table/collection is likely an entity
2. **TypeScript/Python types** — Interface definitions, dataclasses, structs
3. **API routes** — Resources in REST URLs are entities (`/users`, `/projects`, `/invoices`)
4. **UI labels** — What nouns appear in the interface? What do users talk about?
5. **Domain language** — What words does the README/docs use repeatedly?

### How to Choose What's an Entity

**Include:**
- Things users create, view, edit, or delete
- Things the system generates that users care about
- Configuration objects that change system behavior
- Credentials, tokens, or keys

**Exclude:**
- Internal data structures (linked lists, hashmaps, caches)
- Framework-specific concepts (middleware, reducers, controllers)
- Temporary objects (request context, session state)

### Examples Across Different Project Types

**SaaS (Project Management):**
```markdown
## Project
A container for related tasks, files, and team members.

- **What is it?** A workspace where a team organizes their work. Has a name, description, and member list.
- **Who creates it?** Any user can create a project from the dashboard.
- **Who modifies it?** Project owner and admins can rename, archive, or delete.
- **What depends on it?** All Tasks, Files, and Comments belong to a Project. Deleting a Project cascades to everything inside it.

## Task
A unit of work assigned to a team member.

- **What is it?** An item with a title, description, status (todo/in-progress/done), assignee, and due date.
- **Who creates it?** Any project member.
- **Who modifies it?** The assignee or project admin can change status, reassign, or close.
- **What depends on it?** Comments are attached to Tasks. Time entries reference Tasks. Sprint velocity is calculated from completed Tasks.
```

**Game (RPG):**
```markdown
## Character
The player's in-game avatar.

- **What is it?** A persistent game entity with stats (HP, STR, DEX, INT), inventory, equipped items, and quest progress.
- **Who creates it?** The player, during the character creation screen.
- **Who modifies it?** The game engine (leveling up, taking damage), the player (equipping items, spending stat points).
- **What depends on it?** Save files, leaderboards, multiplayer matchmaking.

## Card
A spell, item, or ability the player can play during combat.

- **What is it?** A game object with a name, mana cost, effect description, rarity, and type (attack/defense/utility).
- **Who creates it?** Game designers (defined in card_database.json). Players discover cards through loot drops.
- **Who modifies it?** Cards are immutable. Upgrades create a new variant (e.g., "Fireball+" replaces "Fireball").
- **What depends on it?** Decks are collections of Cards. Combat resolution depends on Card effects. Balance patches modify Card stats globally.
```

### Anti-Patterns

- ❌ Listing database columns — "User has id, email, password_hash, created_at" is a schema dump, not an entity description.
- ❌ Including framework concepts — "Controller", "Middleware", "Reducer" are not domain entities.
- ❌ Skipping "What depends on it?" — This is the most important field. It reveals hidden coupling.

---

## File 5: decisions.md

### What It Answers
"Why was it built this way instead of some other way?"

### This Is the Most Important File

This single file can save weeks of confusion. Most repositories are impossible to understand because nobody recorded *why* architectural choices were made. Without this context, every new contributor (human or AI) will:
- Try to "fix" things that were intentional
- Propose alternatives that were already considered and rejected
- Repeat mistakes that were already learned from

### Structure

Each decision follows this pattern:

```markdown
## Why [decision]?

[Explanation of the context and constraints that led to this choice]

[What alternatives were considered and why they were rejected]

[What tradeoffs were accepted]
```

### How to Research It

This is the hardest file to create from code alone. Decisions live in:

1. **Git history** — `git log --oneline` for major refactors or dependency changes
2. **Pull request descriptions** — Often explain "why" in the body text
3. **Issue tracker** — Closed issues often contain design discussions
4. **Code comments** — Look for comments that say "NOTE:", "HACK:", "WORKAROUND:", "TODO:"
5. **Config files** — Pinned dependency versions often have a reason
6. **Architecture Decision Records (ADRs)** — If they exist, copy them into decisions.md
7. **Ask the user** — This is the file where user input is most valuable

**Questions to ask the user for this file:**

- "Were there any major decisions where you debated between two approaches?"
- "Is there anything in the codebase that looks weird but has a good reason for existing?"
- "What dependencies did you evaluate before choosing the ones you use?"
- "Are there any constraints imposed by external factors (client requirements, hosting limits, compliance, legacy systems)?"

### What Decisions to Document

**Always document:**
- Programming language choice (if it's not obvious)
- Framework choice
- Database choice
- Authentication method
- Hosting/deployment strategy
- Any pinned dependency versions
- Anything that deviates from "the obvious choice"

**Document if relevant:**
- Why monolith vs. microservices
- Why SQL vs. NoSQL
- Why REST vs. GraphQL
- Why SSR vs. SPA vs. static
- Why self-hosted vs. managed services
- Why a specific third-party API was chosen

### Examples Across Different Project Types

**Web App:**
```markdown
## Why Next.js instead of Vite or Create React App?

We need server-side rendering for SEO (the landing page and public case studies must be indexable). Vite is faster for development but doesn't provide SSR without additional configuration. CRA is deprecated.

The tradeoff: Next.js imposes its own routing conventions and makes it harder to eject to a custom setup later.

## Why Supabase instead of Firebase?

Supabase gives us a real PostgreSQL database, which means we can write complex queries (joins, CTEs) without learning a proprietary query language. Firebase's Firestore has no joins.

The tradeoff: Supabase's auth and storage are less mature than Firebase's, and the community is smaller.

## Why we don't use an ORM

The database schema has 4 tables and will likely never exceed 10. Prisma, Drizzle, and TypeORM all add complexity (code generation, migration files, type mapping) that isn't justified at this scale. We use raw SQL with parameterized queries.

The tradeoff: If the schema grows significantly, we'll revisit this.
```

**Embedded Firmware:**
```markdown
## Why C instead of Rust?

The target microcontroller (STM32F103) has 64KB of flash. Rust's core library and panic handling alone consume ~20KB. Our C firmware uses 31KB. We can't afford the overhead.

The tradeoff: No borrow checker, manual memory management, and classic C footguns. We mitigate this with -Wall -Wextra -Werror and a CI pipeline that runs static analysis (cppcheck).

## Why polling instead of interrupt-driven I/O?

The sensor sampling rate is 10 Hz. At this frequency, the overhead of setting up and tearing down interrupt contexts exceeds the cost of a simple polling loop. We poll on a 50ms timer tick.

The tradeoff: If we add a sensor requiring >100 Hz sampling, we'll need to refactor to interrupts.
```

### Anti-Patterns

- ❌ "We use React because it's popular" — That's not a reason. WHY is popularity relevant? (Community support? Hiring pool? Existing team knowledge?)
- ❌ Documenting only technology choices — Also document *design* choices ("Why is the cart stored in localStorage instead of the database?")
- ❌ Being defensive — Don't justify bad decisions. If something is known to be suboptimal, say so: "We chose X because of time constraints. The better long-term choice would be Y."

---

## File 6: state.json

### What It Answers
"What's the current health and status of this project, in machine-readable form?"

### Structure

```json
{
  "version": "1.0.0",
  "health": "active",
  "status": "production",
  "last_audit": "2025-01-15",
  "languages": {},
  "known_issues": [],
  "dependencies": [],
  "constraints": [],
  "vibe_version": "1.0",
  "vibe_updated": "2025-01-15T10:00:00Z"
}
```

### Field Definitions

| Field | Type | Required | Description |
|---|---|---|---|
| `version` | string | ✅ | Project version (semver preferred) |
| `health` | enum | ✅ | `"active"`, `"maintenance"`, `"archived"`, `"experimental"` |
| `status` | string | ✅ | `"prototype"`, `"alpha"`, `"beta"`, `"production"`, `"deprecated"` |
| `last_audit` | string | ✅ | ISO date of last .vibe review |
| `languages` | object | ✅ | Map of language → version/details |
| `known_issues` | string[] | ✅ | Human-readable list of known problems |
| `dependencies` | string[] or object | ✅ | Major external dependencies (not every npm package — just the important ones) |
| `constraints` | string[] | ✅ | Hard rules that must not be violated |
| `vibe_version` | string | ✅ | Version of the .vibe spec used (currently `"1.0"`) |
| `vibe_updated` | string | ✅ | ISO timestamp of last .vibe update |

### What to Include in `known_issues`

Only items that affect the user experience or developer workflow:
- ✅ "Search is slow on datasets larger than 10,000 records"
- ✅ "OAuth login fails on Safari due to cookie SameSite issue"
- ❌ "The utils.ts file needs refactoring" (internal, doesn't affect anyone)

### What to Include in `constraints`

Rules that a new contributor must not violate:
- ✅ "Must support Node.js 18+ (LTS only)"
- ✅ "No GPL-licensed dependencies (client requirement)"
- ✅ "Bundle size must stay under 500KB gzipped"
- ✅ "All API responses must complete in under 200ms"

### Example (Simple Web App)

```json
{
  "version": "2.1.0",
  "health": "active",
  "status": "production",
  "last_audit": "2025-06-01",
  "languages": {
    "typescript": "5.4",
    "python": "3.12"
  },
  "known_issues": [
    "PDF export occasionally fails for documents with >50 pages",
    "Dark mode has contrast issues on the settings page"
  ],
  "dependencies": [
    "next.js",
    "supabase",
    "openai",
    "stripe",
    "resend"
  ],
  "constraints": [
    "Node.js 18+ required",
    "Must work offline after initial load",
    "No tracking or analytics scripts",
    "OpenAI API key must be server-side only"
  ],
  "vibe_version": "1.0",
  "vibe_updated": "2025-06-01T14:30:00Z"
}
```

---

## The Update Protocol

The `.vibe/` folder is a **living document**. It must be updated whenever the project changes in a way that affects intent, architecture, flows, entities, decisions, or health.

### When to Update

| Event | What to Update |
|---|---|
| New feature added | `flows.md` (new user journey), `entities.md` (new nouns), `state.json` (version) |
| Major refactor | `architecture.md` (if systems changed), `decisions.md` (why you refactored) |
| New dependency added | `decisions.md` (why this dep?), `state.json` (dependencies list) |
| Bug discovered | `state.json` (known_issues) |
| Bug fixed | `state.json` (remove from known_issues, bump version) |
| Constraint changed | `state.json` (constraints), `decisions.md` (why the change) |
| Project pivoted | `purpose.md` (new purpose), potentially everything |

### The Rule

> **Every time an AI agent touches the repository, it must update the .vibe folder. Not just the code — the understanding.**

This means the workflow for an AI agent is:

```
Receive task
  ↓
Read .vibe/ to understand the project
  ↓
Make code changes
  ↓
Update relevant .vibe/ files to reflect new understanding
  ↓
Commit code + .vibe changes together
```

### What "Update" Means

- **state.json**: Always update `vibe_updated` timestamp. Update `version`, `known_issues`, `dependencies`, and `constraints` as needed.
- **Other files**: Only update if the *intent* changed. Fixing a typo in code doesn't require updating `architecture.md`. Adding a new payment system does.

---

## Quality Checklist

After creating a `.vibe/` folder, verify it passes these checks:

### purpose.md
- [ ] Can a non-technical person read it and understand what the project does?
- [ ] Does the "NOT do" section have at least 3 items?
- [ ] Does "success" describe a concrete scenario, not abstract goals?

### architecture.md
- [ ] Does the diagram fit on one screen?
- [ ] Does every arrow/connection have a label (protocol, method)?
- [ ] Are there fewer than 10 boxes in the diagram? (If more, you're too granular)
- [ ] Would the diagram survive a language rewrite?

### flows.md
- [ ] Are there at least 2 flows?
- [ ] Does each step describe something a user DOES or SEES (not internal processing)?
- [ ] Can someone follow the flow without reading any code?

### entities.md
- [ ] Does every entity have all 4 fields (what/who creates/who modifies/what depends)?
- [ ] Are all entities domain nouns (not framework concepts)?
- [ ] Is the "what depends on it" field filled out? (This is the most commonly skipped)

### decisions.md
- [ ] Does every decision explain the alternatives that were rejected?
- [ ] Does every decision state the tradeoff that was accepted?
- [ ] Are there at least 3 decisions? (Every project has at least language + framework + database)

### state.json
- [ ] Is `vibe_updated` set to the current date/time?
- [ ] Does `known_issues` contain real issues (not aspirational improvements)?
- [ ] Does `constraints` contain only things that MUST NOT be violated?
- [ ] Is the JSON valid? (run through a linter)

---

## Common Mistakes

### 1. Writing a second README
`.vibe/` is not a README. The README says "here's how to install and run this." `.vibe/` says "here's what this IS and WHY it exists." If your `purpose.md` has installation instructions, you're doing it wrong.

### 2. Describing implementation instead of intent
If you find yourself writing function names, file paths, or class hierarchies, stop. Those belong in code docs. `.vibe/` should survive a complete rewrite.

### 3. Making it too long
If `architecture.md` is 500 lines, you're describing the codebase, not the architecture. The `.vibe/` folder for a 100,000-line project should be roughly the same size as for a 1,000-line project — because it describes the same number of *systems* and *decisions*.

### 4. Skipping decisions.md
This is the file people skip because it's hard. It requires knowing *why*, which means talking to the original developers or reading git history. It's also the most valuable file. Don't skip it.

### 5. Not updating state.json
If `vibe_updated` is 6 months old and the project had 200 commits since then, the `.vibe/` folder is stale and untrustworthy. Always update the timestamp.

### 6. Including sensitive information
Never put API keys, passwords, internal URLs, or customer data in `.vibe/` files. They're meant to be committed to version control.

---

## FAQ

**Q: What if the project is tiny (one file, 200 lines)?**
A: Still create `.vibe/`. Even a small script has a purpose, makes architectural decisions, and has constraints. The files will just be shorter.

**Q: What if I can't figure out the "why" behind a decision?**
A: Write "Reason unknown — this decision predates available documentation. Best guess: [your hypothesis]." Marking uncertainty is better than guessing or omitting.

**Q: Should `.vibe/` be committed to version control?**
A: Yes. It's part of the project, not a personal note. It should be reviewed in PRs just like code.

**Q: What if the project has multiple deployable services (monorepo)?**
A: One `.vibe/` folder at the root. The `architecture.md` diagram shows all services. If a service is complex enough to need its own `.vibe/`, it should probably be its own repository.

**Q: How is this different from Architecture Decision Records (ADRs)?**
A: ADRs are one file per decision, chronologically ordered, and often contain implementation details. `decisions.md` is a single file with all active decisions, focused on intent. If ADRs exist, use them as source material for `decisions.md`.

**Q: Can I add custom files to `.vibe/`?**
A: No. The power of the standard is that every `.vibe/` folder has exactly the same 6 files. An AI agent (or human) always knows exactly where to look. If you need project-specific documentation, put it in `docs/`.

---

## Version

This document describes **.vibe specification version 1.0**.

Created: 2026-06-06
Author: Developed through collaborative iteration between a human systems architect and an AI coding agent during the Omni OS project.
