-- Generic job table backing the in-process job runner. New job types register a
-- handler; no schema migration needed to add one (spec extension point).
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES assets(id),
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

ALTER TABLE assets ADD COLUMN thumbnail_hash TEXT;

-- EXIF taken-at stored as local time + optional UTC offset, never collapsed to a
-- single instant (see spec: assuming UTC silently corrupts sort when the
-- camera's timezone was wrong).
CREATE TABLE photo_metadata (
    asset_id TEXT PRIMARY KEY REFERENCES assets(id),
    taken_at_local TEXT,
    taken_at_offset_minutes INTEGER,
    camera TEXT,
    gps_lat REAL,
    gps_lon REAL
);
