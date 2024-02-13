use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use directories::UserDirs;
use miette::{Context, IntoDiagnostic};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short = 'p', long, env)]
    garden_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Write {
        #[clap(short, long)]
        title: Option<String>,
    },
}

fn main() -> miette::Result<()> {
    let args = Args::parse();

    let Some(garden_path) = args.garden_path.or_else(get_default_garden_path) else {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::ValueValidation, format!("garden path not found"))
            .exit()
    };

    if !garden_path.exists() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "garden directory `{}` doesn't exist, or is inaccessible",
                garden_path.display()
            ),
        )
        .exit()
    }

    match args.cmd {
        Commands::Write { title } => garden::write(garden_path, title).wrap_err("garden::write"),
    }
}

fn get_default_garden_path() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| user_dirs.home_dir().join("garden"))
}
