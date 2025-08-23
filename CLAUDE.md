# Sundial - Rust CLI Project

## Project Goal
A Rust CLI application that controls screen temperature (via hyprsunseet) based on sunrise/sunset times.

## Architecture

### Module Structure
```
src/
├── main.rs       # Application struct + CLI entry point + system calls
├── config.rs     # Configuration management (TOML + directories)
├── cache.rs      # API response caching (date-based files)
├── sun_times.rs  # Sunrise/sunset API integration
└── screen.rs     # Screen state calculations
```

### Design Patterns
- **Application struct**: Centralized coordination and state management
- **Pure functions**: Modules provide stateless utilities
- **Dependency injection**: `Application::with_config()` for testing
- **Error propagation**: `Result<T, Box<dyn std::error::Error>>` with `?` operator

## Rust Coding Standards

### Error Handling
- Use `Result<T, Box<dyn std::error::Error>>` for all fallible functions
- Propagate errors with `?` operator
- Return `Box<dyn std::error::Error>` for simplicity over custom error types

### Module Organization
- Each module owns its domain structs and logic
- Export only what's needed: `pub` for external use, private by default
- Use specific imports: `use module::{Type, function}` instead of `use module::*`
- Test helpers: `#[cfg(test)] pub fn get_test_config()` pattern

### Testing Strategy
- Unit tests in each module's `#[cfg(test)] mod tests`
- Test pure functions in isolation with mock data
- Use temporary directories for file I/O tests
- Focus on domain logic, avoid complex integration testing

### Code Style
- Use `#[derive(Debug, Serialize, Deserialize)]` for data structures
- Use descriptive variable names: `config_dir`, `sun_times`, `screen_state`

### Application Pattern
```rust
struct Application {
    config: Config,
    data_dir: PathBuf,
}

impl Application {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> { ... }
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> { ... }
}
```

## Communication Guidelines
- Direct, concise communication without unnecessary preamble
- Focus on technical content and problem-solving
- Provide context for design decisions

## Development Workflow
- Test frequently: `cargo test` after changes
- Commit working increments
- Use strangler pattern for major refactors
- Verify functionality with real sunrise/sunset data

## Testing
Run tests: `cargo test`
- Mock API responses for reliable testing
- File I/O testing with temporary directories
- Each module tests its own domain logic
