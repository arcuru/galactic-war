# Development

_This section provides information for developers interested in contributing to Galactic War._

## Development Setup

### Prerequisites

- **Rust** (latest stable version)
- **Cargo** (included with Rust)
- **Git** for version control
- **mdbook** for documentation (optional)

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/arcuru/galactic-war.git
cd galactic-war

# Build the entire workspace
cargo build --workspace

# Build specific crates
cargo build -p galactic-war      # Library crate only
cargo build -p galactic-war-bin  # Binary crate only

# Run tests across workspace (including database tests)
cargo nextest run --workspace

# Start the development server (includes database features by default)
cargo run --bin galactic-war

# Alternative using task runner
task run
```

### Development Dependencies

The project uses several key Rust crates:

- **axum** - Web server framework
- **serde** - Serialization/deserialization
- **tokio** - Async runtime
- **serde_yaml** - YAML configuration parsing
- **indexmap** - Ordered maps for consistent iteration
- **sqlx** - Database access and migrations (optional, with 'db' feature)
- **chrono** - Date and time handling for persistence

## Project Structure

The project is organized as a Cargo workspace with two main crates:

### Library Crate (`crates/lib/`)

**Package name**: `galactic-war`  
Contains the core game logic as a reusable library:

- Galaxy and system management (`src/game_system.rs`)
- Resource calculations and event processing
- Database persistence layer (`src/db/`, `src/models/`)
- Application state management (`src/app.rs`)
- Configuration handling (`src/config.rs`)

### Binary Crate (`crates/bin/`)

**Package name**: `galactic-war-bin` (produces `galactic-war` binary)  
HTTP server that exposes the library via REST API:

- Web server setup with Axum (`src/main.rs`)
- API route definitions and handlers
- Static file serving and templating
- Configuration loading and environment setup

### Configuration System (`src/config.rs`)

Handles YAML-based game configuration:

- Galaxy parameters
- Structure definitions
- Resource and cost scaling
- Game mode customization

### System Implementation (`src/system.rs`)

Individual solar system logic:

- Structure management
- Event scheduling and processing
- Resource production
- Construction handling

### Database Persistence (`src/app.rs`, `src/persistence.rs`, `src/db/`, `src/models/`)

Real-time database persistence system (optional, enabled with 'db' feature):

**Application State Management (`src/app.rs`)**

- `AppState` coordinates between in-memory state and database
- Automatic galaxy loading from database when accessed
- Background persistence with configurable intervals
- Graceful degradation when database unavailable

**Persistence Manager (`src/persistence.rs`)**

- Background worker for periodic auto-saves
- Write coalescing and batching optimization
- Dirty tracking to minimize database writes
- Shutdown handling with final saves

**Database Layer (`src/db/`)**

- Database connection and migration management
- CRUD operations for galaxies, systems, structures, events
- SQLite integration with connection pooling
- Error handling and recovery

**Data Models (`src/models/`)**

- Database row structures for all entities
- Type conversion between database (i64) and game (usize) types
- Serialization support for complex game data

## Contributing Guidelines

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Include unit tests for new functionality

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Implement your changes with tests
4. Update documentation if needed
5. Submit a pull request with clear description

### Issue Reporting

When reporting bugs or suggesting features:

- Provide clear reproduction steps
- Include relevant configuration details
- Describe expected vs. actual behavior
- Add screenshots if applicable

## Database Development

### Working with Database Persistence

**Environment Setup**

```bash
# Set database URL (optional, defaults to sqlite:galactic_war.db)
export DATABASE_URL=sqlite:dev.db

# Configure persistence settings
export GALACTIC_WAR_AUTO_SAVE_INTERVAL=10  # Save every 10 seconds in dev
export RUST_LOG=info                       # Enable persistence logging
```

**Feature Flags (Library Crate)**

- `db`: Enables database persistence functionality (SQLite, migrations, auto-save)
- `bin`: Enables web server dependencies (axum, tokio) for library use

The binary crate automatically includes database features by default.

**Database Migrations**

```bash
# Run with automatic migrations (default)
cargo run --bin galactic-war

# Check migration status
cargo install sqlx-cli
sqlx migrate info --database-url sqlite:galactic_war.db
```

**Development Database**

```bash
# Use separate database for development
DATABASE_URL=sqlite:dev.db cargo run --bin galactic-war

# Reset development database
rm dev.db && cargo run --bin galactic-war
```

### Testing Database Features

**Database Test Coverage**

```bash
# Run all tests (database tests included in workspace)
cargo nextest run --workspace

# Run library tests only (includes database tests)
cargo nextest run -p galactic-war

