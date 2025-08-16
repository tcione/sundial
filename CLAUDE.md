# Sundial - Rust Learning Project

## Project Goal
Rewriting a bash script (`original-sundial.sh`) that controls screen temperature (blue light filter) based on sunrise/sunset times in Rust for learning purposes.

## Original Bash Script Functionality
See `original-sundial.sh` for the complete implementation. Key features:
- Fetches sunrise/sunset data for Berlin from API (`api.sunrisesunset.io`)
- Compares current time to determine day/night
- Sets screen temperature: 6000K (day) / 2800K (night) + gamma values
- Manages hyprsunset process and avoids unnecessary updates
- Persists state in `~/.sundial-temperature`
- Uses `hyprctl` commands to control temperature/gamma

## Learning Approach
- Pair programming: Claude as navigator, human types all code
- Adding dependencies incrementally as needed
- Focusing on understanding concepts deeply

## Current Progress

### ✅ Completed
1. **Project Analysis** - Understood bash script functionality
2. **Project Setup** - Created Cargo.toml with reqwest dependency
3. **HTTP Client Decision** - Chose synchronous approach using `reqwest::blocking`
4. **Basic Structure** - Created main.rs with TODO roadmap
5. **HTTP Function** - Started `fetch_sunrise_sunset()` function

### 🔄 Current Step
**Implementing HTTP client for sunrise/sunset API**
- Added `fetch_sunrise_sunset()` function 
- Using `Result<String, Box<dyn std::error::Error>>` return type
- Function needs small fix: missing `return` in `Ok(text);`

### 📚 Concepts Learned
- **Result<T, E>**: Sum type/tagged union for error handling
- **Box<T>**: Heap allocation for unknown-sized types
- **dyn trait**: Trait objects for dynamic dispatch
- **? operator**: Error propagation
- **format! macro**: String interpolation

### 📋 Next Steps
1. Fix the return statement in `fetch_sunrise_sunset()`
2. Test the HTTP function
3. Add JSON parsing (will need serde)
4. Implement time comparison logic
5. Add file I/O for state management
6. Implement process checking
7. Add hyprctl commands

### 🎯 Dependencies Added
```toml
reqwest = { version = "0.12", features = ["json", "blocking"] }
```

### 🎯 Dependencies Still Needed
- `serde` (JSON parsing)
- `chrono` (time handling)
- Others TBD as we progress

## Current Code Status
- `main.rs`: Has basic structure + `fetch_sunrise_sunset()` function
- Function needs fix: line 10 should be `Ok(text)` not `Ok(text);`
- Ready to test HTTP functionality once fixed