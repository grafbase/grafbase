CREATE TABLE IF NOT EXISTS records (
    pk TEXT not NULL,
    sk TEXT not NULL,
    gsi1pk TEXT,
    gsi1sk TEXT,
    gsi2pk TEXT,
    gsi2sk TEXT,
    entity_type TEXT,
    relation_names JSON not NULL,
    document JSON not NULL,
    created_at TEXT not NULL,
    updated_at TEXT not NULL,
    PRIMARY KEY(pk,sk)
);
