//! FPL Optimizer CLI
//!
//! Command-line interface for Fantasy Premier League squad optimization.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use fpl_optimizer::{
    config::Config,
    data::{CsvLoader, ProblemData},
    output::{display_result, write_csv, write_json},
    problems::{
        BenchBoostSquad, DifferentialSquad, IterativeConfig, IterativeSquads, LimitedBestSquad,
        NoLimitBest11, Problem, SetAndForget, WeightedBenchSquad,
    },
    types::Cost,
};

#[derive(Parser)]
#[command(name = "fpl")]
#[command(author, version, about = "FPL Squad Optimization using MILP")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// Data directory path
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// Output directory path
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve an optimization problem
    Solve {
        /// Problem type to solve
        #[arg(short, long, default_value = "limited")]
        problem: ProblemType,

        /// Budget in millions (e.g., 100.0)
        #[arg(short, long)]
        budget: Option<f64>,
    },

    /// Generate multiple diverse squads
    Iterative {
        /// Number of squads to generate
        #[arg(short, long, default_value = "50")]
        count: usize,
    },

    /// Solve all problem types
    All,

    /// Show version and configuration
    Info,
}

#[derive(Clone, Copy, ValueEnum, Debug)]
enum ProblemType {
    /// No budget or team limits - theoretical best 11
    NoLimit,
    /// Standard squad with all FPL constraints
    Limited,
    /// Squad with bench players weighted at 0.1
    WeightedBench,
    /// Bench boost chip - all 15 players count
    BenchBoost,
    /// Only players with ownership < 5%
    Differential,
    /// Optimized for multiple gameweeks
    SetAndForget,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config if exists
    let config = if cli.config.exists() {
        Config::load(&cli.config).unwrap_or_default()
    } else {
        Config::default()
    };

    // Determine data directory
    let data_dir = cli
        .data_dir
        .unwrap_or_else(|| PathBuf::from(&config.general.data_dir));

    // Determine output directory
    let output_dir = cli
        .output_dir
        .unwrap_or_else(|| PathBuf::from(&config.output.output_dir));

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Load data
    if cli.verbose {
        println!("Loading data from: {}", data_dir.display());
    }

    let players = CsvLoader::load_players(&data_dir).context("Failed to load player data")?;
    let projections =
        CsvLoader::load_projections(&data_dir).context("Failed to load projection data")?;
    let data = ProblemData::new(players, projections);

    if cli.verbose {
        println!(
            "Loaded {} players with projections for {:?} gameweeks",
            data.num_players(),
            data.gameweeks()
        );
    }

    match cli.command {
        Commands::Solve { problem, budget } => {
            let budget_cost = budget.map(Cost::from_millions).unwrap_or(config.budget());

            let result = match problem {
                ProblemType::NoLimit => {
                    println!("Solving: No Limit Best 11");
                    NoLimitBest11.solve(&data)?
                }
                ProblemType::Limited => {
                    println!("Solving: Limited Best Squad (Budget: {})", budget_cost);
                    LimitedBestSquad::with_budget(budget_cost).solve(&data)?
                }
                ProblemType::WeightedBench => {
                    println!("Solving: Weighted Bench Squad");
                    WeightedBenchSquad::new(budget_cost, config.bench.weight).solve(&data)?
                }
                ProblemType::BenchBoost => {
                    println!("Solving: Bench Boost Squad");
                    BenchBoostSquad::with_budget(budget_cost).solve(&data)?
                }
                ProblemType::Differential => {
                    println!(
                        "Solving: Differential Squad (max {}% ownership)",
                        config.differential.max_ownership
                    );
                    DifferentialSquad::new(budget_cost, config.differential.max_ownership)
                        .solve(&data)?
                }
                ProblemType::SetAndForget => {
                    println!("Solving: Set and Forget");
                    SetAndForget::new(budget_cost, config.bench.weight).solve(&data)?
                }
            };

            // Display result
            println!("{}", display_result(&result));

            // Write CSV output
            let csv_path = output_dir.join(format!("{}.csv", result.problem_name.to_lowercase().replace(' ', "_")));
            write_csv(&result, &csv_path)?;
            println!("Results written to: {}", csv_path.display());
        }

        Commands::Iterative { count } => {
            println!("Generating {} diverse squads...", count);

            let iterative = IterativeSquads::new(IterativeConfig {
                iterations: count,
                cutoff: config.iterative.cutoff,
                seed: config.iterative.random_seed,
                budget: config.budget(),
            });

            let results = iterative.solve_all(&data)?;

            // Write JSON output
            let json_path = output_dir.join("iterative_model.json");
            write_json(&results, &json_path)?;
            println!("Results written to: {}", json_path.display());

            // Summary
            println!("\nGenerated {} squads", results.len());
            if let Some(first) = results.first() {
                println!("First squad players: {:?}", first.players);
            }
        }

        Commands::All => {
            println!("Solving all problem types...\n");

            let budget = config.budget();

            let problems: Vec<Box<dyn Problem>> = vec![
                Box::new(NoLimitBest11),
                Box::new(LimitedBestSquad::with_budget(budget)),
                Box::new(WeightedBenchSquad::new(budget, config.bench.weight)),
                Box::new(BenchBoostSquad::with_budget(budget)),
                Box::new(DifferentialSquad::new(
                    budget,
                    config.differential.max_ownership,
                )),
                Box::new(SetAndForget::new(budget, config.bench.weight)),
            ];

            for problem in problems {
                println!("Solving: {}", problem.name());

                match problem.solve(&data) {
                    Ok(result) => {
                        println!("  Expected Points: {:.2}", result.objective_value);
                        println!("  Total Cost: {}", result.total_cost);

                        // Write CSV
                        let csv_path = output_dir.join(format!("{}.csv", problem.output_name()));
                        write_csv(&result, &csv_path)?;
                        println!("  Output: {}", csv_path.display());
                    }
                    Err(e) => {
                        println!("  Error: {}", e);
                    }
                }
                println!();
            }
        }

        Commands::Info => {
            println!("FPL Optimizer v{}", env!("CARGO_PKG_VERSION"));
            println!();
            println!("Configuration:");
            println!("  Budget: {}", config.budget());
            println!("  Team Limit: {} players per team", config.constraints.team_limit);
            println!("  Bench Weight: {}", config.bench.weight);
            println!("  Max Ownership (Differential): {}%", config.differential.max_ownership);
            println!();
            println!("Data:");
            println!("  Players: {}", data.num_players());
            println!("  Teams: {}", data.num_teams());
            println!("  Gameweeks: {:?}", data.gameweeks());
        }
    }

    Ok(())
}
