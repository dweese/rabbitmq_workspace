#!/bin/bash
# check_psql.sh

# Configuration
DB_HOST="${PGHOST:-localhost}"
DB_PORT="${PGPORT:-5432}"
DB_USER="${PGUSER:-postgres}"
DB_NAME="${PGDATABASE:-postgres}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Checking PostgreSQL connection..."
echo "Host: $DB_HOST:$DB_PORT"
echo "User: $DB_USER"
echo "Database: $DB_NAME"
echo "---"

# Test connection with a simple query
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT version();" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ PostgreSQL is alive and responding!${NC}"
    
    # Show some basic info
    echo
    echo "📊 Server Info:"
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT version();" 2>/dev/null | head -1
    
    echo
    echo "📈 Connection Stats:"
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
        SELECT 
            'Active connections: ' || count(*) 
        FROM pg_stat_activity 
        WHERE state = 'active';" 2>/dev/null
    
    exit 0
else
    echo -e "${RED}❌ PostgreSQL is not responding!${NC}"
    echo
    echo "Possible issues:"
    echo "• PostgreSQL service is not running"
    echo "• Wrong connection parameters"
    echo "• Authentication failure"
    echo "• Network connectivity issues"
    
    # Try to give more specific error info
    echo
    echo "🔧 Troubleshooting:"
    echo "• Check if PostgreSQL is running: sudo systemctl status postgresql"
    echo "• Test basic connectivity: pg_isready -h $DB_HOST -p $DB_PORT"
    echo "• Check logs: sudo journalctl -u postgresql"
    
    exit 1
fi