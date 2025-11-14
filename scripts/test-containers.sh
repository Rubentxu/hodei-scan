#!/bin/bash
# Test Containers Manager for Hodei-Scan
# This script manages Docker containers for running tests that require infrastructure

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
CONTAINER_NAME_PREFIX="hodei-test"
POSTGRES_IMAGE="postgres:15-alpine"
POSTGRES_PORT=5433
POSTGRES_DB="hodei_test"
POSTGRES_USER="postgres"
POSTGRES_PASSWORD="postgres"

# Function to check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Error: Docker is not installed or not in PATH${NC}"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        echo -e "${RED}Error: Docker daemon is not running${NC}"
        exit 1
    fi
}

# Function to generate unique container name
generate_container_name() {
    local service=$1
    local timestamp=$(date +%s)
    local pid=$$
    echo "${CONTAINER_NAME_PREFIX}-${service}-${pid}-${timestamp}"
}

# Function to start PostgreSQL container
start_postgres() {
    local container_name=$(generate_container_name "postgres")
    echo -e "${YELLOW}Starting PostgreSQL container: ${container_name}${NC}"

    # Find available port starting from POSTGRES_PORT
    local port_to_use=${POSTGRES_PORT}
    local max_attempts=10
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        # Check if port is available using netstat or ss
        if command -v ss &> /dev/null; then
            if ss -tuln | grep -q ":${port_to_use} "; then
                echo -e "${YELLOW}Port ${port_to_use} is in use, trying $((port_to_use + 1))...${NC}"
                port_to_use=$((port_to_use + 1))
                attempt=$((attempt + 1))
                continue
            fi
        elif command -v lsof &> /dev/null; then
            if lsof -Pi :${port_to_use} -sTCP:LISTEN -t >/dev/null 2>&1; then
                echo -e "${YELLOW}Port ${port_to_use} is in use, trying $((port_to_use + 1))...${NC}"
                port_to_use=$((port_to_use + 1))
                attempt=$((attempt + 1))
                continue
            fi
        fi
        # Port is available, break out of loop
        break
    done

    if [ $attempt -eq $max_attempts ]; then
        echo -e "${RED}Failed to find an available port in range ${POSTGRES_PORT}-$((${POSTGRES_PORT} + max_attempts - 1))${NC}"
        echo -e "${YELLOW}Please run 'just --justfile Justfile container-cleanup' to remove old containers${NC}"
        return 1
    fi

    echo -e "${GREEN}Using port ${port_to_use}${NC}"

    # Clean up any existing container with the same name (ignore errors)
    docker rm -f "${container_name}" > /dev/null 2>&1 || true

    # Start container
    docker run -d \
        --name "${container_name}" \
        --network bridge \
        -e POSTGRES_PASSWORD=${POSTGRES_PASSWORD} \
        -e POSTGRES_DB=${POSTGRES_DB} \
        -p ${port_to_use}:5432 \
        ${POSTGRES_IMAGE}

    echo -e "${GREEN}PostgreSQL container started: ${container_name}${NC}"
    echo -e "Connection URL: postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${port_to_use}/${POSTGRES_DB}"
    echo -e "Container name: ${container_name}"

    # Wait for PostgreSQL to be ready
    echo -e "${YELLOW}Waiting for PostgreSQL to be ready...${NC}"
    for i in {1..30}; do
        if docker exec "${container_name}" pg_isready -U ${POSTGRES_USER} > /dev/null 2>&1; then
            echo -e "${GREEN}PostgreSQL is ready!${NC}"
            echo "${container_name}"
            return 0
        fi
        echo -n "."
        sleep 1
    done

    echo -e "${RED}PostgreSQL failed to start in time${NC}"
    docker logs "${container_name}"
    return 1
}

# Function to stop and remove container
stop_container() {
    local container_name=$1

    if [ -z "$container_name" ]; then
        echo -e "${RED}Error: Container name not provided${NC}"
        return 1
    fi

    echo -e "${YELLOW}Stopping container: ${container_name}${NC}"
    if docker ps -q -f name="${container_name}" | grep -q .; then
        docker stop "${container_name}" > /dev/null 2>&1
        docker rm "${container_name}" > /dev/null 2>&1
        echo -e "${GREEN}Container stopped and removed${NC}"
    else
        echo -e "${YELLOW}Container not running${NC}"
    fi
}

# Function to clean up all test containers
cleanup_all() {
    echo -e "${YELLOW}Cleaning up all test containers...${NC}"

    # Get all test containers (both running and stopped)
    local containers=$(docker ps -aq --filter "name=${CONTAINER_NAME_PREFIX}" 2>/dev/null | sort)

    if [ -z "$containers" ]; then
        echo -e "${GREEN}No test containers found${NC}"
        return 0
    fi

    echo -e "${YELLOW}Found containers:${NC}"
    docker ps -a --filter "name=${CONTAINER_NAME_PREFIX}" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

    for container in $containers; do
        echo -e "${YELLOW}Stopping and removing container: ${container}${NC}"
        docker stop "$container" > /dev/null 2>&1 || true
        docker rm "$container" > /dev/null 2>&1 || true
    done

    echo -e "${GREEN}All test containers cleaned up${NC}"
}

