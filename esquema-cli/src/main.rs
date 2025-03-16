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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    //TODO read these from the file?
    let results = genapi(&args.lexdir, &args.outdir, &[("xyz.statusphere", None)])?;
    for path in &results {
        println!(
            "{} ({} bytes)",
            path.as_ref().display(),
            fs::metadata(path.as_ref())?.len()
        );
    }
    Ok(())
}
