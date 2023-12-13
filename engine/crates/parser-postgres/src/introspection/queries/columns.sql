SELECT columns.table_schema                         AS schema,
       columns.table_name                           AS table_name,
       columns.column_name                          AS column_name,
       CAST(columns.udt_name::regtype::oid AS int4) AS type_id,
       columns.udt_name                             AS type_name,
       columns.udt_schema                           AS type_schema,
       columns.data_type = 'ARRAY'                  AS is_array,
       pg_attrdef.adbin IS NOT NULL                 AS has_default,
       columns.is_nullable = 'YES'                  AS is_nullable,
       columns.identity_generation                  AS identity_generation

FROM information_schema.columns columns

         -- for default values
         JOIN pg_attribute ON pg_attribute.attname = columns.column_name

         -- also for defaults
         JOIN (SELECT pg_class.oid,
                      relname,
                      pg_namespace.nspname AS namespace
               FROM pg_class
                        JOIN pg_namespace ON pg_namespace.oid = pg_class.relnamespace) AS pg_class
              ON pg_class.oid = pg_attribute.attrelid
                  AND pg_class.relname = columns.table_name
                  AND pg_class.namespace = columns.table_schema

         -- also for defaults
         LEFT OUTER JOIN pg_attrdef
                         ON pg_attrdef.adrelid = pg_attribute.attrelid
                             AND pg_attrdef.adnum = pg_attribute.attnum
                             AND pg_class.namespace = columns.table_schema

WHERE table_schema <> ALL ( $1 )
ORDER BY schema, table_name, columns.ordinal_position;
