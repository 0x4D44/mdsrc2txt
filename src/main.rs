/// mdsrc2txt - Combines programming source code files from a directory or ZIP file
/// into a single text file whose name is generated from the current date/time and the
/// input name.
use chrono::Local;
use clap::{CommandFactory, Parser};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::ZipArchive;

/// Name used for files in the root of the input directory (no subdirectory).
const ROOT_BUCKET: &str = "ROOT";

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

    /// Split output by top-level subdirectory (one file per subdirectory, plus one for root)
    #[arg(short, long)]
    split: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Also print help if "-?" is given.
    if std::env::args().any(|arg| arg == "-?") {
        Cli::command().print_help()?;
        println!(); // Add newline after help.
        return Ok(());
    }

    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn run(cli: Cli) -> Result<String, Box<dyn std::error::Error>> {
    let input_path = Path::new(&cli.input);
    if !input_path.exists() {
        return Err(format!(
            "Error: Input path '{}' does not exist.",
            input_path.display()
        )
        .into());
    }

    let base_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("input");
    let now = Local::now();
    let datetime_str = now.format("%Y%m%d-%H%M%S").to_string();

    if cli.split {
        run_split(input_path, base_name, &datetime_str)
    } else {
        run_combined(input_path, base_name, &datetime_str)
    }
}

/// Original behavior: combine all source files into a single output file.
fn run_combined(
    input_path: &Path,
    base_name: &str,
    datetime_str: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let output_file_name = format!("{}-{}-COMBINED.TXT", datetime_str, base_name);
    let mut output_file = File::create(&output_file_name)?;
    println!("\x1b[94mCreating output file:\x1b[0m {}", output_file_name);

    let mut total_files = 0;
    let mut total_size: u64 = 0;

    if input_path.is_dir() {
        process_directory(
            input_path,
            &mut output_file,
            &mut total_files,
            &mut total_size,
        )?;
    } else if input_path.is_file() {
        process_zip(
            input_path,
            &mut output_file,
            &mut total_files,
            &mut total_size,
        )?;
    } else {
        return Err(format!(
            "Error: Input path '{}' is neither a directory nor a file.",
            input_path.display()
        )
        .into());
    }

    println!();
    println!(
        "\x1b[94mProcessing completed.\x1b[0m \x1b[93mCombined file created: {}\x1b[0m",
        output_file_name
    );
    Ok(output_file_name)
}

/// Split mode: create one output file per top-level subdirectory.
fn run_split(
    input_path: &Path,
    base_name: &str,
    datetime_str: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Collect files into buckets by top-level subdirectory
    let buckets: HashMap<String, Vec<(String, String)>> = if input_path.is_dir() {
        collect_directory_buckets(input_path)?
    } else if input_path.is_file() {
        collect_zip_buckets(input_path)?
    } else {
        return Err(format!(
            "Error: Input path '{}' is neither a directory nor a file.",
            input_path.display()
        )
        .into());
    };

    if buckets.is_empty() {
        println!("\x1b[93mNo source files found.\x1b[0m");
        return Ok(String::new());
    }

    // Sort bucket names for consistent output order
    let mut bucket_names: Vec<&String> = buckets.keys().collect();
    bucket_names.sort();

    let mut output_files = Vec::new();
    let mut grand_total_files = 0;
    let mut grand_total_size: u64 = 0;

    for bucket_name in bucket_names {
        let files = &buckets[bucket_name];
        if files.is_empty() {
            continue;
        }

        let output_file_name = format!("{}-{}-{}-COMBINED.TXT", datetime_str, base_name, bucket_name);
        let mut output_file = File::create(&output_file_name)?;
        println!("\x1b[94mCreating output file:\x1b[0m {}", output_file_name);

        let mut bucket_files = 0;
        let mut bucket_size: u64 = 0;

        for (filename, content) in files {
            write_file_content(&mut output_file, filename, content)?;
            bucket_files += 1;
            bucket_size += content.len() as u64;
            print!(
                "\r\x1B[2K\x1b[94mAdding file:\x1b[0m {} | Files: {} | Size: {}",
                format_filename(filename, 30),
                bucket_files,
                format_size(bucket_size)
            );
            std::io::stdout().flush()?;
        }

        println!();
        grand_total_files += bucket_files;
        grand_total_size += bucket_size;
        output_files.push(output_file_name);
    }

    println!(
        "\x1b[94mProcessing completed.\x1b[0m Created {} files, {} total files, {} total size.",
        output_files.len(),
        grand_total_files,
        format_size(grand_total_size)
    );

    // Return the first output file name (for test compatibility)
    Ok(output_files.into_iter().next().unwrap_or_default())
}

