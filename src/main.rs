// src/main.rs
use clap::{value_parser, Arg, Command};
use excalidraw_dsl::EDSLCompiler;
use std::path::PathBuf;
use std::process;

fn main() {
    env_logger::init();

    let matches = Command::new("edsl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("ExcaliDraw-DSL Compiler - Generate Excalidraw diagrams from DSL")
        .author("ExcaliDraw-DSL Team")
        .arg(
            Arg::new("input")
                .help("Input EDSL file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .value_name("FILE")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("layout")
                .short('l')
                .long("layout")
                .help("Layout algorithm")
                .value_name("ALGORITHM")
                .value_parser(["dagre", "force"]),
        )
        .arg(
            Arg::new("validate")
                .long("validate")
                .help("Validate input only (don't generate output)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if let Err(e) = run(matches) {
        eprintln!("Error: {}", e);

        // Print the error chain
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }

        process::exit(1);
    }
}

fn run(matches: clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = PathBuf::from(matches.get_one::<String>("input").unwrap());
    let verbose = matches.get_flag("verbose");

    if verbose {
        println!("Reading input file: {}", input_path.display());
    }

    // Read input file
    let input_content = std::fs::read_to_string(&input_path).map_err(|e| {
        format!(
            "Failed to read input file '{}': {}",
            input_path.display(),
            e
        )
    })?;

    // Create compiler
    #[allow(unused_mut)]
    let mut compiler = EDSLCompiler::new();

    // Override layout if specified
    if let Some(layout) = matches.get_one::<String>("layout") {
        if verbose {
            println!("Using layout algorithm: {}", layout);
        }
        // Note: In a full implementation, we'd want to modify the global config
        // For now, users can specify layout in the EDSL frontmatter
    }

    // Validate mode
    if matches.get_flag("validate") {
        if verbose {
            println!("Validating EDSL syntax...");
        }

        match compiler.validate(&input_content) {
            Ok(()) => {
                println!("✓ Validation passed!");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Validation failed: {}", e);
                return Err(e.into());
            }
        }
    }

    // Determine output path
    let output_path = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut path = input_path.clone();
            path.set_extension("json");
            path
        });

    if verbose {
        println!("Compiling EDSL to Excalidraw JSON...");
        println!("Output file: {}", output_path.display());
    }

    // Compile EDSL
    let output_json = compiler.compile(&input_content)?;

    // Write output
    std::fs::write(&output_path, &output_json).map_err(|e| {
        format!(
            "Failed to write output file '{}': {}",
            output_path.display(),
            e
        )
    })?;

    if verbose {
        let element_count = count_elements_in_json(&output_json);
        println!(
            "✓ Successfully generated {} Excalidraw elements",
            element_count
        );
    }

    println!("Generated Excalidraw JSON: {}", output_path.display());

    Ok(())
}

fn count_elements_in_json(json: &str) -> usize {
    // Simple count by looking for element objects
    json.matches(r#""type":"#).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cli_basic_compilation() {
        let edsl_content = r#"
---
layout: dagre
---

user[User]
api[API]
db[Database]

user -> api -> db
        "#;

        // Create temporary input file
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, edsl_content).unwrap();

        // Create temporary output file
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        // Build command matches
        let matches = Command::new("edsl")
            .arg(Arg::new("input").index(1))
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new("layout")
                    .short('l')
                    .long("layout")
                    .value_parser(["dagre", "force"]),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("validate")
                    .long("validate")
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches_from(vec![
                "edsl",
                input_file.path().to_str().unwrap(),
                "-o",
                output_path,
            ]);

        // Run the CLI
        let result = run(matches);
        if let Err(e) = &result {
            eprintln!("CLI test error: {}", e);
        }
        assert!(result.is_ok());

        // Check that output file was created and contains valid JSON
        let output_content = fs::read_to_string(output_path).unwrap();
        assert!(output_content.contains("type"));
        assert!(output_content.contains("rectangle"));
    }

    #[test]
    fn test_cli_validation_mode() {
        let edsl_content = r#"
        web_server[Web Server]
        database[Database] { shape: cylinder; }
        web_server -> database
        "#;

        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, edsl_content).unwrap();

        let matches = Command::new("edsl")
            .arg(Arg::new("input").index(1))
            .arg(
                Arg::new("layout")
                    .short('l')
                    .long("layout")
                    .value_parser(["dagre", "force"]),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("validate")
                    .long("validate")
                    .action(clap::ArgAction::SetTrue),
            )
            .get_matches_from(vec![
                "edsl",
                input_file.path().to_str().unwrap(),
                "--validate",
            ]);

        let result = run(matches);
        assert!(result.is_ok());
    }
}
