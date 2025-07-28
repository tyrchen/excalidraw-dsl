// src/main.rs
use clap::{Parser, Subcommand};
use excalidraw_dsl::EDSLCompiler;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "edsl",
    version,
    about = "ExcaliDraw-DSL Compiler - Generate Excalidraw diagrams from DSL",
    author = "ExcaliDraw-DSL Team"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert EDSL file to Excalidraw JSON
    #[command(alias = "compile")]
    Convert {
        /// Input EDSL file
        input: PathBuf,

        /// Output file path (defaults to input with .excalidraw extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Layout algorithm
        #[arg(short, long, value_enum, default_value = "dagre")]
        layout: LayoutAlgorithm,

        /// Validate input only (don't generate output)
        #[arg(long)]
        validate: bool,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Watch for file changes and recompile automatically
        #[arg(short, long)]
        watch: bool,
    },

    /// Run HTTP/WebSocket server for EDSL compilation
    #[command(alias = "serve")]
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "3002")]
        port: u16,

        /// Host to bind to
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate EDSL file syntax
    Validate {
        /// Input EDSL file
        input: PathBuf,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate Excalidraw JSON file
    #[command(alias = "validate-ex")]
    ValidateExcalidraw {
        /// Input Excalidraw file
        input: PathBuf,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Watch EDSL file and recompile on changes
    Watch {
        /// Input EDSL file
        input: PathBuf,

        /// Output file path (defaults to input with .excalidraw extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Train ML layout models
    #[cfg(feature = "ml-layout")]
    Train {
        /// Output directory for trained models
        #[arg(short, long, default_value = "./models")]
        output: PathBuf,

        /// Training configuration (quick-test, m4-optimized, or custom)
        #[arg(short, long, default_value = "quick-test")]
        config: String,

        /// Training phases to run (comma-separated: data, gnn, rl, constraint, enhanced, eval)
        #[arg(short, long, default_value = "data,gnn,rl,constraint,enhanced,eval")]
        phases: String,

        /// Number of training samples to generate
        #[arg(long, default_value = "1000")]
        samples: usize,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Use Apple M4 optimizations
        #[arg(long)]
        m4_optimize: bool,

        /// Skip model evaluation phase
        #[arg(long)]
        skip_eval: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum LayoutAlgorithm {
    Dagre,
    Force,
    Elk,
}

impl std::fmt::Display for LayoutAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutAlgorithm::Dagre => write!(f, "dagre"),
            LayoutAlgorithm::Force => write!(f, "force"),
            LayoutAlgorithm::Elk => write!(f, "elk"),
        }
    }
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");

        // Print the error chain
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {err}");
            source = err.source();
        }

        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Convert {
            input,
            output,
            layout,
            validate,
            verbose,
            watch,
        } => {
            if watch {
                run_watch(WatchArgs {
                    input,
                    output,
                    verbose,
                })
            } else {
                run_convert(ConvertArgs {
                    input,
                    output,
                    layout,
                    validate,
                    verbose,
                })
            }
        }
        Commands::Server {
            port,
            host,
            verbose,
        } => run_server(ServerArgs {
            port,
            host,
            verbose,
        }),
        Commands::Validate { input, verbose } => run_validate(ValidateArgs { input, verbose }),
        Commands::ValidateExcalidraw { input, verbose } => {
            run_validate_excalidraw(ValidateExcalidrawArgs { input, verbose })
        }
        Commands::Watch {
            input,
            output,
            verbose,
        } => run_watch(WatchArgs {
            input,
            output,
            verbose,
        }),
        #[cfg(feature = "ml-layout")]
        Commands::Train {
            output,
            config,
            phases,
            samples,
            verbose,
            m4_optimize,
            skip_eval,
        } => run_train(TrainArgs {
            output,
            config,
            phases,
            samples,
            verbose,
            m4_optimize,
            skip_eval,
        }),
    }
}

struct ConvertArgs {
    input: PathBuf,
    output: Option<PathBuf>,
    layout: LayoutAlgorithm,
    validate: bool,
    verbose: bool,
}

