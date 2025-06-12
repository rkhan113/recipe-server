# Roha Khan<br>
This software is wrtten in order to learn Rust full stack web basics. It will serve recipes. 

# Recipe Server

A simple Rust web server using Tokio, Axum, Askama, and SQLx with SQLite
that serves random recipes from a database.


## Setup Instructions

### Requirements

- Rust (with Cargo)
- SQLite (no need to install separately — handled by SQLx)
- `sqlx-cli` (install once)

```sh
cargo install sqlx-cli
```


### Steps

Step 1: Seed the Database

Run this once to:
- Create the database (db/recipe.db)
- Run migrations
- Load data from assets/static/recipes.json

```
cargo run -- --init-from assets/static/recipes.json
```

Step 2. Prepare SQL query cache

SQLx macros like query! require compile-time query validation. Run this to generate query metadata:

Mac (zsh/bash):
```
DATABASE_URL=sqlite://db/recipe.db cargo sqlx prepare
```

Windows (PowerShell):
```
$env:DATABASE_URL = "sqlite://db/recipe.db"; cargo sqlx prepare
```

Step 3. Run the server
Start the web server (after seeding):
```
cargo run
```
Open your browser at http://127.0.0.1:3000 to see a random recipe.


## Project Structure

- src/recipe.rs - Recipe data structures and JSON loading
- migrations/ - SQL migrations for DB schema
- assets/static/ - Static files including CSS and recipes JSON
- src/templates.rs - Askama templates for HTML rendering



## Notes
At this point I see a "set `DATABASE_URL` to use query macros online..." (I'm using VS-code & rut-analyzer), however is working in spite of this!!