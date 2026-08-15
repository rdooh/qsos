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