/// Recursively processes a directory and writes allowed source files to the output.
fn process_directory(
    path: &Path,
    output: &mut File,
    total_files: &mut usize,
    total_size: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(path) {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() && is_source_file(entry_path) {
            let bytes = std::fs::read(entry_path)?;
            let content = String::from_utf8_lossy(&bytes).into_owned();
            write_file_content(output, &entry_path.to_string_lossy(), &content)?;
            *total_files += 1;
            *total_size += content.len() as u64;
            // Update in-place: only the literal "Adding file:" is in light blue.
            print!(
                "\r\x1B[2K\x1b[94mAdding file:\x1b[0m {} | Total files: {} | Total size: {}",
                format_filename(&entry_path.to_string_lossy(), 30),
                *total_files,
                format_size(*total_size)
            );
            std::io::stdout().flush()?;
        }
    }
    Ok(())
}

/// Processes a ZIP file and writes allowed source files to the output.
fn process_zip(
    path: &Path,
    output: &mut File,
    total_files: &mut usize,
    total_size: &mut u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        != Some("zip".to_string())
    {
        return Err(format!(
            "Error: Input file '{}' is not a ZIP file or a directory.",
            path.display()
        )
        .into());
    }

    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;
        if zip_file.is_file() {
            let file_name = zip_file.name().to_owned();
            if is_source_file(Path::new(&file_name)) {
                let mut buffer = Vec::new();
                zip_file.read_to_end(&mut buffer)?;
                let content = String::from_utf8_lossy(&buffer).into_owned();
                write_file_content(output, &file_name, &content)?;
                *total_files += 1;
                *total_size += content.len() as u64;
                print!(
                    "\r\x1B[2K\x1b[94mAdding file:\x1b[0m {} | Total files: {} | Total size: {}",
                    format_filename(&file_name, 30),
                    *total_files,
                    format_size(*total_size)
                );
                std::io::stdout().flush()?;
            }
        }
    }
    Ok(())
}

/// Collects source files from a directory into buckets by top-level subdirectory.
/// Returns a map from bucket name to list of (filename, content) pairs.
fn collect_directory_buckets(
    path: &Path,
) -> Result<HashMap<String, Vec<(String, String)>>, Box<dyn std::error::Error>> {
    let mut buckets: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() && is_source_file(entry_path) {
            let relative = entry_path.strip_prefix(path).unwrap_or(entry_path);
            let bucket_name = get_bucket_name(relative);
            let bytes = std::fs::read(entry_path)?;
            let content = String::from_utf8_lossy(&bytes).into_owned();
            buckets
                .entry(bucket_name)
                .or_default()
                .push((entry_path.to_string_lossy().into_owned(), content));
        }
    }

    Ok(buckets)
}

/// Collects source files from a ZIP file into buckets by top-level subdirectory.
/// Returns a map from bucket name to list of (filename, content) pairs.
fn collect_zip_buckets(
    path: &Path,
) -> Result<HashMap<String, Vec<(String, String)>>, Box<dyn std::error::Error>> {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        != Some("zip".to_string())
    {
        return Err(format!(
            "Error: Input file '{}' is not a ZIP file or a directory.",
            path.display()
        )
        .into());
    }

    let mut buckets: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let file = File::open(path)?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i)?;
        if zip_file.is_file() {
            let file_name = zip_file.name().to_owned();
            if is_source_file(Path::new(&file_name)) {
                let bucket_name = get_bucket_name(Path::new(&file_name));
                let mut buffer = Vec::new();
                zip_file.read_to_end(&mut buffer)?;
                let content = String::from_utf8_lossy(&buffer).into_owned();
                buckets
                    .entry(bucket_name)
                    .or_default()
                    .push((file_name, content));
            }
        }
    }

    Ok(buckets)
}

