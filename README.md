# Roha Khan<br>
This software is wrtten in order to learn Rust full stack web basics. It will serve recipes. 

# Recipe Server

A simple Rust web server using Tokio, Axum, Askama, and SQLx with SQLite
that serves random recipes from a database.


## Requirements

- Rust (with Cargo)
- SQLite
- `sqlx-cli` (for database migrations)

```sh
cargo install sqlx-cli
```

## Setup

1. Create the database and run migrations
```
mkdir -p db
sqlx database create --database-url sqlite://db/recipe.db
cargo run -- migrate
```

2. Seed the database from JSON
You can initialize your database with recipes from the provided JSON file once by running:
```
cargo run -- --init-from assets/static/recipes.json
```
This loads all recipes from the JSON into the SQLite database and then exits.

3. Run the server
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

