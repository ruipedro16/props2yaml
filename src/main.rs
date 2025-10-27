mod converter;
mod errors;
mod properties;
mod yaml;

use crate::converter::Format;
use clap::Parser;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(name = "props2yaml")]
#[command(version = "0.1.0")]
#[command(about = "Convert between Java properties and YAML formats", long_about = None)]
struct Cli {
    /// Input file (use '-' for stdin)
    #[arg(value_name = "FILE")]
    input: String,

    /// Output file (defaults to stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Target format (properties, yaml)
    #[arg(short, long, value_name = "FORMAT")]
    format: Option<String>,

    /// Source format (auto-detected from file extension if not provided)
    #[arg(short = 'F', long, value_name = "FORMAT")]
    from: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip YAML formatting with yamlfmt
    #[arg(long)]
    skip_format: bool,

    /// Path to yamlfmt binary (optional, searches PATH if not provided)
    #[arg(long, value_name = "PATH")]
    yamlfmt_path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("{:#?}", cli)
    }

    let input_content = if cli.input == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        fs::read_to_string(&cli.input)?
    };

    // Determine source format if not specified
    let from_format = match (&cli.format, cli.input.as_str()) {
        (Some(from_str), _) => Format::from_str(from_str)?,

        (None, path) if path != "-" => Format::from_path(path)?,

        _ => {
            eprintln!("Error: Must specify --from when reading from stdin");
            std::process::exit(1);
        }
    };

    if cli.verbose {
        println!("Source format: {:?}", from_format);
    }

    let to_format = match (&cli.format, &cli.output) {
        (Some(to_str), _) => Format::from_str(to_str).unwrap_or_else(|e| {
            eprintln!("Failed to parse --format: {}", e);
            std::process::exit(1);
        }),

        (None, Some(output_path)) => {
            let path_str = output_path.to_str().unwrap_or_else(|| {
                eprintln!("Failed to convert output path to string");
                std::process::exit(1);
            });

            Format::from_path(path_str).unwrap_or_else(|e| {
                eprintln!("Failed to determine format from output path: {}", e);
                std::process::exit(1);
            })
        }

        (None, None) => {
            // Default: convert to the opposite format
            match from_format {
                Format::Properties => Format::Yaml,
                Format::Yaml => Format::Properties,
            }
        }
    };

    if cli.verbose {
        println!("Target format: {:?}", to_format);
        println!("Converting...");
    }

    let output_content = converter::convert(
        &input_content,
        from_format,
        to_format,
        cli.skip_format,
        cli.verbose,
        cli.yamlfmt_path.as_ref(),
    )?;

    if cli.verbose {
        println!(
            "Writing output to: {}",
            cli.output
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "stdout".to_string())
        );
    }

    if let Some(output_path) = cli.output {
        fs::write(output_path, output_content)?;
        if cli.verbose {
            println!("Conversion completed successfully!");
        }
    } else {
        print!("{}", output_content);
    }

    Ok(())
}
