CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,  
    instructions TEXT NOT NULL,
    tags TEXT,
    source TEXT 
);
