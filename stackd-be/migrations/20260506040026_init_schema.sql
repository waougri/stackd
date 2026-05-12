-- items: Already optimized for WITHOUT ROWID using TEXT IDs.
CREATE TABLE IF NOT EXISTS items (
                                     id TEXT PRIMARY KEY NOT NULL,
                                     name TEXT NOT NULL UNIQUE
) WITHOUT ROWID;

-- locations: Removed AUTOINCREMENT to allow WITHOUT ROWID.
-- Note: You must provide the ID manually or use a UUID.
CREATE TABLE IF NOT EXISTS locations (
                                         id INTEGER PRIMARY KEY NOT NULL,
                                         name TEXT NOT NULL,
                                         parent_id TEXT -- Changed to TEXT to match UUID IDs
) WITHOUT ROWID;

-- inventory: Optimization for a high-volume log table.
CREATE TABLE IF NOT EXISTS inventory (
                                         id TEXT PRIMARY KEY NOT NULL,
                                         timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                                         location_id TEXT NOT NULL REFERENCES locations(id),
                                         item_id TEXT NOT NULL REFERENCES items(id),
                                         action_type TEXT NOT NULL,
                                         value INTEGER NOT NULL
) WITHOUT ROWID;