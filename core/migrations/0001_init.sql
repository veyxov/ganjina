CREATE TABLE owners (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE assets (
    id TEXT PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    original_filename TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    owner_id TEXT REFERENCES owners(id),
    created_at TEXT NOT NULL
);
