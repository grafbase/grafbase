#!/bin/bash
set -e

# Function to create databases
create_databases() {
    local database_list=$1
    local databases=(${database_list//,/ })

    for db in "${databases[@]}"; do
        echo "Creating database: $db"
        psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
            CREATE DATABASE $db;
            GRANT ALL PRIVILEGES ON DATABASE $db TO $POSTGRES_USER;
EOSQL
        
        # Import schema if SQL file exists
        if [ -f "/docker-sql/${db}.sql" ]; then
            echo "Importing schema for $db from /docker-sql/${db}.sql"
            psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db" -f "/docker-sql/${db}.sql"
        fi
    done
}

# Wait for PostgreSQL to start
until pg_isready -U "$POSTGRES_USER"; do
    echo "Waiting for PostgreSQL to start..."
    sleep 1
done

# Create databases if environment variable is set
if [ -n "$POSTGRES_MULTIPLE_DATABASES" ]; then
    echo "Creating databases: $POSTGRES_MULTIPLE_DATABASES"
    create_databases "$POSTGRES_MULTIPLE_DATABASES"
    echo "Multiple databases created"
fi
