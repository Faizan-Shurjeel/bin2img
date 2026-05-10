// bin2img — Binary file to 2D byte matrix exporter
// Converts any compiled binary (EXE, DLL, ELF, etc.) into a fixed-width
// row matrix (256 bytes per row) for visualization in MATLAB.
//
// Usage:
//   bin2img <input>              → writes <input>.bin + prints dimensions
//   bin2img <input> -o out       → writes out.bin
//   bin2img <input> --csv        → writes <input>.csv (slower, larger)
//   bin2img <input> --batch dir/ → processes every file in a directory
//
// MATLAB loading (raw mode, recommended):
//   fid   = fopen('output.bin', 'rb');
//   img   = fread(fid, [256 Inf], 'uint8')';
//   fclose(fid);
//   img   = mat2gray(img);

use std::{
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    process,
    time::Instant,
};

// ── Constants ────────────────────────────────────────────────────────────────

/// Standard width used in malware visualization literature (Nataraj et al., 2011).
/// Each row = exactly 256 bytes of the binary's address space.
const ROW_WIDTH: usize = 256;

// ── CLI parsing (no external crates) ────────────────────────────────────────

struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    csv: bool,
    batch: bool,
}

fn parse_args() -> Args {
    let raw: Vec<String> = env::args().skip(1).collect();

    if raw.is_empty() || raw.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!(
            "bin2img — binary to image matrix exporter\n\
             \n\
             USAGE:\n\
             \x20 bin2img <input_file> [OPTIONS]\n\
             \n\
             OPTIONS:\n\
             \x20 -o <path>   Output file stem (default: same as input)\n\
             \x20 --csv       Write CSV instead of raw binary\n\
             \x20 --batch     Treat <input> as a directory; process all files inside\n\
             \x20 -h          Show this help\n\
             \n\
             OUTPUT (raw mode, default):\n\
             \x20 <stem>.bin  — flat u8 array, row-major, 256 cols, N rows\n\
             \x20              Load in MATLAB: fread(fid, [256 Inf], 'uint8')'\n\
             \n\
             OUTPUT (csv mode):\n\
             \x20 <stem>.csv  — same data as comma-separated integers\n\
             \x20              Load in MATLAB: csvread('stem.csv')"
        );
        process::exit(0);
    }

    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut csv = false;
    let mut batch = false;
    let mut i = 0;

    while i < raw.len() {
        match raw[i].as_str() {
            "-o" => {
                i += 1;
                output = Some(PathBuf::from(&raw[i]));
            }
            "--csv" => csv = true,
            "--batch" => batch = true,
            other => {
                if input.is_none() {
                    input = Some(PathBuf::from(other));
                } else {
                    eprintln!("Unexpected argument: {other}");
                    process::exit(1);
                }
            }
        }
        i += 1;
    }

    Args {
        input: input.unwrap_or_else(|| {
            eprintln!("Error: no input file specified.");
            process::exit(1);
        }),
        output,
        csv,
        batch,
    }
}

// ── Core conversion ──────────────────────────────────────────────────────────

/// Read `path`, chunk into fixed-width rows, pad the last row with zeros.
/// Returns (matrix_bytes, num_rows).
fn file_to_matrix(path: &Path) -> io::Result<(Vec<u8>, usize)> {
    let raw = fs::read(path)?;

    if raw.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "file is empty"));
    }

    // Calculate number of complete + partial rows
    let num_rows = raw.len().div_ceil(ROW_WIDTH);
    let padded_len = num_rows * ROW_WIDTH;

    // Allocate padded buffer (zero-filled by default in Rust)
    let mut matrix = vec![0u8; padded_len];
    matrix[..raw.len()].copy_from_slice(&raw);

    Ok((matrix, num_rows))
}

// ── Writers ──────────────────────────────────────────────────────────────────

/// Write raw u8 binary — fastest, smallest, recommended for MATLAB.
fn write_raw(matrix: &[u8], dest: &Path) -> io::Result<()> {
    let f = File::create(dest)?;
    let mut w = BufWriter::new(f);
    w.write_all(matrix)?;
    Ok(())
}

/// Write CSV — human-readable but ~4× larger and slower to load.
fn write_csv(matrix: &[u8], dest: &Path, num_rows: usize) -> io::Result<()> {
    let f = File::create(dest)?;
    let mut w = BufWriter::new(f);

    for row in 0..num_rows {
        let start = row * ROW_WIDTH;
        let end = start + ROW_WIDTH;
        let row_bytes = &matrix[start..end];

        // Write 255 values with commas, then the last without
        for (i, &b) in row_bytes.iter().enumerate() {
            if i < ROW_WIDTH - 1 {
                write!(w, "{b},")?;
            } else {
                writeln!(w, "{b}")?;
            }
        }
    }
    Ok(())
}

// ── Per-file orchestration ───────────────────────────────────────────────────

fn process_file(input: &Path, output_stem: &Path, use_csv: bool) -> io::Result<()> {
    let t = Instant::now();

    let (matrix, num_rows) = file_to_matrix(input)?;
    let file_size = fs::metadata(input)?.len();

    let (out_path, mode_label) = if use_csv {
        (output_stem.with_extension("csv"), "csv")
    } else {
        (output_stem.with_extension("bin"), "raw")
    };

    if use_csv {
        write_csv(&matrix, &out_path, num_rows)?;
    } else {
        write_raw(&matrix, &out_path)?;
    }

    let out_size = fs::metadata(&out_path)?.len();
    let elapsed = t.elapsed();

    println!(
        "  [{mode_label}] {name}",
        name = input.file_name().unwrap_or_default().to_string_lossy()
    );
    println!("  size   : {file_size} bytes → image {ROW_WIDTH}×{num_rows} px");
    println!("  output : {} ({out_size} bytes)", out_path.display());
    println!("  done in: {:.2?}", elapsed);
    println!();

    // Print MATLAB snippet so the user can just copy-paste
    if !use_csv {
        let stem = out_path.file_name().unwrap_or_default().to_string_lossy();
        println!("  % MATLAB load:");
        println!("  % fid = fopen('{stem}', 'rb');");
        println!("  % img = fread(fid, [256 Inf], 'uint8')';");
        println!("  % fclose(fid);");
        println!("  % img = mat2gray(img);");
        println!();
    }

    Ok(())
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    if args.batch {
        // Directory mode: process every file inside
        let dir = &args.input;
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Cannot read directory {}: {e}", dir.display());
                process::exit(1);
            }
        };

        println!("batch mode → {}\n", dir.display());

        let mut ok = 0u32;
        let mut fail = 0u32;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            // Output sits in the same directory, named <original_stem>.bin/.csv
            let stem = dir.join(path.file_stem().unwrap_or_default());
            match process_file(&path, &stem, args.csv) {
                Ok(_) => ok += 1,
                Err(e) => {
                    eprintln!("  SKIP {} — {e}", path.display());
                    fail += 1;
                }
            }
        }

        println!("summary: {ok} converted, {fail} skipped");
    } else {
        // Single file mode
        let stem = match &args.output {
            Some(o) => o.clone(),
            None => args.input.with_extension(""), // strip existing extension
        };

        println!();
        if let Err(e) = process_file(&args.input, &stem, args.csv) {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
