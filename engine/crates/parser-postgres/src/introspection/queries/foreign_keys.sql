SELECT "constraint".conname       AS constraint_name,
       "constraint".schema        AS constrained_schema,
       "constraint".table_name    AS constrained_table_name,
       child_attribute.attname    AS constrained_column_name,
       relation_namespace.nspname AS referenced_schema,
       parent_class.relname       AS referenced_table_name,
       parent_attribute.attname   AS referenced_column_name

FROM (SELECT pg_namespace.nspname                         AS schema,
             unnest(pg_constraint.conkey)                 AS child, -- list of constrained columns
             unnest(pg_constraint.confkey)                AS parent, -- list of referenced columns
             pg_class.relname                             AS table_name,
             pg_namespace.nspname                         AS schema_name,
             generate_subscripts(pg_constraint.conkey, 1) AS conkey_idx,
             pg_constraint.oid,
             pg_constraint.confrelid,
             pg_constraint.conrelid,
             pg_constraint.conname
      FROM pg_class
               JOIN pg_constraint ON pg_constraint.conrelid = pg_class.oid
               JOIN pg_namespace ON pg_class.relnamespace = pg_namespace.oid
      WHERE pg_constraint.contype = 'f' -- f = foreign key
      ORDER BY conkey_idx) "constraint"

JOIN pg_attribute parent_attribute
  ON parent_attribute.attrelid = "constraint".confrelid
  AND parent_attribute.attnum = "constraint".parent
JOIN pg_class parent_class
  ON parent_class.oid = "constraint".confrelid
JOIN pg_attribute child_attribute
  ON child_attribute.attrelid = "constraint".conrelid
  AND child_attribute.attnum = "constraint".child
JOIN pg_class child_class
  ON "constraint".confrelid = child_class.oid
JOIN pg_namespace relation_namespace
  ON child_class.relnamespace = relation_namespace.oid

WHERE "constraint".conname <> ALL ( $1 )

-- order matters, be careful if changing
ORDER BY constrained_schema, constrained_table_name, constraint_name, "constraint".conkey_idx;
