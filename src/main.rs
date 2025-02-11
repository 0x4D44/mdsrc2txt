/// mdsrc2txt - Combines programming source code files from a directory or ZIP file
/// into a single text file whose name is generated from the current date/time and the
/// input name.
use chrono::Local;
use clap::{Parser, CommandFactory};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(
    name = "mdcode2txt",
    author = "Your Name",
    version = "1.0",
    about = "Combines programming source code files from a directory or ZIP file into a single text file",
    long_about = None
)]
struct Cli {
    /// Input directory or ZIP file to process
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Also print help if "-?" is given.
    if std::env::args().any(|arg| arg == "-?") {
        Cli::command().print_help()?;
        println!(); // Add newline after help.
        return Ok(());
    }

    let cli = Cli::parse();

    let input_path = Path::new(&cli.input);
    if !input_path.exists() {
        eprintln!("Error: Input path '{}' does not exist.", input_path.display());
        std::process::exit(1);
    }

    // Build the output file name in the format:
    // YYYYMMDD-HHMMSS-<input_basename>-COMBINED.TXT
    let base_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("input");
    let now = Local::now();
    let datetime_str = now.format("%Y%m%d-%H%M%S").to_string();
    let output_file_name = format!("{}-{}-COMBINED.TXT", datetime_str, base_name);
    let mut output_file = File::create(&output_file_name)?;
    // Only the literal "Creating output file:" is in light blue.
    println!("\x1b[94mCreating output file:\x1b[0m {}", output_file_name);

    // Counters for progress
    let mut total_files = 0;
    let mut total_size: u64 = 0;

    // List of allowed file extensions (all in lowercase).
    // Added support for .def, .dlg, .rc, .cur, and .ico.
    let allowed_exts = vec![
        "rs", "py", "java", "c", "cpp", "h", "js", "ts", "go", "rb", "swift", "kt", "php", "cs",
        "def", "dlg", "rc", "cur", "ico",
    ];

    if input_path.is_dir() {
        // Process directory recursively.
        for entry in WalkDir::new(input_path) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if allowed_exts.contains(&ext_lower.as_str()) {
                        // Read file as bytes and convert using lossy UTF-8 conversion.
                        let bytes = std::fs::read(path)?;
                        let content = String::from_utf8_lossy(&bytes).into_owned();
                        write_file_content(&mut output_file, &path.to_string_lossy(), &content)?;
                        total_files += 1;
                        total_size += content.len() as u64;
                        // Update in-place: only the literal "Adding file:" is in light blue.
                        print!(
                            "\r\x1B[2K\x1b[94mAdding file:\x1b[0m {} | Total files: {} | Total size: {}",
                            format_filename(&path.to_string_lossy(), 30),
                            total_files,
                            format_size(total_size)
                        );
                        std::io::stdout().flush()?;
                    }
                }
            }
        }
    } else if input_path.is_file() {
        // Process ZIP file. (Only ZIP files are accepted as file input.)
        if input_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            == Some("zip".to_string())
        {
            let file = File::open(input_path)?;
            let mut zip = ZipArchive::new(file)?;
            for i in 0..zip.len() {
                let mut zip_file = zip.by_index(i)?;
                if zip_file.is_file() {
                    // Get an owned copy of the file name.
                    let file_name = zip_file.name().to_owned();
                    let ext = Path::new(&file_name)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if allowed_exts.contains(&ext.as_str()) {
                        // Read ZIP file content as bytes then convert.
                        let mut buffer = Vec::new();
                        zip_file.read_to_end(&mut buffer)?;
                        let content = String::from_utf8_lossy(&buffer).into_owned();
                        write_file_content(&mut output_file, &file_name, &content)?;
                        total_files += 1;
                        total_size += content.len() as u64;
                        // In-place progress update.
                        print!(
                            "\r\x1B[2K\x1b[94mAdding file:\x1b[0m {} | Total files: {} | Total size: {}",
                            format_filename(&file_name, 30),
                            total_files,
                            format_size(total_size)
                        );
                        std::io::stdout().flush()?;
                    }
                }
            }
        } else {
            eprintln!(
                "Error: Input file '{}' is not a ZIP file or a directory.",
                input_path.display()
            );
            std::process::exit(1);
        }
    } else {
        eprintln!(
            "Error: Input path '{}' is neither a directory nor a file.",
            input_path.display()
        );
        std::process::exit(1);
    }

    // Move to a new line after progress updates.
    println!();
    // Print final message with colored output:
    // Light blue for "Processing completed." and light yellow for "Combined file created:".
    println!(
        "\x1b[94mProcessing completed.\x1b[0m \x1b[93mCombined file created: {}\x1b[0m",
        output_file_name
    );
    Ok(())
}

/// Writes a header (the file name), the file’s content, and a separator to the output.
fn write_file_content(
    output: &mut File,
    filename: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(output, "File: {}\n", filename)?;
    writeln!(output, "{}\n", content)?;
    writeln!(output, "----------------------------------------\n")?;
    Ok(())
}

