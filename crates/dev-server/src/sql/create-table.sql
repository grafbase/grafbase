CREATE TABLE IF NOT EXISTS records (
    "pk" TEXT not NULL,
    "sk" TEXT not NULL,
    "gsi1pk" TEXT not NULL,
    "gsi1sk" TEXT not NULL,
    "gsi2pk" TEXT not NULL,
    "gsi2sk" TEXT not NULL,
    "type" TEXT not NULL,
    "document" JSON not NULL,
    "created_at" TEXT not NULL,
    "updated_at" TEXT not NULL,
    PRIMARY KEY("pk","sk")
);
