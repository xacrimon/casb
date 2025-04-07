-- cached data for all indexes in the repository
CREATE TABLE index_set  (
    id INTEGER NOT NULL PRIMARY KEY,
    storage_id TEXT UNIQUE NOT NULL,
    data BLOB NOT NULL
) STRICT;

-- maps the 4 byte prefix of a blob id to any known indexes that reference it
CREATE TABLE blob_prefix_map (
    prefix INTEGER NOT NULL,
    index_id INTEGER NOT NULL -- no foreign key to avoid a large and unused index
) STRICT;
