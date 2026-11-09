# realworld-app-topcoat-sqlite

## Background

I started learning Rust by building a RealWorld-style full-stack blog application with the [Leptos](https://leptos.dev/) framework. As a follow-up learning exercise, I rebuilt the same application with [Dioxus](https://dioxuslabs.com/), another popular Rust UI framework, to compare the development experience and architecture.

More recently, the [Topcoat](https://crates.io/crates/topcoat) framework became available with an interesting approach: server-rendered UI without a separate WASM frontend build and without wiring up a separate Axum application layer for the app itself. This repository is the same RealWorld-style application implemented with Topcoat and SQLite, as another step in that learning journey.

The goal is to keep the app practical and easy to run locally, while exploring how the same product surface maps across Rust web frameworks.

## Quick glance

<!-- Add 1–2 screenshots here to show the home/feed/article views. -->

<img width="1895" height="1035" alt="image" src="https://github.com/user-attachments/assets/49273829-16e1-4a77-bb05-7daea5afa822" />

<img width="1895" height="1035" alt="image" src="https://github.com/user-attachments/assets/353b6eb9-0b84-4e2e-9c57-c27a1e9d7754" />



**App access (local):**

```bash
cargo run
```

Then open the URL printed by the server in your browser.

## Minimum tooling

| Tool | Purpose | Install / verify |
| --- | --- | --- |
| Rust toolchain (`stable`) | Build and run the app | Install via [rustup](https://rustup.rs/). Verify: `rustc --version` and `cargo --version` |
| `topcoat` CLI (`topcoat-cli` / `cargo-topcoat`) | Topcoat development commands (`topcoat dev`, formatting, asset/UI helpers) | Install a prebuilt binary (if available in your setup): `cargo binstall topcoat-cli` — or build/install with Cargo as appropriate for your environment. Verify: `topcoat --version` or `cargo topcoat --help` |
| `sqlx-cli` *(optional but recommended)* | Create/reset the SQLite database and run migrations from the command line | `cargo install sqlx-cli` — verify: `sqlx --version` |

> `cargo run` works for this project. The Topcoat CLI is useful for Topcoat-specific workflows (for example, `topcoat dev` / `cargo topcoat dev`) and project utilities.

## Build and run

1. Clone the repository and enter the project directory:

```bash
git clone <repository-url>
cd realworld-app-topcoat-sqlite
```

2. Configure the database URL (see `.env`):

```bash
source .env
```

3. Create the database and run migrations:

```bash
cargo install sqlx-cli   # if not already installed
sqlx database setup
```

If you hit database issues, see [README_DATABASE.md](./README_DATABASE.md).

4. Run the application:

```bash
cargo run
```

For a Topcoat development workflow (if you use the CLI), you can also try:

```bash
cargo topcoat dev
# or
topcoat dev

santhosh@fedora:~/my_github_repos/realworld-app-topcoat-sqlite$ topcoat dev

  topcoat dev server
  watching for file changes...
  press r to reload

  ready on http://127.0.0.1:3000

  no changes; application up to date

```

Open the address printed by the server in your browser.

## Features

- Server-rendered UI with Topcoat views/components
- SQLite backend (`sqlx`) with migrations
- Authentication using Topcoat's built-in session support
- Articles, comments, profiles, favorites/follows
- Full-text search (SQLite FTS5)
- Dark/light theme preference
- Password hashing with `argon2`

## Sample data

Migrations may include sample users/data for demo purposes (e.g. `user1`–`user5`, password same as username). To remove sample data, delete the `basedata` migration files under `./migrations` and recreate the database (see [README_DATABASE.md](./README_DATABASE.md)).

## Project layout

- `src/main.rs` — app entry, router/layout, shared UI shell
- `src/views/` — route handlers/views
- `src/components/` — reusable UI components
- `src/models/` — data models and queries
- `src/auth/` — auth helpers
- `migrations/` — SQLx migrations
- `styles.css` — project styles

## Other variants (same app)

This app is part of a series of RealWorld-style implementations using different Rust frameworks, databases, and auth approaches:

| Framework | Database | Auth | Repository |
| --- | --- | --- | --- |
| **Topcoat** | SQLite | Topcoat built-in session | *This repository* |
| **Dioxus** | SQLite | JWT (`jsonwebtoken`) | [realworld-app-dioxus-sqlite](https://github.com/santhosh7403/realworld-app-dioxus-sqlite) |
| **Dioxus** | SQLite | Session (`axum_session`, `axum_session_auth`) | [axum-session-auth-realworld-app-dioxus-sqlite](https://github.com/santhosh7403/axum-session-auth-realworld-app-dioxus-sqlite) |
| **Dioxus** | SQLite | Session (`tower_sessions`, `axum_login`) | [tower-sessions-axum-login-realworld-app-dioxus-sqlite](https://github.com/santhosh7403/tower-sessions-axum-login-realworld-app-dioxus-sqlite) |
| **Dioxus** | SQLite | PASETO (`pasetors`) | [paseto-auth-realworld-app-dioxus-sqlite](https://github.com/santhosh7403/paseto-auth-realworld-app-dioxus-sqlite) |
| **Leptos** | SQLite | JWT (`jsonwebtoken`) | [realworld-app-leptos-axum-sqlite](https://github.com/santhosh7403/realworld-app-leptos-axum-sqlite) |
| **Leptos** | PostgreSQL | JWT (`jsonwebtoken`) | [realworld-app-leptos-axum](https://github.com/santhosh7403/realworld-app-leptos-axum) |
| **Leptos** | PostgreSQL | Session (`axum_session`, `axum_session_auth`) | [axum-session-auth-realworld-app-leptos-postgres](https://github.com/santhosh7403/axum-session-auth-realworld-app-leptos-postgres) |

## Notes

- SQLite FTS5 is used for article search; see the [SQLite FTS5 docs](https://www.sqlite.org/fts5.html).
- Styling uses project CSS plus CDN assets referenced from the app shell (Tailwind CDN and Font Awesome in the current layout). Adjust as needed for your deployment.
- `README_DATABASE.md` contains additional `sqlx` database commands.