fn run_convert(args: ConvertArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        println!("Reading input file: {}", args.input.display());
    }

    // Read input file
    let input_content = std::fs::read_to_string(&args.input).map_err(|e| {
        format!(
            "Failed to read input file '{}': {}",
            args.input.display(),
            e
        )
    })?;

    // Create compiler
    let mut compiler = EDSLCompiler::new();

    // Validate mode
    if args.validate {
        if args.verbose {
            println!("Validating EDSL syntax...");
        }

        match compiler.validate(&input_content) {
            Ok(()) => {
                println!("✓ Validation passed!");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Validation failed: {e}");
                return Err(e.into());
            }
        }
    }

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("excalidraw");
        path
    });

    if args.verbose {
        println!("Compiling EDSL to Excalidraw JSON...");
        println!("Output file: {}", output_path.display());
        println!("Layout algorithm: {}", args.layout);
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

    if args.verbose {
        let element_count = count_elements_in_json(&output_json);
        println!("✓ Successfully generated {element_count} Excalidraw elements");
    }

    println!("Generated Excalidraw JSON: {}", output_path.display());

    Ok(())
}

struct ServerArgs {
    port: u16,
    host: String,
    verbose: bool,
}

fn run_server(args: ServerArgs) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "server")]
    {
        if args.verbose {
            println!("Starting EDSL server on {}:{}", args.host, args.port);
        }

        // Create runtime and run server
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            let state = excalidraw_dsl::server::http::AppState::new();
            excalidraw_dsl::server::http::start_server(args.port, state).await
        })?;

        Ok(())
    }

    #[cfg(not(feature = "server"))]
    {
        eprintln!("Server feature not enabled. Build with --features server to enable server functionality.");
        std::process::exit(1);
    }
}

struct ValidateArgs {
    input: PathBuf,
    verbose: bool,
}

fn run_validate(args: ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        println!("Validating file: {}", args.input.display());
    }

    // Read input file
    let input_content = std::fs::read_to_string(&args.input).map_err(|e| {
        format!(
            "Failed to read input file '{}': {}",
            args.input.display(),
            e
        )
    })?;

    // Create compiler and validate
    let mut compiler = EDSLCompiler::new();

    match compiler.validate(&input_content) {
        Ok(()) => {
            println!("✓ Validation passed!");
            if args.verbose {
                // Try to parse and show some statistics
                if let Ok(elements) = compiler.compile_to_elements(&input_content) {
                    println!("  - {} elements found", elements.len());
                    let nodes = elements.iter().filter(|e| e.r#type == "rectangle").count();
                    let edges = elements
                        .iter()
                        .filter(|e| e.r#type == "arrow" || e.r#type == "line")
                        .count();
                    println!("  - {nodes} nodes, {edges} edges");
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {e}");
            Err(e.into())
        }
    }
}

struct ValidateExcalidrawArgs {
    input: PathBuf,
    verbose: bool,
}

fn run_validate_excalidraw(args: ValidateExcalidrawArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        println!("Validating Excalidraw file: {}", args.input.display());
    }

    // Read input file
    let input_content = std::fs::read_to_string(&args.input).map_err(|e| {
        format!(
            "Failed to read input file '{}': {}",
            args.input.display(),
            e
        )
    })?;

    // Create compiler and validate
    let compiler = EDSLCompiler::new();

    match compiler.validate_excalidraw(&input_content) {
        Ok(()) => {
            println!("✓ Valid Excalidraw file!");
            if args.verbose {
                // Try to parse and show some statistics
                use serde_json::Value;
                if let Ok(value) = serde_json::from_str::<Value>(&input_content) {
                    let element_count = match &value {
                        Value::Array(elements) => elements.len(),
                        Value::Object(map) => map
                            .get("elements")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.len())
                            .unwrap_or(0),
                        _ => 0,
                    };
                    println!("  - {element_count} elements found");
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {e}");
            Err(e.into())
        }
    }
}

struct WatchArgs {
    input: PathBuf,
    output: Option<PathBuf>,
    verbose: bool,
}

#[cfg(feature = "ml-layout")]
struct TrainArgs {
    output: PathBuf,
    config: String,
    phases: String,
    samples: usize,
    verbose: bool,
    m4_optimize: bool,
    skip_eval: bool,
}

fn run_watch(args: WatchArgs) -> Result<(), Box<dyn std::error::Error>> {
    use notify::{Event, EventKind, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("excalidraw");
        path
    });

    println!("Watching {} for changes...", args.input.display());
    println!("Output will be written to: {}", output_path.display());

    // Initial compilation
    compile_file(&args.input, &output_path, args.verbose)?;

    // Create a channel to receive the events
    let (tx, rx) = channel();

    // Create a watcher object with debounced events
    let mut watcher =
        notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
            Ok(event) => {
                if let Err(e) = tx.send(event) {
                    eprintln!("Error sending watch event: {e}");
                }
            }
            Err(e) => eprintln!("Watch error: {e:?}"),
        })?;

    // Add a path to be watched
    watcher.watch(&args.input, RecursiveMode::NonRecursive)?;

    // Main watch loop
    loop {
        match rx.recv() {
            Ok(event) => match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {
                    println!("\n📝 File changed, recompiling...");
                    match compile_file(&args.input, &output_path, args.verbose) {
                        Ok(_) => println!("✓ Compilation successful"),
                        Err(e) => eprintln!("✗ Compilation failed: {e}"),
                    }
                }
                EventKind::Remove(_) => {
                    eprintln!("⚠️  Input file was removed");
                }
                _ => {}
            },
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }
}

