#!/bin/bash
# Run database migrations for HAL9

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Default values
DB_TYPE=${DATABASE_TYPE:-"postgres"}
MIGRATION_DIR=""
CONNECTION_URL=""

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --type TYPE       Database type (postgres|sqlite) [default: postgres]"
    echo "  -u, --url URL        Database connection URL"
    echo "  -d, --dir DIR        Migration directory [auto-detected]"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  DATABASE_URL         Database connection URL"
    echo "  DATABASE_TYPE        Database type (postgres|sqlite)"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            DB_TYPE="$2"
            shift 2
            ;;
        -u|--url)
            CONNECTION_URL="$2"
            shift 2
            ;;
        -d|--dir)
            MIGRATION_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Validate database type
if [[ "$DB_TYPE" != "postgres" && "$DB_TYPE" != "sqlite" ]]; then
    echo -e "${RED}Error: Invalid database type: $DB_TYPE${NC}"
    echo "Valid types are: postgres, sqlite"
    exit 1
fi

# Set connection URL
if [[ -z "$CONNECTION_URL" ]]; then
    CONNECTION_URL=${DATABASE_URL:-""}
fi

# Set default URLs if not provided
if [[ -z "$CONNECTION_URL" ]]; then
    if [[ "$DB_TYPE" == "postgres" ]]; then
        CONNECTION_URL="postgresql://postgres:postgres@localhost:5432/hal9_dev"
    else
        CONNECTION_URL="sqlite:data/hal9.db?mode=rwc"
    fi
fi

# Find migration directory
if [[ -z "$MIGRATION_DIR" ]]; then
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    SERVER_DIR="$(dirname "$SCRIPT_DIR")/architecture/server"
    MIGRATION_DIR="$SERVER_DIR/migrations/$DB_TYPE"
fi

# Check if migration directory exists
if [[ ! -d "$MIGRATION_DIR" ]]; then
    echo -e "${RED}Error: Migration directory not found: $MIGRATION_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}🗄️  HAL9 Database Migration Runner${NC}"
echo "======================================"
echo "Database Type: $DB_TYPE"
echo "Migration Dir: $MIGRATION_DIR"
echo "Connection URL: ${CONNECTION_URL//:*@/:***@}" # Hide password
echo ""

# Check for sqlx-cli
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}Warning: sqlx-cli not found. Installing...${NC}"
    cargo install sqlx-cli --features postgres,sqlite
fi

# Create database if it doesn't exist
echo -e "${GREEN}Creating database if needed...${NC}"
if [[ "$DB_TYPE" == "postgres" ]]; then
    sqlx database create --database-url "$CONNECTION_URL" 2>/dev/null || true
else
    # For SQLite, directory creation is handled by the connection string
    mkdir -p "$(dirname "${CONNECTION_URL#sqlite:}")" 2>/dev/null || true
fi

# Run migrations
echo -e "${GREEN}Running migrations...${NC}"
cd "$MIGRATION_DIR"

# List migration files
echo "Found migrations:"
for file in *.sql; do
    echo "  - $file"
done
echo ""

# Execute migrations based on type
if [[ "$DB_TYPE" == "postgres" ]]; then
    # For PostgreSQL, use sqlx migrate
    export DATABASE_URL="$CONNECTION_URL"
    sqlx migrate run
else
    # For SQLite, execute SQL files directly
    DB_FILE="${CONNECTION_URL#sqlite:}"
    DB_FILE="${DB_FILE%%\?*}"  # Remove query parameters
    
    for migration in $(ls -1 *.sql | sort); do
        echo "Applying $migration..."
        sqlite3 "$DB_FILE" < "$migration"
    done
fi

echo ""
echo -e "${GREEN}✅ Migrations completed successfully!${NC}"

# Show migration status
echo ""
echo "Migration status:"
if [[ "$DB_TYPE" == "postgres" ]]; then
    psql "$CONNECTION_URL" -c "SELECT version, description, installed_on, success FROM _sqlx_migrations ORDER BY version;" 2>/dev/null || true
else
    DB_FILE="${CONNECTION_URL#sqlite:}"
    DB_FILE="${DB_FILE%%\?*}"
    echo "SQLite migrations applied. Database file: $DB_FILE"
    echo "Tables created:"
    sqlite3 "$DB_FILE" ".tables" | tr ' ' '\n' | sort | column
fi