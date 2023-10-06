SELECT
  pg_class.relname AS name,
  pg_namespace.nspname AS schema
FROM pg_class
INNER JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace
WHERE pg_class.relkind = 'r' -- r = relation, e.g. a table
AND pg_namespace.nspname <> ALL ( $1 )
ORDER BY schema, name;
