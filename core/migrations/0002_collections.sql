CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE collection_assets (
    collection_id TEXT NOT NULL REFERENCES collections(id),
    asset_id TEXT NOT NULL REFERENCES assets(id),
    PRIMARY KEY (collection_id, asset_id)
);

-- Generic, type-agnostic link between any two assets (e.g. a note and the photo
-- it's about). Undirected: (a, b) and (b, a) are the same link, enforced by
-- always storing the pair in a canonical order (asset_a < asset_b).
CREATE TABLE asset_links (
    id TEXT PRIMARY KEY,
    asset_a TEXT NOT NULL REFERENCES assets(id),
    asset_b TEXT NOT NULL REFERENCES assets(id),
    created_at TEXT NOT NULL,
    UNIQUE (asset_a, asset_b)
);
