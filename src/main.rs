//! novac: Nova WebAssembly Compiler CLI.

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    let mut input_file: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut emit_wat = false;
    let mut print_wat = false;
    let mut measure_time = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("Error: Missing argument for -o/--output");
                    process::exit(1);
                }
            }
            "--wat" => {
                emit_wat = true;
            }
            "--print-wat" => {
                print_wat = true;
            }
            "--time" => {
                measure_time = true;
            }
            arg if !arg.starts_with('-') => {
                if input_file.is_none() {
                    input_file = Some(arg.to_string());
                } else {
                    eprintln!("Error: Unexpected multiple input files: '{}'", arg);
                    process::exit(1);
                }
            }
            unknown => {
                eprintln!("Error: Unknown flag '{}'", unknown);
                print_help();
                process::exit(1);
            }
        }
        i += 1;
    }

    let input_path = match input_file {
        Some(path) => path,
        None => {
            eprintln!("Error: No input file provided.");
            print_help();
            process::exit(1);
        }
    };

    let source = match fs::read_to_string(&input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: Could not read file '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    let base_name = Path::new(&input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let wasm_out_path = output_file.unwrap_or_else(|| format!("{}.wasm", base_name));

    let start_time = Instant::now();

    // 1. Compile to Wasm Binary
    let wasm_bytes = match nova_wasm::compile_to_wasm(&source) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("\x1b[1;31m[novac compilation error]\x1b[0m {}", err);
            process::exit(1);
        }
    };

    let elapsed = start_time.elapsed();

    // Write binary output
    if let Err(e) = fs::write(&wasm_out_path, &wasm_bytes) {
        eprintln!("Error writing output Wasm file '{}': {}", wasm_out_path, e);
        process::exit(1);
    }

    println!(
        "\x1b[1;32m✓ Compiled\x1b[0m '{}' -> '{}' ({} bytes)",
        input_path,
        wasm_out_path,
        wasm_bytes.len()
    );

    if measure_time {
        println!(
            "  ⏱ Total compilation time: \x1b[1;36m{:.2} µs\x1b[0m",
            elapsed.as_secs_f64() * 1_000_000.0
        );
    }

    // Optional WAT emission
    if emit_wat || print_wat {
        match nova_wasm::compile_to_wat(&source) {
            Ok(wat) => {
                if print_wat {
                    println!("\n--- WebAssembly Text Format (.wat) ---\n{}", wat);
                }
                if emit_wat {
                    let wat_path = format!("{}.wat", base_name);
                    let _ = fs::write(&wat_path, wat);
                    println!("\x1b[1;32m✓ Emitted WAT\x1b[0m '{}'", wat_path);
                }
            }
            Err(err) => {
                eprintln!("\x1b[1;31m[WAT generation error]\x1b[0m {}", err);
            }
        }
    }
}

fn print_help() {
    println!("=================================================================");
    println!("  novac // Nova WebAssembly Compiler (Direct Wasm Binary Emitter)");
    println!("=================================================================");
    println!("USAGE:");
    println!("  novac <input.nova> [FLAGS] [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -o, --output <path>    Specify output .wasm binary file path");
    println!("  --wat                  Emit human-readable .wat alongside .wasm");
    println!("  --print-wat            Print S-expression WAT to standard output");
    println!("  --time                 Measure and print compilation latency");
    println!("  -h, --help             Show this help information");
    println!();
    println!("EXAMPLES:");
    println!("  novac math.nova -o math.wasm");
    println!("  novac game.nova --wat --time");
}
