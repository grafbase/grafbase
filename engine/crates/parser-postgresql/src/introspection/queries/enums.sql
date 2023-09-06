SELECT 
  pg_namespace.nspname AS schema,
  pg_type.typname      AS enum_name,
  pg_enum.enumlabel    AS enum_value
FROM pg_type
JOIN pg_enum ON pg_type.oid = pg_enum.enumtypid
JOIN pg_namespace ON pg_namespace.oid = pg_type.typnamespace
WHERE pg_namespace.nspname <> ALL ( $1 )
ORDER BY pg_enum.enumsortorder;
