WITH rawindex AS (SELECT indrelid,
                         indexrelid,
                         indisprimary,
                         unnest(indkey)                 AS indkeyid,
                         generate_subscripts(indkey, 1) AS indkeyidx
                  FROM pg_index
                  WHERE indpred IS NULL -- filter out partial indexes
                    AND NOT indisexclusion -- filter out exclusion constraints
                    AND (indisunique OR indisprimary)
)
SELECT schemainfo.nspname    AS schema,
       indexinfo.relname     AS constraint_name,
       tableinfo.relname     AS table_name,
       columninfo.attname    AS column_name,
       rawindex.indisprimary AS is_primary_key
FROM rawindex

INNER JOIN pg_class AS tableinfo ON tableinfo.oid = rawindex.indrelid
INNER JOIN pg_class AS indexinfo ON indexinfo.oid = rawindex.indexrelid
INNER JOIN pg_namespace AS schemainfo ON schemainfo.oid = tableinfo.relnamespace

LEFT JOIN pg_attribute AS columninfo
    ON columninfo.attrelid = tableinfo.oid AND columninfo.attnum = rawindex.indkeyid

WHERE schemainfo.nspname <> ALL ( $1 )
ORDER BY schema, table_name, constraint_name, rawindex.indkeyidx;
