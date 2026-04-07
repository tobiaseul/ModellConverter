# ModellConverter

Cross-platform RC transmitter model file converter supporting EdgeTX, Ethos, and Jeti Duplex formats. Desktop GUI via Tauri + Rust backend.

## Tech Stack

- **Language:** Rust (edition 2021)
- **GUI Framework:** Tauri (web frontend + Rust backend)
- **Binary Parsing:** `binrw` (declarative binary format parsing)
- **Serialization:** serde (YAML, JSON)
- **CLI:** clap with derive macros
- **Testing:** tempfile, pretty_assertions

## Project Structure

```
src/
  ├── main.rs               # CLI entry point
  ├── lib.rs                # Core library
  ├── cli.rs                # CLI argument parsing
  ├── convert.rs            # Format conversion routing
  ├── format.rs             # Format trait definition
  ├── error.rs              # Error types
  ├── ir/
  │   ├── mod.rs            # Intermediate representation types
  │   └── model.rs          # Model struct (central data structure)
  ├── formats/
  │   ├── mod.rs
  │   ├── edgetx/           # EdgeTX format (YAML-based)
  │   │   ├── mod.rs
  │   │   ├── parser.rs
  │   │   ├── serializer.rs
  │   │   └── schema.rs
  │   ├── ethos/            # Ethos format (binary)
  │   │   ├── mod.rs
  │   │   ├── parser.rs
  │   │   ├── serializer.rs
  │   │   ├── schema.rs
  │   │   └── known_offsets.rs
  │   └── jeti/             # Jeti Duplex (in development)
  │       ├── mod.rs
  │       ├── parser.rs
  │       └── schema.rs
  └── reveng/               # Reverse engineering tools
      ├── mod.rs
      ├── diff.rs           # Binary file comparison
      └── hexdump.rs        # Hex dump utility

src-tauri/
  ├── src/
  │   ├── main.rs           # Tauri app entry
  │   └── lib.rs            # Tauri backend commands
  ├── tauri.conf.json       # App config
  └── Cargo.toml

ui/                         # Web frontend (HTML/CSS/JS)
  ├── index.html
  ├── style.css
  └── app.js
```

## Key Patterns

### Format Implementation
Each format module implements the `Format` trait:
```rust
pub trait Format {
    fn from_bytes(bytes: &[u8]) -> Result<ir::Model>;
    fn to_bytes(model: &ir::Model) -> Result<Vec<u8>>;
}
```

All formats convert through a common Intermediate Representation (`ir::Model`). This decouples format logic and enables format-to-format conversions.

### Error Handling
- Custom error types in `error.rs` using `thiserror`
- Propagate with `?` operator
- Result<T> used throughout

### Binary Parsing (Ethos)
- Uses `binrw` derive macros for declarative struct-to-bytes mapping
- Endianness, padding, offsets defined via attributes
- See `src/formats/ethos/schema.rs` for examples

### CLI Structure
- Arguments defined via `clap` derive macros in `cli.rs`
- Subcommands: `convert`, `reveng`
- Entry point: `main.rs` routes to conversion logic

## Build Commands

```bash
# CLI only
cargo build --release
./target/release/modell-converter convert --from ethos input.bin output.yml

# Desktop GUI (development)
cd src-tauri
cargo tauri dev       # Hot reload in development

# Desktop GUI (production build)
cd src-tauri
cargo tauri build

# Automated build script (both CLI + GUI)
./build-all.sh

# Tests
cargo test
cargo test -- --nocapture   # With output
```

## Testing

- Unit tests co-located with source files
- Integration tests should go in `tests/` directory
- Use `tempfile` for temporary file fixtures
- Use `pretty_assertions` for readable assert_eq! output
- Test command: `cargo test`

## Adding a New Format

1. Create `src/formats/newformat/` with:
   - `mod.rs` — Format struct + trait impl
   - `parser.rs` — Parsing logic
   - `serializer.rs` — Serialization logic
   - `schema.rs` — Data structures (if binary format)

2. Implement `Format` trait in `mod.rs`

3. Register in `src/convert.rs`:
   ```rust
   "newformat" => formats::newformat::Format::from_bytes(bytes),
   ```

4. Add tests in format module

## Tauri Backend

- Invoke Rust functions from frontend via Tauri commands
- Commands defined in `src-tauri/src/lib.rs` with `#[tauri::command]`
- Return `Result<T>` for error handling
- Frontend calls via `invoke('command_name', { args })`

## Important Notes

- **Disclaimer warnings:** Model files must be verified by human before transmitter use
- **Jeti format:** In development, incomplete support
- **Binary formats:** Endianness, padding, and offsets are critical for Ethos — validate with real hardware
- **Testing:** Always test converted files on actual transmitter before flying

## Git Workflow

- Feature branches from `main`
- Build tests pass before PR
- Tauri icon updates: See `src-tauri/icons/` (requires specific formats per platform)