/// Formats a byte count into a human-readable string with units.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Formats the filename to a fixed field width (30 characters).  
/// If the filename is longer, it truncates the start (showing the last characters) 
/// and prefixes the result with "..." to indicate truncation.
fn format_filename(filename: &str, field_width: usize) -> String {
    if filename.len() > field_width {
        let truncation_indicator = "...";
        let trimmed_len = field_width - truncation_indicator.len();
        let truncated = &filename[filename.len() - trimmed_len..];
        format!("{}{}", truncation_indicator, truncated)
    } else {
        format!("{:>width$}", filename, width = field_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    /// A helper function similar to the one in main to check if a file
    /// is a “source code” file (based on its extension) in a case-insensitive way.
    fn is_source_file(path: &Path, allowed_exts: &[&str]) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| allowed_exts.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    #[test]
    fn test_is_source_file() {
        let allowed = vec!["rs", "py", "c", "h", "def", "dlg", "rc", "cur", "ico"];
        assert!(is_source_file(Path::new("main.rs"), &allowed));
        assert!(is_source_file(Path::new("script.py"), &allowed));
        assert!(is_source_file(Path::new("code.C"), &allowed)); // Case-insensitive
        assert!(is_source_file(Path::new("icon.ICO"), &allowed)); // Case-insensitive
        assert!(!is_source_file(Path::new("readme.txt"), &allowed));
        assert!(!is_source_file(Path::new("binary.bin"), &allowed));
    }

    #[test]
    fn test_process_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory with source files and a non-code file.
        let dir = tempdir()?;
        let src_file = dir.path().join("test.c");
        fs::write(&src_file, "int main() { return 0; }")?;
        let header_file = dir.path().join("test.h");
        fs::write(&header_file, "#define TEST 1")?;
        let non_code_file = dir.path().join("notes.txt");
        fs::write(&non_code_file, "This is not code.")?;

        // Prepare an output file.
        let output_path = dir.path().join("output.txt");
        let mut output_file = File::create(&output_path)?;

        // Allowed extensions including additional ones.
        let allowed_exts = vec![
            "rs", "py", "java", "c", "cpp", "h", "js", "ts", "go", "rb", "swift", "kt", "php", "cs",
            "def", "dlg", "rc", "cur", "ico",
        ];
        let mut total_files = 0;
        let mut total_size = 0;
        for entry in WalkDir::new(dir.path()) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if allowed_exts.contains(&ext_lower.as_str()) {
                        let bytes = fs::read(path)?;
                        let content = String::from_utf8_lossy(&bytes).into_owned();
                        write_file_content(&mut output_file, &path.to_string_lossy(), &content)?;
                        total_files += 1;
                        total_size += content.len();
                    }
                }
            }
        }
        // Only the .c and .h files should be processed.
        assert_eq!(total_files, 2);
        assert!(total_size > 0);

        let output_contents = fs::read_to_string(output_path)?;
        assert!(output_contents.contains("test.c"));
        assert!(output_contents.contains("test.h"));
        Ok(())
    }

    #[test]
    fn test_process_zip() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary ZIP file with source and non-code files.
        let dir = tempdir()?;
        let zip_path = dir.path().join("test.zip");
        {
            let file = File::create(&zip_path)?;
            let mut zip = zip::ZipWriter::new(file);
            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("code.c", options)?;
            zip.write_all(b"int main() { return 0; }")?;
            zip.start_file("header.h", options)?;
            zip.write_all(b"#define TEST 1")?;
            zip.start_file("notes.txt", options)?;
            zip.write_all(b"This is not code.")?;
            zip.finish()?;
        }

        let file = File::open(&zip_path)?;
        let mut zip = ZipArchive::new(file)?;
        let allowed_exts = vec![
            "rs", "py", "java", "c", "cpp", "h", "js", "ts", "go", "rb", "swift", "kt", "php", "cs",
            "def", "dlg", "rc", "cur", "ico",
        ];
        let mut total_files = 0;
        let mut total_size = 0;
        let mut combined = Vec::new();
        for i in 0..zip.len() {
            let mut zip_file = zip.by_index(i)?;
            if zip_file.is_file() {
                let file_name = zip_file.name().to_owned();
                let ext = Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if allowed_exts.contains(&ext.as_str()) {
                    let mut buffer = Vec::new();
                    zip_file.read_to_end(&mut buffer)?;
                    let content = String::from_utf8_lossy(&buffer).into_owned();
                    write!(&mut combined, "File: {}\n", file_name)?;
                    write!(&mut combined, "{}\n", content)?;
                    write!(&mut combined, "----------------------------------------\n")?;
                    total_files += 1;
                    total_size += content.len();
                }
            }
        }
        // Only the code.c and header.h files should be processed.
        assert_eq!(total_files, 2);
        assert!(total_size > 0);
        let combined_str = String::from_utf8(combined)?;
        assert!(combined_str.contains("code.c"));
        assert!(combined_str.contains("header.h"));
        Ok(())
    }
}
