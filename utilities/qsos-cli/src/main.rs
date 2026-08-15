use clap::{Parser, Subcommand};
use qsos_core::{ProjectLayout, EXIT_ERROR, EXIT_SUCCESS};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "qsos", about = "QSOS utilities — lint, graph, query, ingest")]
struct Cli {
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Static compliance checks (ADR, Gherkin, lifecycle, DSL)
    Lint {
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        sync: bool,
    },
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
    Query {
        #[arg(long, conflicts_with_all = ["file", "blast_radius"])]
        ticket: Option<String>,
        #[arg(long, conflicts_with_all = ["ticket", "blast_radius"])]
        file: Option<PathBuf>,
        #[arg(long, conflicts_with_all = ["ticket", "file"])]
        blast_radius: Option<String>,
    },
    Ingest,
    /// Scaffold a QSOS-governed project layout
    Init {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        test_runner: Option<String>,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum GraphAction {
    Compile,
}

fn main() {
    let cli = Cli::parse();
    let layout = ProjectLayout::discover(&cli.root);

    let code = match cli.command {
        Commands::Lint { file, sync } => run_lint(&layout, file.as_deref(), sync),
        Commands::Graph { action } => run_graph(&layout, action),
        Commands::Query {
            ticket,
            file,
            blast_radius,
        } => run_query(&layout, ticket.as_deref(), file.as_deref(), blast_radius.as_deref()),
        Commands::Ingest => run_ingest(&layout),
        Commands::Init {
            name,
            prefix,
            description,
            check,
            dry_run,
            test_runner,
        } => run_init(
            &cli.root,
            name.as_deref(),
            prefix.as_deref(),
            description.as_deref(),
            test_runner.as_deref(),
            check,
            dry_run,
        ),
    };

    process::exit(code);
}

fn run_lint(layout: &ProjectLayout, file: Option<&std::path::Path>, sync: bool) -> i32 {
    if sync {
        eprintln!("qsos lint --sync not yet implemented (QSO-021)");
        return EXIT_ERROR;
    }

    let report = match file {
        Some(path) => qsos_lint::lint_file(layout, path),
        None => qsos_lint::lint_project(layout),
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into()));
    report.exit_code()
}

fn run_graph(layout: &ProjectLayout, action: GraphAction) -> i32 {
    let registry = match action {
        GraphAction::Compile => qsos_graph::compile_and_write(layout),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&registry).unwrap_or_else(|_| "{}".into())
    );
    EXIT_SUCCESS
}

fn run_query(
    layout: &ProjectLayout,
    ticket: Option<&str>,
    file: Option<&std::path::Path>,
    blast_radius: Option<&str>,
) -> i32 {
    let result = match (ticket, file, blast_radius) {
        (Some(id), None, None) => qsos_graph::query_ticket(layout, id),
        (None, Some(path), None) => {
            let rel = layout.rel_path(path);
            qsos_graph::query_file(layout, &rel)
        }
        (None, None, Some(artifact)) => qsos_graph::query_blast_radius(layout, artifact),
        _ => {
            eprintln!("qsos query requires exactly one of: --ticket, --file, --blast-radius");
            return EXIT_ERROR;
        }
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".into())
    );
    EXIT_SUCCESS
}

fn run_ingest(layout: &ProjectLayout) -> i32 {
    match qsos_ingest::ingest(layout) {
        Ok(()) => EXIT_SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            EXIT_ERROR
        }
    }
}

fn run_init(
    root: &std::path::Path,
    name: Option<&str>,
    prefix: Option<&str>,
    description: Option<&str>,
    test_runner: Option<&str>,
    check: bool,
    dry_run: bool,
) -> i32 {
    use qsos_init::{InitConfig, InitMode};

    let mode = if check {
        InitMode::Check
    } else if dry_run {
        InitMode::DryRun
    } else {
        InitMode::Write
    };

    let config = if check {
        InitConfig {
            name: name.unwrap_or("project").to_string(),
            prefix: prefix
                .map(|p| qsos_init::normalize_prefix(p).unwrap_or_else(|_| p.to_string()))
                .unwrap_or_else(|| "XXX-".into()),
            description: description.unwrap_or("").to_string(),
            test_runner: test_runner.map(str::to_string),
        }
    } else {
        let name = match name {
            Some(n) => match qsos_init::validate_name(n) {
                Ok(()) => n.to_string(),
                Err(e) => {
                    eprintln!("{e}");
                    return EXIT_ERROR;
                }
            },
            None => {
                eprintln!("qsos init requires --name (unless --check)");
                return EXIT_ERROR;
            }
        };
        let prefix = match prefix {
            Some(p) => match qsos_init::normalize_prefix(p) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{e}");
                    return EXIT_ERROR;
                }
            },
            None => {
                eprintln!("qsos init requires --prefix (unless --check)");
                return EXIT_ERROR;
            }
        };
        InitConfig {
            name,
            prefix,
            description: description.unwrap_or("QSOS-governed project").to_string(),
            test_runner: test_runner.map(str::to_string),
        }
    };

    match qsos_init::run_init(root, &config, mode) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into()));
            report.exit_code(mode)
        }
        Err(msg) => {
            eprintln!("{msg}");
            EXIT_ERROR
        }
    }
}
