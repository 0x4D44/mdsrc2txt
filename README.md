# mdsrc2txt

`mdsrc2txt` is a robust Rust CLI tool designed to aggregate programming source code files from a directory or a ZIP archive into a single, formatted text file. This is particularly useful for preparing code contexts for Large Language Models (LLMs), creating documentation snapshots, or performing code reviews.

## Features

- **Recursive Processing:** Automatically traverses nested directories to find all relevant source files.
- **ZIP Support:** Directly processes `.zip` archives without needing manual extraction.
- **Smart Filtering:** Automatically filters files based on a comprehensive list of source code extensions (see [Supported Extensions](#supported-extensions)).
- **Formatted Output:** Generates a clean, readable output file with headers and separators for each file.
- **Timestamped naming:** Output files are named with a timestamp (`YYYYMMDD-HHMMSS`) and the input base name to prevent overwrites and keep history organized.
- **Progress Tracking:** Displays a real-time progress indicator with file counts and total size processed.

## Installation

### From Source

Ensure you have [Rust and Cargo installed](https://rustup.rs/).

1.  Clone the repository:
    ```bash
    git clone https://github.com/yourusername/mdsrc2txt.git
    cd mdsrc2txt
    ```

2.  Build the project:
    ```bash
    cargo build --release
    ```

The binary will be available in `target/release/mdsrc2txt`.

## Usage

Run the tool by providing the path to a directory or a ZIP file.

```bash
# Run directly with cargo
cargo run -- <input_path>

# Or using the built binary
./target/release/mdsrc2txt <input_path>
```

### Examples

**Process a local directory:**
```bash
mdsrc2txt ./my-project-src
```
*Generates: `20260102-123045-my-project-src-COMBINED.TXT`*

**Process a ZIP file:**
```bash
mdsrc2txt ./legacy-code.zip
```
*Generates: `20260102-123045-legacy-code-COMBINED.TXT`*

## Supported Extensions

The tool currently supports and filters for the following file extensions (case-insensitive):

- **Systems/Core:** `.rs`, `.c`, `.cpp`, `.h`, `.go`
- **Web/Scripting:** `.js`, `.ts`, `.py`, `.php`, `.rb`
- **Application/Mobile:** `.java`, `.kt`, `.swift`, `.cs`
- **Resources:** `.rc`, `.def`, `.dlg`, `.cur`, `.ico`

## Development

### Prerequisites
- Rust 1.70+ (recommended)

### Running Tests
The project maintains a high level of code coverage. Run the test suite to verify functionality:

```bash
cargo test
```

To check code coverage (requires `cargo-llvm-cov`):
```bash
cargo llvm-cov
```

### Code Quality
Ensure your changes meet the project standards before committing:

```bash
# Check for compilation errors
cargo check

# Format code
cargo fmt

# Linting
cargo clippy -- -D warnings
```

## License

[MIT](LICENSE) (or your preferred license)
