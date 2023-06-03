use indicatif::{ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};
use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Compile the project and display any errors.
    Compile {
        /// The root path of the Unity project
        #[clap(value_hint = ValueHint::FilePath)]
        project_path: PathBuf,
    },

    /// Compile the project and run tests
    Test {
        /// The root path of the Unity project
        #[clap(value_hint = ValueHint::FilePath)]
        project_path: PathBuf,

        /// Which set of tests to run
        #[arg(short, value_enum)]
        mode: unitool::TestMode,

        /// Optional `;`-delimited filters
        #[arg(short)]
        filters: Option<String>,

        /// The assemblies to include
        #[arg(short, default_value="EditTests;PlayTests")]
        assemblies: String,
    },
}

fn main() {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Compile { project_path } => {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(ProgressStyle::with_template("{spinner} [{elapsed_precise}] {msg}").unwrap());
            spinner.enable_steady_tick(Duration::from_millis(120));
            spinner.set_message("Compiling...");

            let errs = unitool::compile(&project_path).unwrap();
            if errs.is_empty() {
                spinner.finish_with_message(
                    format!("{}",
                            unitool::display::green("Compilation succeeded")));
            } else {
                spinner.finish_with_message(
                    format!("{}",
                            unitool::display::red("Compilation failed")));
                for err in &errs {
                    println!("  {}", err);
                }
            }
        },
        SubCommand::Test { project_path, mode, assemblies, filters } => {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(ProgressStyle::with_template("{spinner} [{elapsed_precise}] {msg}").unwrap());
            spinner.enable_steady_tick(Duration::from_millis(120));
            spinner.set_message("Compiling and running tests...");

            let (errs, results) = unitool::test(&project_path, mode, &assemblies, filters).unwrap();
            if let Some(results) = results {
                println!("{}", results);
            } else {
                for err in &errs {
                    println!("  {}", err);
                }
            }
        }
    }
}
