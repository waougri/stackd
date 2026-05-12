-- 1. Create the new table with the correct types
CREATE TABLE IF NOT EXISTS locations_new (
                                             id TEXT PRIMARY KEY NOT NULL,
                                             name TEXT NOT NULL,
                                             parent_id TEXT
) WITHOUT ROWID;

-- 2. Move data from the old table to the new one
-- Note: existing INTEGER IDs will be stored as strings (e.g., "1", "2")
INSERT INTO locations_new (id, name, parent_id)
SELECT CAST(id AS TEXT), name, parent_id FROM locations;

-- 3. Drop the old table
-- CAUTION: This will break existing Foreign Key constraints temporarily
DROP TABLE locations;

-- 4. Rename the new table to the original name
ALTER TABLE locations_new RENAME TO locations;

-- 5. Re-create your Inventory table or update its references
-- Since inventory.location_id is already TEXT, you just need to ensure
-- the Foreign Key constraint points to the new locations table.