
CREATE TABLE items (
                       id TEXT PRIMARY KEY NOT NULL,
                       name TEXT NOT NULL UNIQUE
);

CREATE TABLE locations (
                           id INTEGER PRIMARY KEY AUTOINCREMENT,
                           name TEXT NOT NULL,
                           parent_id INTEGER
);

CREATE TABLE inventory (
                           id INTEGER PRIMARY KEY AUTOINCREMENT,
                           timestamp INTEGER NOT NULL,
                           location_id INTEGER NOT NULL REFERENCES locations(id),
                           item_id TEXT NOT NULL REFERENCES items(id),
                           action_type TEXT NOT NULL,
                           value INTEGER NOT NULL
);
