#!/bin/bash

set -e

# Load testing configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$PROJECT_ROOT/testing/config.env"

if [ -f "$CONFIG_FILE" ]; then
    source "$CONFIG_FILE"
else
    echo "Warning: Testing config file not found at $CONFIG_FILE"
    # Fallback to hardcoded values
    COMPOSE_FILE="docker-compose.test.yml"
    PROJECT_NAME="log-server-test"
    TEST_DOCKERFILE="testing/Dockerfile.test-runner"
    TEST_IMAGE_NAME="log-server-test-runner"
    NETWORK_NAME="log-server-test_log-server-test-network"
    TEST_BASE_URL="http://log-server-test-app:8080"
    HEALTH_CHECK_RETRIES=60
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

cleanup() {
    print_status "Cleaning up test environment..."
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME down -v --remove-orphans 2>/dev/null || true
    print_success "Test environment cleaned up"
}

setup() {
    print_status "Setting up test environment..."
    
    print_status "Building and starting test services..."
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME up --build -d
    
    print_status "Waiting for services to be ready..."
    
    local retries=${HEALTH_CHECK_RETRIES:-60}
    while [ $retries -gt 0 ]; do
        if curl -s -f http://localhost:8082/health >/dev/null 2>&1; then
            print_success "Services are ready!"
            return 0
        fi
        retries=$((retries - 1))
        if [ $retries -gt 0 ]; then
            echo -n "."
            sleep 2
        fi
    done
    
    print_error "Services failed to become ready"
    return 1
}

run_tests() {
    print_status "Running integration tests..."
    
    # Temporarily disable .dockerignore to include tests directory
    local dockerignore_backup=""
    if [ -f .dockerignore ]; then
        dockerignore_backup=$(mktemp)
        mv .dockerignore "$dockerignore_backup"
    fi

    # Build test runner using standard docker build with layer caching
    print_status "Building test container from $TEST_DOCKERFILE..."
    
    docker build \
        -f "$TEST_DOCKERFILE" \
        -t "$TEST_IMAGE_NAME" \
        . >/dev/null

    # Restore .dockerignore
    if [ -n "$dockerignore_backup" ]; then
        mv "$dockerignore_backup" .dockerignore
    fi

    # Run tests in containerized environment
    print_status "Executing tests..."
    docker run --rm \
        --network="$NETWORK_NAME" \
        -e TEST_BASE_URL="$TEST_BASE_URL" \
        "$TEST_IMAGE_NAME"

    print_success "Tests completed!"
}

logs() {
    print_status "Showing service logs..."
    docker compose -f $COMPOSE_FILE -p $PROJECT_NAME logs --tail=50
}

main() {
    case "${1:-full}" in
        setup)
            cleanup
            setup
            print_success "Test environment is ready. Run '$0 test' to execute tests."
            trap - EXIT
            ;;
        test)
            if ! curl -s -f http://localhost:8082/health >/dev/null 2>&1; then
                print_error "Test services are not running. Run '$0 setup' first."
                exit 1
            fi
            run_tests
            trap - EXIT
            ;;
        logs)
            logs
            trap - EXIT
            ;;
        cleanup)
            cleanup
            trap - EXIT
            ;;
        full)
            print_status "Running complete integration test workflow..."
            cleanup
            setup
            run_tests
            cleanup
            print_success "Integration testing workflow completed!"
            trap - EXIT
            ;;
        *)
            echo "Usage: $0 {setup|test|logs|cleanup|full}"
            echo ""
            echo "Commands:"
            echo "  setup   - Start test environment only"
            echo "  test    - Run tests against running environment"
            echo "  logs    - Show service logs"
            echo "  cleanup - Stop and clean test environment"
            echo "  full    - Complete workflow (default)"
            echo ""
            echo "Examples:"
            echo "  $0 full          # Complete test run"
            echo "  $0 setup && $0 test && $0 cleanup"
            trap - EXIT
            exit 1
            ;;
    esac
}

trap cleanup EXIT

main "$@"