# Run specific test modules
cargo nextest run -p galactic-war db::
cargo nextest run -p galactic-war app::tests
cargo nextest run -p galactic-war persistence::tests
```

**Test Database Isolation**

- Tests use in-memory SQLite databases (`sqlite::memory:`)
- Each test gets a fresh database instance
- No cleanup required between tests

**Adding Database Tests**

```rust
#[tokio::test]
async fn test_my_feature() {
    let db = Database::new_test().await;
    // Your test logic here
}
```

### Common Development Tasks

**Adding New Database Fields**

1. Update database models in `crates/lib/src/models/`
2. Create migration file in `crates/lib/migrations/`
3. Update CRUD operations in `crates/lib/src/db/`
4. Add test coverage
5. Update persistence logic if needed

**Debugging Persistence Issues**

```bash
# Enable detailed persistence logging
RUST_LOG=galactic_war::persistence=debug cargo run --bin galactic-war

# Check database contents directly
sqlite3 galactic_war.db ".tables"
sqlite3 galactic_war.db "SELECT * FROM galaxies;"
```

**Performance Testing**

```bash
# Test with frequent saves
GALACTIC_WAR_AUTO_SAVE_INTERVAL=1 cargo run --bin galactic-war

# Monitor save performance
RUST_LOG=galactic_war::persistence=info cargo run --bin galactic-war
```

## Testing Strategy

### Unit Tests

```bash
# Run all tests across workspace
cargo nextest run --workspace

# Run tests for specific crate
cargo nextest run -p galactic-war      # Library tests
cargo nextest run -p galactic-war-bin  # Binary tests

# Run specific test module
cargo nextest run -p galactic-war system

# Run with output
cargo nextest run --workspace -- --nocapture
```

### Integration Tests

Test the complete system with various configurations:

```bash
# Test with different galaxy configs (from binary crate)
cargo run --bin galactic-war -- --config galaxies/classic.yaml
cargo run --bin galactic-war -- --config galaxies/blitz.yaml
```

### Configuration Testing

Create custom test configurations for specific scenarios:

```yaml
# test-config.yaml
system_count: 10
size: { x: 10, y: 10 }
systems:
  resources: { crew: 100, metal: 100, water: 100 }
  # ... minimal structure definitions
```

## API Development

### Adding New Endpoints

1. Define the route in `crates/bin/src/main.rs`
2. Implement the handler function
3. Add appropriate error handling
4. Update API documentation
5. Add integration tests

### WebSocket Updates

Real-time updates are sent via WebSocket when:

- System state changes
- Events complete
- Errors occur

Example WebSocket event:

```rust
// In your update function
websocket_send(SystemUpdate {
    system: coords,
    data: updated_system_info,
});
```

## Performance Considerations

### Memory Usage

- Entire galaxy state is kept in memory
- Consider memory usage when adding features
- Use efficient data structures (Vec, HashMap)
- Avoid unnecessary cloning

### CPU Efficiency

- Event processing happens every tick
- Optimize hot code paths
- Use profiling tools to identify bottlenecks
- Consider batch processing for bulk operations

### Network Optimization

- Minimize API response sizes
- Use WebSocket for real-time updates
- Implement appropriate caching headers
- Consider request rate limiting

## Release Process

### Version Management

- Use semantic versioning (MAJOR.MINOR.PATCH)
- Tag releases in git
- Update CHANGELOG.md
- Consider feature flags for experimental features

### Deployment

- Docker containers for easy deployment
- Configuration via environment variables
- Health check endpoints
- Graceful shutdown handling

### Docker Compose Example

Use Docker Compose to run the game with persistent database storage:

```yaml
version: "3.8"
services:
  galactic-war:
    image: galactic-war:latest
    ports:
      - "3050:3050"
    volumes:
      - ./data:/app/data
      - ./galaxies:/app/galaxies
    environment:
      - GALACTIC_WAR_PERSISTENCE=true
      - GALACTIC_WAR_AUTO_SAVE_INTERVAL=30
      - GALACTIC_WAR_WRITE_COALESCING=true
      - DATABASE_URL=sqlite:/app/data/galactic_war.db
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3050/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  data:
    driver: local
```

This configuration:

- Maps port 3050 for web access
- Persists database in `./data/` directory
- Mounts galaxy configurations from `./galaxies/`
- Enables auto-save every 30 seconds
- Includes health checks for monitoring
- Automatically restarts on failure

## Future Development Areas

### Priority Features

1. Fleet and combat system
2. User authentication
3. Multi-galaxy management improvements
4. Mobile optimization

### Technical Debt

- Improve error handling consistency
- Add more comprehensive logging
- Enhance test coverage
- Optimize performance bottlenecks

### Architecture Improvements

- Microservices architecture for scaling
- Advanced database features (PostgreSQL, compression)
- Caching layer for performance
- Load balancing for high availability

## Community

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General discussion and questions
- **Pull Requests** - Code contributions and reviews

### Getting Help

- Check existing documentation
- Search through GitHub issues
- Create a new issue with detailed information
- Join community discussions

The Galactic War project welcomes contributions from developers of all skill levels. Whether you're fixing bugs, adding features, or improving documentation, your contributions help make the game better for everyone.
