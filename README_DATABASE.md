# Database Setup

This project uses SQLite via `sqlx`. The database location is controlled by `DATABASE_URL` in `.env`.

## Install `sqlx-cli`

```bash
cargo install sqlx-cli
```

## Initialize the database

From the project root:

```bash
cd realworld-app-topcoat-sqlite
source .env
sqlx database setup
```

`sqlx database setup` creates the database and runs pending migrations in `./migrations`.

## Useful commands

```bash
sqlx database create   # create database from DATABASE_URL
sqlx database drop     # drop database from DATABASE_URL
sqlx database reset    # drop, recreate, and run migrations
sqlx database setup    # create and run pending migrations
sqlx migrate run       # run pending migrations
```

Set `DATABASE_URL` before running `sqlx` commands (usually via `source .env`).
