# Development

*This section provides information for developers interested in contributing to Galactic War.*

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

# Build the project (with database features)
cargo build --features bin,db

# Build without database features
cargo build --features bin

# Run tests (including database tests)
cargo test --features db

# Start the development server with persistence
cargo run --features bin,db

# Start without persistence (in-memory only)
cargo run --features bin
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

### Core Library (`src/lib.rs`)
Contains the main game logic as a reusable library:
- Galaxy and system management
- Resource calculations
- Event processing
- Configuration handling

### Binary Application (`src/main.rs`)
HTTP server that exposes the library via REST API:
- Web server setup with Axum
- API route definitions
- WebSocket handling
- Static file serving

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

**Feature Flags**
- Use `--features db` to enable database functionality
- The `bin` feature enables the HTTP server binary
- Combine features: `--features bin,db` for full functionality

**Database Migrations**
```bash
# Run with automatic migrations (default)
cargo run --features bin,db

# Check migration status
cargo install sqlx-cli
sqlx migrate info --database-url sqlite:galactic_war.db
```

**Development Database**
```bash
# Use separate database for development
DATABASE_URL=sqlite:dev.db cargo run --features bin,db

# Reset development database
rm dev.db && cargo run --features bin,db
```

### Testing Database Features

**Database Test Coverage**
```bash
# Run all database tests
cargo test --features db db::

# Run integration tests with AppState
cargo test --features db app::tests

# Run specific persistence tests
cargo test --features db persistence::tests
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
1. Update database models in `src/models/`
2. Create migration file in `migrations/`
3. Update CRUD operations in `src/db/`
4. Add test coverage
5. Update persistence logic if needed

**Debugging Persistence Issues**
```bash
# Enable detailed persistence logging
RUST_LOG=galactic_war::persistence=debug cargo run --features bin,db

# Check database contents directly
sqlite3 galactic_war.db ".tables"
sqlite3 galactic_war.db "SELECT * FROM galaxies;"
```

**Performance Testing**
```bash
# Test with frequent saves
GALACTIC_WAR_AUTO_SAVE_INTERVAL=1 cargo run --features bin,db

# Monitor save performance
RUST_LOG=galactic_war::persistence=info cargo run --features bin,db
```

## Testing Strategy

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test system

# Run with output
cargo test -- --nocapture
```

### Integration Tests
Test the complete system with various configurations:
```bash
# Test with different galaxy configs
cargo run -- --config galaxies/classic.yaml
cargo run -- --config galaxies/blitz.yaml
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
1. Define the route in `src/main.rs`
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