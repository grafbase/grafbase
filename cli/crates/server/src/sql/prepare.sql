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
    owned_by TEXT,
    PRIMARY KEY(pk, sk)
);

CREATE TABLE IF NOT EXISTS modifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    modification_type TEXT not NULL,
    approximate_creation_date_time INTEGER not NULL,
    pk_old TEXT,
    sk_old TEXT,
    gsi1pk_old TEXT,
    gsi1sk_old TEXT,
    gsi2pk_old TEXT,
    gsi2sk_old TEXT,
    entity_type_old TEXT,
    relation_names_old JSON,
    document_old JSON,
    created_at_old TEXT,
    updated_at_old TEXT,
    pk_new TEXT,
    sk_new TEXT,
    gsi1pk_new TEXT,
    gsi1sk_new TEXT,
    gsi2pk_new TEXT,
    gsi2sk_new TEXT,
    entity_type_new TEXT,
    relation_names_new JSON,
    document_new JSON,
    created_at_new TEXT,
    updated_at_new TEXT
);

CREATE TRIGGER IF NOT EXISTS update_trigger
AFTER
UPDATE
    ON records BEGIN
INSERT INTO
    modifications (
        modification_type,
        approximate_creation_date_time,
        pk_old,
        sk_old,
        gsi1pk_old,
        gsi1sk_old,
        gsi2pk_old,
        gsi2sk_old,
        entity_type_old,
        relation_names_old,
        document_old,
        created_at_old,
        updated_at_old,
        pk_new,
        sk_new,
        gsi1pk_new,
        gsi1sk_new,
        gsi2pk_new,
        gsi2sk_new,
        entity_type_new,
        relation_names_new,
        document_new,
        created_at_new,
        updated_at_new
    )
VALUES
    (
        'UPDATE',
        unixepoch(),
        old.pk,
        old.sk,
        old.gsi1pk,
        old.gsi1sk,
        old.gsi2pk,
        old.gsi2sk,
        old.entity_type,
        old.relation_names,
        old.document,
        old.created_at,
        old.updated_at,
        new.pk,
        new.sk,
        new.gsi1pk,
        new.gsi1sk,
        new.gsi2pk,
        new.gsi2sk,
        new.entity_type,
        new.relation_names,
        new.document,
        new.created_at,
        new.updated_at
    );

END;

CREATE TRIGGER IF NOT EXISTS insert_trigger
AFTER
INSERT
    ON records BEGIN
INSERT INTO
    modifications (
        modification_type,
        approximate_creation_date_time,
        pk_old,
        sk_old,
        gsi1pk_old,
        gsi1sk_old,
        gsi2pk_old,
        gsi2sk_old,
        entity_type_old,
        relation_names_old,
        document_old,
        created_at_old,
        updated_at_old,
        pk_new,
        sk_new,
        gsi1pk_new,
        gsi1sk_new,
        gsi2pk_new,
        gsi2sk_new,
        entity_type_new,
        relation_names_new,
        document_new,
        created_at_new,
        updated_at_new
    )
VALUES
    (
        'INSERT',
        unixepoch(),
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        new.pk,
        new.sk,
        new.gsi1pk,
        new.gsi1sk,
        new.gsi2pk,
        new.gsi2sk,
        new.entity_type,
        new.relation_names,
        new.document,
        new.created_at,
        new.updated_at
    );

END;

CREATE TRIGGER IF NOT EXISTS delete_trigger
AFTER
    DELETE ON records BEGIN
INSERT INTO
    modifications(
        modification_type,
        approximate_creation_date_time,
        pk_old,
        sk_old,
        gsi1pk_old,
        gsi1sk_old,
        gsi2pk_old,
        gsi2sk_old,
        entity_type_old,
        relation_names_old,
        document_old,
        created_at_old,
        updated_at_old,
        pk_new,
        sk_new,
        gsi1pk_new,
        gsi1sk_new,
        gsi2pk_new,
        gsi2sk_new,
        entity_type_new,
        relation_names_new,
        document_new,
        created_at_new,
        updated_at_new
    )
VALUES
    (
        'DELETE',
        unixepoch(),
        old.pk,
        old.sk,
        old.gsi1pk,
        old.gsi1sk,
        old.gsi2pk,
        old.gsi2sk,
        old.entity_type,
        old.relation_names,
        old.document,
        old.created_at,
        old.updated_at,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL
    );

END;