fn compile_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_content = std::fs::read_to_string(input_path)?;
    let mut compiler = EDSLCompiler::new();

    if verbose {
        println!(
            "Compiling {} -> {}",
            input_path.display(),
            output_path.display()
        );
    }

    let output_json = compiler.compile(&input_content)?;
    std::fs::write(output_path, &output_json)?;

    if verbose {
        let element_count = count_elements_in_json(&output_json);
        println!("Generated {element_count} elements");
    }

    Ok(())
}

fn count_elements_in_json(json: &str) -> usize {
    // Simple count by looking for element objects
    json.matches(r#""type":"#).count()
}

#[cfg(feature = "ml-layout")]
fn run_train(args: TrainArgs) -> Result<(), Box<dyn std::error::Error>> {
    use excalidraw_dsl::training::{config::TrainingConfig, TrainingOrchestrator};

    if args.verbose {
        println!("🚀 Starting ML Layout Training");
        println!("Output directory: {}", args.output.display());
        println!("Configuration: {}", args.config);
        println!("Phases: {}", args.phases);
        println!("Samples: {}", args.samples);
        if args.m4_optimize {
            println!("🍎 Apple M4 optimizations enabled");
        }
    }

    // Create output directory
    std::fs::create_dir_all(&args.output)?;

    // Parse requested phases
    let requested_phases: Vec<&str> = args.phases.split(',').map(|s| s.trim()).collect();

    // Create training configuration
    let mut config = match args.config.as_str() {
        "quick-test" => TrainingConfig::quick_test(),
        "m4-optimized" => TrainingConfig::m4_optimized(),
        _ => {
            eprintln!("Unknown config '{}'. Using quick-test.", args.config);
            TrainingConfig::quick_test()
        }
    };

    // Apply command line overrides
    config.output_dir = args.output.clone();
    config.data_config.num_samples = args.samples;

    if args.m4_optimize {
        config = TrainingConfig::m4_optimized();
        config.output_dir = args.output.clone();
        config.data_config.num_samples = args.samples;
    }

    // Skip evaluation if requested
    if args.skip_eval {
        // Skip evaluation phase (handled by phase selection)
    }

    if args.verbose {
        println!("\n📋 Training Configuration:");
        println!("  Data samples: {}", config.data_config.num_samples);
        println!("  GNN epochs: {}", config.gnn_config.model.max_epochs);
        println!("  RL episodes: {}", config.rl_config.num_episodes);
        println!(
            "  Constraint epochs: {}",
            config.constraint_config.model.max_epochs
        );
        println!(
            "  Enhanced epochs: {}",
            config.enhanced_config.num_integration_epochs
        );
        println!("  Device: {:?}", config.device);
    }

    // Create and run training orchestrator
    let mut orchestrator = TrainingOrchestrator::new(config)?;

    println!("\n🎯 Starting training pipeline...");

    // Run training phases
    for phase in &requested_phases {
        match *phase {
            "data" => {
                if args.verbose {
                    println!("\n📊 Running data generation phase...");
                }
                orchestrator.run_data_generation()?;
            }
            "gnn" => {
                if args.verbose {
                    println!("\n🧠 Running GNN training phase...");
                }
                orchestrator.run_gnn_training()?;
            }
            "rl" => {
                if args.verbose {
                    println!("\n🎮 Running RL training phase...");
                }
                orchestrator.run_rl_training()?;
            }
            "constraint" => {
                if args.verbose {
                    println!("\n🧩 Running constraint training phase...");
                }
                orchestrator.run_constraint_training()?;
            }
            "enhanced" => {
                if args.verbose {
                    println!("\n⚡ Running enhanced model training phase...");
                }
                orchestrator.run_enhanced_training()?;
            }
            "eval" => {
                if !args.skip_eval {
                    if args.verbose {
                        println!("\n📈 Running evaluation phase...");
                    }
                    orchestrator.run_evaluation()?;
                }
            }
            _ => {
                eprintln!("⚠️  Unknown phase '{phase}', skipping...");
            }
        }
    }

    // Generate final report
    let metrics_path = args.output.join("training_metrics.json");
    let report_path = args.output.join("training_report.md");

    orchestrator.save_metrics(&metrics_path)?;
    orchestrator.generate_report(&report_path)?;

    println!("\n✅ Training completed successfully!");
    println!("📊 Metrics saved to: {}", metrics_path.display());
    println!("📄 Report generated: {}", report_path.display());
    println!("🗂️  Models saved in: {}", args.output.display());

    if args.verbose {
        println!("\n📋 Training Summary:");
        if let Some(metrics) = orchestrator.get_metrics() {
            if let Some(ref data_stats) = metrics.data_generation {
                println!(
                    "  📊 Generated {} training samples",
                    data_stats.total_samples
                );
            }
            if let Some(ref gnn_results) = metrics.gnn_training {
                println!("  🧠 GNN final loss: {:.6}", gnn_results.final_loss);
            }
            if let Some(ref rl_results) = metrics.rl_training {
                println!("  🎮 RL average reward: {:.3}", rl_results.avg_reward);
            }
            if let Some(ref constraint_results) = metrics.constraint_training {
                println!(
                    "  🧩 Constraint accuracy: {:.1}%",
                    constraint_results.accuracy * 100.0
                );
            }
            if let Some(ref enhanced_results) = metrics.enhanced_training {
                println!(
                    "  ⚡ Enhanced quality score: {:.3}",
                    enhanced_results.quality_score
                );
            }
            if let Some(ref eval_results) = metrics.evaluation {
                println!(
                    "  📈 Overall evaluation score: {:.3}",
                    eval_results.overall_score
                );
            }
        }
    }

    Ok(())
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

        // Create args
        let args = ConvertArgs {
            input: input_file.path().to_path_buf(),
            output: Some(output_file.path().to_path_buf()),
            layout: LayoutAlgorithm::Dagre,
            validate: false,
            verbose: false,
        };

        // Run the CLI
        let result = run_convert(args);
        if let Err(e) = &result {
            eprintln!("CLI test error: {e}");
        }
        assert!(result.is_ok());

        // Check that output file was created and contains valid JSON
        let output_content = fs::read_to_string(output_file.path()).unwrap();
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

        let args = ValidateArgs {
            input: input_file.path().to_path_buf(),
            verbose: false,
        };

        let result = run_validate(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_algorithm_display() {
        assert_eq!(format!("{}", LayoutAlgorithm::Dagre), "dagre");
        assert_eq!(format!("{}", LayoutAlgorithm::Force), "force");
    }
}
