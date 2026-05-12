-- Add migration script here
CREATE TABLE inventory_new
(
    id          TEXT PRIMARY KEY NOT NULL,
    timestamp   DATETIME DEFAULT CURRENT_TIMESTAMP,
    location_id TEXT             NOT NULL REFERENCES locations (id),
    item_id     TEXT             NOT NULL REFERENCES items (id),
    action_type TEXT             NOT NULL CHECK (action_type IN
                                                 ('ADD',
                                                  'REMOVE',
                                                  'UPDATE')),
    value       INTEGER          NOT NULL,
    move_id     TEXT
) WITHOUT ROWID;

INSERT INTO inventory_new
SELECT id, timestamp, location_id, item_id, action_type, value, move_id
FROM inventory;

DROP TABLE inventory;
ALTER TABLE inventory_new
    RENAME TO inventory;

CREATE INDEX IF NOT EXISTS idx_inventory_item_location
    ON inventory (item_id, location_id);