# mdsrc2txt

## Project Overview
`mdsrc2txt` is a Rust CLI tool designed to combine programming source code files from a directory or ZIP archive into a single text file. The output filename includes a timestamp and the base name of the input.

This tool is useful for creating a single document containing all source code for documentation, review, or analysis purposes (e.g., preparing context for LLMs).

## Key Features
- **Input Support:** Accepts both directories and `.zip` files.
- **Filtering:** Automatically filters for common source code extensions (e.g., `.rs`, `.py`, `.c`, `.cpp`, `.js`, etc.) and some resource files (`.rc`, `.ico`, `.cur`).
- **Output:** Generates a timestamped file: `YYYYMMDD-HHMMSS-<input_basename>-COMBINED.TXT`.
- **Formatting:** Separates files with a header and a separator line.

## Usage

### Running the Tool
Use `cargo run` to execute the tool directly:

```bash
cargo run -- <input_path>
```

**Example:**
```bash
cargo run -- ./my-project-src
# or
cargo run -- ./source-code.zip
```

### Help
To see the help message and list of options:
```bash
cargo run -- --help
```

## Development

### Build
To build the project in release mode:
```bash
cargo build --release
```
The binary will be located in `target/release/mdsrc2txt`.

### Testing
The project includes a comprehensive suite of unit and integration tests. The code is structured to allow testing of the core orchestration logic.

Run tests using:
```bash
cargo test
```

For code coverage (requires `cargo-llvm-cov`):
```bash
cargo llvm-cov
```

## Project Structure
- **`src/main.rs`**: Contains the application logic.
    - `main()`: Entry point, argument parsing, and error reporting.
    - `run(cli: Cli)`: Core orchestration logic.
    - `process_directory(...)`: Logic for recursively processing directories.
    - `process_zip(...)`: Logic for processing ZIP archives.
    - `tests` module: Contains unit and integration tests.
- **`Cargo.toml`**: Project configuration and dependencies (`chrono`, `clap`, `walkdir`, `zip`).

## Conventions
- **Code Style:** Follows standard Rust idioms ( `rustfmt`).
- **Error Handling:** Uses `Result` for logic and logs errors to stderr in `main`.
- **Tests:** Co-located in the `src/main.rs` file under the `tests` module. Logic is separated from `main` to facilitate testing.