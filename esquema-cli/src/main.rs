use clap::Parser;
use esquema_codegen::genapi;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "./esquema-example/lexicons")]
    lexdir: PathBuf,
    #[arg(short, long, default_value = "./esquema-example/src/lexicons")]
    outdir: PathBuf,
    #[arg(short, long, default_value = "lexicons")]
    modName: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    //TODO do crud operations like delete the files and create the folder if needed
    //TODO pass down the mod name thats for the name of the folder to create instead of just lexicons
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
