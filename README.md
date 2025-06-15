# Recipe Server
**ROha Khan**
This software is wrtten in order to learn Rust full stack web basics.
A simple Rust web server using Tokio, Axum, Askama, and SQLx with SQLite that serves 
random recipes from a database.

---

## Features Implemented (or attempted to so far)

- Serves HTML-rendered recipes from a local SQLite database
- Supports querying a recipe by its `id` via URL query string (e.g. `/?id=my_recipe_id`)
- Handles optional `source` field and displays it if available
- Handles optional `tags` field (display logic implemented, but tag rendering may not be fully functional yet)
- Includes fallback hardcoded recipe if database lookup fails
- Loads data from a JSON file on first run (via `--init-from`)
- Fully working database migrations using SQLx

---

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


---

## Project Structure

src/main.rs           
- App entry point, routing, state

src/recipe.rs         
- Recipe model & JSON loader

src/templates.rs      
- Askama template data structs

src/error.rs          
- Custom error types

assets/static/recipe.css    
- App styling

assets/static/recipes.json  
- Initial data source

assets/templates/index.html 
- Basic html templating for Askama

migrations/0001...sql        
- Database schema definition


---

## Known Limitations
- Tag data is stored and parsed, but currently not appearing in HTML. This will be revisited in a future update.
- No REST API endpoints yet — upcoming in the next development phase.

---

## Notes
At this point I see a "set `DATABASE_URL` to use query macros online..." (I'm using VS-code & rut-analyzer), however is working in spite of this!!


---

## Liscence 
MIT


---

## Development Environment
- OS Name: Microsoft Windows 11 (10.0.26100 N/A Build 26100)
- Rust: rustc 1.86.0 (05f9846f8 2025-03-31)
- Toolchain: stable-x86_64-pc-windows-msvc (default)

---

**# rest-api-start Branch**
This branch adds a fully functional, Utoipa-documented REST API to the existing recipe server. It is now merged with main, so everything described below is reflected in the main branch. The original early stop version has been saved in a third branch. 



---

## Features Implemented

REST API Endpoints:
- GET /api/v1/recipe/{recipe_id}
    Returns a recipe by its ID in JSON format
- GET /api/v1/random-recipe
    Returns a random recipe from the database

OpenAPI Documentation (via utoipa):
- Integrated Swagger UI (/swagger-ui)
- Also includes Redoc (/redoc) and RapiDoc (/rapidoc)
- Traceable Server Logging using tower-http and tracing_subscriber
- CORS Handling for cross-origin API requests
- Codebase Refactoring for modular api.rs route handling and documentation


---

## Known Limitations

The /tagged-joke equivalent route (i.e., filtering recipes by tag) was intentionally excluded, as tag-based queries aren't yet working reliably in this version.

Swagger UI required a temporary patch via a Git dependency (linked directly to the latest Utoipa commit) due to a documented upstream issue.

No authentication or JWT-based API write support is implemented on this branch.
