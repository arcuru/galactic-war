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

# Build the project
cargo build

# Run tests
cargo test

# Start the development server
cargo run -- --config galaxies/classic.yaml
```

### Development Dependencies
The project uses several key Rust crates:
- **axum** - Web server framework
- **serde** - Serialization/deserialization
- **tokio** - Async runtime
- **serde_yaml** - YAML configuration parsing
- **indexmap** - Ordered maps for consistent iteration

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
3. Database persistence
4. Mobile optimization

### Technical Debt
- Improve error handling consistency
- Add more comprehensive logging
- Enhance test coverage
- Optimize performance bottlenecks

### Architecture Improvements
- Microservices architecture for scaling
- Database integration for persistence
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