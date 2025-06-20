#!/bin/bash
# Setup database tables for HAL9 server

set -e

echo "🔧 Setting up HAL9 database..."
echo

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    # Use SQLite by default for development
    export DATABASE_URL="sqlite://./hal9.db"
    echo "ℹ️  Using SQLite database: $DATABASE_URL"
else
    echo "ℹ️  Using database: $DATABASE_URL"
fi

# Determine database type
if [[ "$DATABASE_URL" == postgres* ]]; then
    DB_TYPE="postgres"
elif [[ "$DATABASE_URL" == sqlite* ]]; then
    DB_TYPE="sqlite"
else
    echo "❌ Unsupported database type in DATABASE_URL"
    exit 1
fi

echo
echo "📋 Running migrations for $DB_TYPE..."

# Run migrations based on database type
MIGRATION_DIR="layers/L3_operational/architecture/server/migrations/$DB_TYPE"

if [ ! -d "$MIGRATION_DIR" ]; then
    echo "❌ Migration directory not found: $MIGRATION_DIR"
    exit 1
fi

# For SQLite, create the database file if it doesn't exist
if [ "$DB_TYPE" = "sqlite" ]; then
    DB_FILE=$(echo $DATABASE_URL | sed 's/sqlite:\/\///')
    mkdir -p $(dirname $DB_FILE)
    touch $DB_FILE
fi

# Run each migration file
for migration in $(ls $MIGRATION_DIR/*.sql | sort); do
    echo "  → Running $(basename $migration)..."
    
    if [ "$DB_TYPE" = "postgres" ]; then
        psql "$DATABASE_URL" < "$migration" || {
            echo "❌ Failed to run migration: $migration"
            exit 1
        }
    else
        sqlite3 "${DB_FILE}" < "$migration" || {
            echo "❌ Failed to run migration: $migration"
            exit 1
        }
    fi
done

echo
echo "✅ Database setup complete!"

# Show table count
if [ "$DB_TYPE" = "postgres" ]; then
    TABLE_COUNT=$(psql -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" "$DATABASE_URL" | tr -d ' ')
    echo "📊 Created $TABLE_COUNT tables"
else
    TABLE_COUNT=$(sqlite3 "${DB_FILE}" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';" | tr -d ' ')
    echo "📊 Created $TABLE_COUNT tables"
fi