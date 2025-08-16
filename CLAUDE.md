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

## Communication Guidelines
- Tone down on compliments
- Avoid phrases like "good question", "excellent thought", or overly flattering language
- Focus on direct, concise communication

## Current Progress

### ✅ Completed
1. **Project Analysis** - Understood bash script functionality
2. **Project Setup** - Created Cargo.toml with reqwest dependency
3. **HTTP Client Decision** - Chose synchronous approach using `reqwest::blocking`
4. **Basic Structure** - Created main.rs with TODO roadmap
5. **HTTP Function** - Started `fetch_sunrise_sunset()` function

### 🔄 Current Step
**Adding time parsing functionality**
- Fixed HTTP function syntax error (`Ok(text)` instead of `Ok(text);`)
- Successfully tested HTTP function - it works with rustls-tls
- Added `SunTimes` struct with `NaiveTime` fields
- Added `parse_military_time()` helper function
- Next: refactor `fetch_sunrise_sunset()` to parse JSON and return `SunTimes`

### 📚 Concepts Learned
- **Result<T, E>**: Sum type/tagged union for error handling
- **Box<T>**: Heap allocation for unknown-sized types
- **dyn trait**: Trait objects for dynamic dispatch
- **? operator**: Error propagation
- **format! macro**: String interpolation
- **References (&T)**: Borrowing data without taking ownership
- **String vs &str**: Owned vs borrowed string data
- **String slices**: Views into string data
- **derive**: Auto-generating trait implementations
- **.into()**: Type conversion using Into trait
- **ok_or_else()**: Converting Option to Result

### 📋 Next Steps
1. Refactor `fetch_sunrise_sunset()` to parse JSON and return `SunTimes` struct
2. Test the refactored function
3. Write unit tests for time parsing
4. Implement time comparison logic
5. Add file I/O for state management
6. Implement process checking
7. Add hyprctl commands

### 🎯 Dependencies Added
```toml
reqwest = { version = "0.12", features = ["json", "blocking", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 🎯 Dependencies Still Needed
- Others TBD as we progress

## Current Code Status
- `main.rs`: Has working HTTP function + SunTimes struct + time parsing helper
- Added `SunTimes` struct with `NaiveTime` fields for sunrise/sunset
- Added `parse_military_time()` function to convert "0652" → NaiveTime
- HTTP function currently returns raw JSON string
- Ready to refactor HTTP function to parse JSON and return SunTimes struct