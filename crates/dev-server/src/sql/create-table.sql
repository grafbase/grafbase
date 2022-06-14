CREATE TABLE IF NOT EXISTS records (
    "pk" TEXT default NULL,
    "sk" TEXT default NULL,
    "type" TEXT not NULL,
    "document" JSON not NULL,
    "created_at" TEXT not NULL,
    "updated_at" TEXT not NULL,
    PRIMARY KEY("pk","sk")
);
