# ModellConverter

A cross-platform tool for converting RC transmitter model files between different firmware formats.

## Supported Formats

- **EdgeTX** (`.yml`) — FrSky EdgeTX transmitter firmware models
- **Ethos** (`.bin`) — FrSky Ethos transmitter firmware models
- **Jeti Duplex** (`.jsn`) — Jeti Duplex RC transmitter models

## Installation

### Desktop GUI (Recommended for most users)

Pre-built installers for Windows, macOS, and Linux are available in the [Releases](https://github.com/tobiaseul/ModellConverter/releases) section.

- **Windows:** Download `.msi` installer
- **macOS:** Download `.dmg` image
- **Linux:** Download `.AppImage` or `.deb` package

### Command-Line Tool

Requires [Rust](https://www.rust-lang.org/):

```bash
cargo install --path .
modell-converter convert --from edgetx --to ethos input.yml output.bin
```

## Usage

### Desktop GUI

Launch the application and:

1. **Select source format** — Choose the firmware format of your model file
2. **Select target format** — Choose the format you want to convert to
3. **Drop or browse** — Drag and drop a file, or click to browse
4. **Convert** — Click "Convert & Save" and choose where to save

### Command-Line Interface

```bash
# Basic conversion
modell-converter convert --from edgetx --to jeti input.yml output.jsn

# Verbose output
modell-converter convert --from ethos --to edgetx -v input.bin output.yml

# Reverse engineering tools
modell-converter reveng hexdump --file model.bin --offset 0x100 --len 32
modell-converter reveng diff --file-a model1.bin --file-b model2.bin
```

## Building from Source

### Requirements

- Rust 1.70+ ([Install](https://www.rust-lang.org/tools/install))
- macOS/Linux/Windows

### Desktop GUI (Tauri)

```bash
cd src-tauri
cargo tauri dev       # Run in development mode
cargo tauri build     # Build for your platform
```

Outputs:
- Windows: `.exe` + `.msi` installer
- macOS: `.app` + `.dmg` image
- Linux: `.AppImage` + `.deb` package

### CLI Binary Only

```bash
cargo build --release
./target/release/modell-converter convert --from edgetx --to ethos model.yml model.bin
```

## Project Structure

```
ModellConverter/
├── src/                    # Rust library + CLI
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Conversion library
│   ├── ir/                # Intermediate representation
│   ├── formats/           # Format parsers & serializers
│   │   ├── edgetx/
│   │   ├── ethos/
│   │   └── jeti/          # (in development)
│   └── reveng/            # Reverse engineering tools
├── src-tauri/             # Desktop GUI
│   ├── src/               # Tauri backend (Rust)
│   └── tauri.conf.json
├── ui/                    # Web frontend
│   ├── index.html
│   ├── style.css
│   └── app.js
├── tests/
└── Cargo.toml
```

## Development

### Testing

```bash
cargo test
```

### Code Structure

- **Intermediate Representation (IR)** — All formats convert to a common IR, then to the target format
- **Format modules** — Each format has its own parser, schema, and serializer
- **Reverse Engineering** — Tools for analyzing and comparing binary files

### Adding a New Format

1. Create `src/formats/newformat/` module
2. Implement `Format` trait:
   - `from_bytes()` — Parse file to IR
   - `to_bytes()` — Serialize IR to file
3. Register in `src/convert.rs`
4. Add tests

## Known Limitations

- Jeti Duplex format support is currently in development
- Some advanced transmitter features may not convert perfectly between formats
- Always verify converted files on your transmitter before flying

## Contributing

Contributions welcome! Areas of interest:

- Jeti Duplex format completion (reverse engineering `.jsn` files)
- Additional format support
- Testing and bug reports
- UI improvements

## License

MIT License — See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Tauri](https://tauri.app/) for desktop GUI
- Uses [binrw](https://github.com/jam1garner/binrw) for binary parsing
