use clap::{Parser, Subcommand};
use esquema_codegen::genapi;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates rust types from your json lexicon files
    Generate(Generate),
}

#[derive(Parser, Debug)]
#[command(name = "generate", about = "Generates Rust types from Lexicon files")]
struct Generate {
    /// The directory location of your lexicon json files. Works recursively
    #[arg(short, long, default_value = "./esquema-example/lexicons")]
    lexdir: PathBuf,
    /// The output directory for the rust files, if not there it will create the folder
    #[arg(short, long, default_value = "./esquema-example/src/lexicons")]
    outdir: PathBuf,
}

fn generate_action(args: &Generate) -> Result<(), Box<dyn std::error::Error>> {
    if !args.outdir.exists() {
        fs::create_dir_all(&args.outdir)?;
    }
    let results = genapi(&args.lexdir, &args.outdir)?;
    for path in &results {
        println!(
            "{} ({} bytes)",
            path.as_ref().display(),
            fs::metadata(path.as_ref())?.len()
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(args) => generate_action(args),
        _ => unreachable!("That was not a valid command"),
    }
}