/// Extracts the bucket name from a relative path.
/// The bucket is the first path component, or ROOT_BUCKET if the file is at the root.
fn get_bucket_name(relative_path: &Path) -> String {
    let components: Vec<_> = relative_path.components().collect();
    if components.len() <= 1 {
        // File is directly in the root
        ROOT_BUCKET.to_string()
    } else {
        // First component is the top-level subdirectory
        components[0]
            .as_os_str()
            .to_string_lossy()
            .into_owned()
    }
}

/// Writes a header (the file name), the file's content, and a separator to the output.
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

/// Checks if a file is a source code file based on its extension.
fn is_source_file(path: &Path) -> bool {
    // List of allowed file extensions (all in lowercase).
    const ALLOWED_EXTS: &[&str] = &[
        "rs", "py", "java", "c", "cpp", "h", "js", "ts", "go", "rb", "swift", "kt", "php", "cs",
        "def", "dlg", "rc", "cur", "ico",
    ];
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ALLOWED_EXTS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use zip::write::FileOptions;

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("main.rs")));
        assert!(is_source_file(Path::new("script.py")));
        assert!(is_source_file(Path::new("code.C"))); // Case-insensitive
        assert!(is_source_file(Path::new("icon.ICO"))); // Case-insensitive
        assert!(!is_source_file(Path::new("readme.txt")));
        assert!(!is_source_file(Path::new("binary.bin")));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_filename() {
        assert_eq!(format_filename("short.txt", 10), " short.txt");
        assert_eq!(format_filename("exactlength", 11), "exactlength");
        assert_eq!(format_filename("verylongfilename.txt", 10), "...ame.txt");
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

        let mut total_files = 0;
        let mut total_size: u64 = 0;
        process_directory(
            dir.path(),
            &mut output_file,
            &mut total_files,
            &mut total_size,
        )?;

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

        // Prepare an output file.
        let output_path = dir.path().join("output.txt");
        let mut output_file = File::create(&output_path)?;

        let mut total_files = 0;
        let mut total_size: u64 = 0;
        process_zip(
            &zip_path,
            &mut output_file,
            &mut total_files,
            &mut total_size,
        )?;

        // Only the code.c and header.h files should be processed.
        assert_eq!(total_files, 2);
        assert!(total_size > 0);

        let combined_str = fs::read_to_string(output_path)?;
        assert!(combined_str.contains("code.c"));
        assert!(combined_str.contains("header.h"));
        Ok(())
    }

    #[test]
    fn test_run_workflow() -> Result<(), Box<dyn std::error::Error>> {
        // Setup a temp dir with some source files
        let dir = tempdir()?;
        let src_file = dir.path().join("main.rs");
        fs::write(&src_file, "fn main() {}")?;

        let cli = Cli {
            input: dir.path().to_string_lossy().into_owned(),
            split: false,
        };

        let output_filename = run(cli)?;
        assert!(Path::new(&output_filename).exists());

        // Cleanup
        fs::remove_file(&output_filename)?;
        Ok(())
    }

    #[test]
    fn test_run_invalid_path() {
        let cli = Cli {
            input: "non_existent_path_xyz".to_string(),
            split: false,
        };
        assert!(run(cli).is_err());
    }

    #[test]
    fn test_run_invalid_zip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let txt_file = dir.path().join("test.txt");
        fs::write(&txt_file, "content")?;

        let cli = Cli {
            input: txt_file.to_string_lossy().into_owned(),
            split: false,
        };
        assert!(run(cli).is_err());
        Ok(())
    }

    #[test]
    fn test_get_bucket_name() {
        // File at root level
        assert_eq!(get_bucket_name(Path::new("main.rs")), ROOT_BUCKET);

        // File in subdirectory
        assert_eq!(get_bucket_name(Path::new("src/lib.rs")), "src");
        assert_eq!(get_bucket_name(Path::new("tests/test1.rs")), "tests");

        // File in nested subdirectory (should still use top-level)
        assert_eq!(get_bucket_name(Path::new("src/utils/helpers.rs")), "src");
    }

    #[test]
    fn test_split_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temp directory with files in root and subdirectories
        let dir = tempdir()?;

        // Root file
        fs::write(dir.path().join("main.rs"), "fn main() {}")?;

        // src subdirectory
        fs::create_dir(dir.path().join("src"))?;
        fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}")?;
        fs::write(dir.path().join("src/util.rs"), "pub fn util() {}")?;

        // tests subdirectory
        fs::create_dir(dir.path().join("tests"))?;
        fs::write(dir.path().join("tests/test1.rs"), "#[test] fn test1() {}")?;

        let cli = Cli {
            input: dir.path().to_string_lossy().into_owned(),
            split: true,
        };

        let first_output = run(cli)?;
        assert!(!first_output.is_empty());

        // Should have created 3 output files (ROOT, src, tests)
        // Find all generated files by pattern
        let output_files: Vec<_> = std::fs::read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.to_string_lossy().contains("-COMBINED.TXT")
                    && p.to_string_lossy().contains(&dir.path().file_name().unwrap().to_string_lossy().to_string())
            })
            .collect();

        assert_eq!(output_files.len(), 3, "Expected 3 output files, got {:?}", output_files);

        // Cleanup
        for f in output_files {
            fs::remove_file(f)?;
        }
        Ok(())
    }

    #[test]
    fn test_split_zip() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let zip_path = dir.path().join("project.zip");
        {
            let file = File::create(&zip_path)?;
            let mut zip = zip::ZipWriter::new(file);
            let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

            // Root file
            zip.start_file("main.c", options)?;
            zip.write_all(b"int main() { return 0; }")?;

            // src subdirectory
            zip.add_directory("src/", options)?;
            zip.start_file("src/lib.c", options)?;
            zip.write_all(b"void lib() {}")?;

            zip.finish()?;
        }

        let cli = Cli {
            input: zip_path.to_string_lossy().into_owned(),
            split: true,
        };

        let first_output = run(cli)?;
        assert!(!first_output.is_empty());

        // Should have created 2 output files (ROOT, src)
        let output_files: Vec<_> = std::fs::read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.to_string_lossy().contains("-COMBINED.TXT")
                    && p.to_string_lossy().contains("project")
            })
            .collect();

        assert_eq!(output_files.len(), 2, "Expected 2 output files, got {:?}", output_files);

        // Cleanup
        for f in output_files {
            fs::remove_file(f)?;
        }
        Ok(())
    }

    #[test]
    fn test_split_empty_subdirs_skipped() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temp directory where one subdir has no source files
        let dir = tempdir()?;

        // src subdirectory with source files
        fs::create_dir(dir.path().join("src"))?;
        fs::write(dir.path().join("src/lib.rs"), "pub fn lib() {}")?;

        // docs subdirectory with only txt files (should be skipped)
        fs::create_dir(dir.path().join("docs"))?;
        fs::write(dir.path().join("docs/readme.txt"), "Documentation")?;

        let cli = Cli {
            input: dir.path().to_string_lossy().into_owned(),
            split: true,
        };

        let first_output = run(cli)?;
        assert!(!first_output.is_empty());

        // Should have created only 1 output file (src), docs should be skipped
        let output_files: Vec<_> = std::fs::read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.to_string_lossy().contains("-COMBINED.TXT")
                    && p.to_string_lossy().contains(&dir.path().file_name().unwrap().to_string_lossy().to_string())
            })
            .collect();

        assert_eq!(output_files.len(), 1, "Expected 1 output file, got {:?}", output_files);
        assert!(output_files[0].to_string_lossy().contains("-src-"), "Expected src output file");

        // Cleanup
        for f in output_files {
            fs::remove_file(f)?;
        }
        Ok(())
    }
}
