-- Add migration script here
-- enforce valid action types
ALTER TABLE inventory ADD COLUMN move_id TEXT;

CREATE INDEX IF NOT EXISTS idx_inventory_item_location
    ON inventory(item_id, location_id);

-- image support
ALTER TABLE items ADD COLUMN image_path TEXT;