# Function to list test containers
list_containers() {
    echo -e "${YELLOW}Active test containers:${NC}"
    docker ps --filter "name=${CONTAINER_NAME_PREFIX}" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
}

# Function to run tests with PostgreSQL
run_tests_with_postgres() {
    local test_command=$*

    # Start PostgreSQL and capture both container name and port
    local container_info
    container_info=$(start_postgres)

    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to start PostgreSQL${NC}"
        exit 1
    fi

    # Extract container name (last line of output)
    local container_name=$(echo "$container_info" | tail -n 1)

    # Extract port from the docker run command output or container
    # For simplicity, we'll use docker inspect to get the port mapping
    local actual_port=$(docker port "${container_name}" 5432 2>/dev/null | cut -d':' -f2 | head -1)

    if [ -z "$actual_port" ]; then
        actual_port=${POSTGRES_PORT}
    fi

    export TEST_DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${actual_port}/${POSTGRES_DB}"

    # Initialize database schema
    echo -e "${YELLOW}Initializing database schema...${NC}"
    echo "CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    default_branch TEXT NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    branch TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    findings_count INTEGER NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS findings (
    id BIGSERIAL PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
    fact_type TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('critical', 'major', 'minor', 'info')),
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    column_number INTEGER NOT NULL,
    end_line INTEGER,
    end_column INTEGER,
    message TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    tags TEXT[] DEFAULT '{}',
    fingerprint TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS baseline_status (
    id BIGSERIAL PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    finding_fingerprint TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'accepted', 'wontfix', 'false_positive')),
    reason TEXT,
    expires_at TIMESTAMPTZ,
    updated_by UUID NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, finding_fingerprint)
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'developer', 'viewer')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_analyses_project_branch ON analyses(project_id, branch);
CREATE INDEX IF NOT EXISTS idx_analyses_timestamp ON analyses(timestamp);
CREATE INDEX IF NOT EXISTS idx_findings_analysis_id ON findings(analysis_id);
CREATE INDEX IF NOT EXISTS idx_findings_fingerprint ON findings(fingerprint);
CREATE INDEX IF NOT EXISTS idx_baseline_project ON baseline_status(project_id);

CREATE OR REPLACE VIEW findings_trend_daily AS
SELECT
    DATE_TRUNC('day', a.timestamp) as day,
    a.project_id,
    a.branch,
    f.severity,
    f.fact_type,
    COUNT(*) as count
FROM analyses a
JOIN findings f ON f.analysis_id = a.id
GROUP BY day, a.project_id, a.branch, f.severity, f.fact_type;" | docker exec -i "${container_name}" psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Database schema initialized successfully${NC}"
    else
        echo -e "${RED}Warning: Failed to initialize schema${NC}"
    fi

    # Create test project to satisfy foreign key constraints
    echo -e "${YELLOW}Creating test project...${NC}"
    echo "INSERT INTO projects (id, name, description, default_branch)
VALUES ('test-project', 'Test Project', 'Test project for baseline tests', 'main')
ON CONFLICT (id) DO NOTHING;" | docker exec -i "${container_name}" psql -U ${POSTGRES_USER} -d ${POSTGRES_DB} > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Test project created${NC}"
    else
        echo -e "${YELLOW}Warning: Failed to create test project${NC}"
    fi

    echo -e "${GREEN}Running tests with PostgreSQL...${NC}"
    echo -e "${YELLOW}Command: ${test_command}${NC}"
    echo -e "${YELLOW}Container: ${container_name}${NC}"
    echo -e "${YELLOW}Database URL: ${TEST_DATABASE_URL}${NC}"

    # Run the tests
    eval "$test_command"
    local test_result=$?

    # Clean up
    stop_container "$container_name"

    if [ $test_result -eq 0 ]; then
        echo -e "${GREEN}Tests passed!${NC}"
    else
        echo -e "${RED}Tests failed!${NC}"
    fi

    return $test_result
}

# Main command handling
case "${1:-}" in
    start-postgres)
        check_docker
        start_postgres
        ;;
    stop)
        check_docker
        stop_container "$2"
        ;;
    cleanup)
        check_docker
        cleanup_all
        ;;
    list)
        check_docker
        list_containers
        ;;
    run-tests)
        check_docker
        shift # Remove 'run-tests' from arguments
        run_tests_with_postgres "$@"
        ;;
    *)
        echo "Usage: $0 {start-postgres|stop|cleanup|list|run-tests <command>}"
        echo ""
        echo "Commands:"
        echo "  start-postgres     Start a PostgreSQL container for testing"
        echo "  stop <name>        Stop and remove a specific container"
        echo "  cleanup            Remove all test containers"
        echo "  list               List all active test containers"
        echo "  run-tests <cmd>    Run tests with PostgreSQL container"
        echo ""
        echo "Examples:"
        echo "  $0 start-postgres"
        echo "  $0 run-tests 'just --justfile justfile db'"
        echo "  $0 cleanup"
        exit 1
        ;;
esac
