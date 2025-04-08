use atrium_api::agent::atp_agent::AtpAgent;
use atrium_api::agent::atp_agent::store::MemorySessionStore;
use atrium_api::types::LimitedNonZeroU8;
use atrium_api::types::string::AtIdentifier::Did as AtIdentifierDid;
use atrium_api::types::string::Did as DidType;
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::{Parser, Subcommand};
use esquema_codegen::genapi;
use std::fs;
use std::path::PathBuf;
use atrium_identity::identity_resolver::IdentityResolver;

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
#[command(
    name = "generate",
    about = "Generates rust types from ATProto lexicons"
)]
struct Generate {
    /// Generates rust types from your local json lexicon files
    #[command(subcommand)]
    subcommand: GenerateCommands,
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    /// Generates rust types from your local json lexicon files
    Local(LocalGenerate),
    Remote(DidGenerate),
}

#[derive(Parser, Debug)]
#[command(
    name = "local",
    about = "Generates Rust types from local Lexicon JSON files"
)]
struct LocalGenerate {
    /// The directory location of your lexicon json files. Works recursively
    #[arg(short, long)]
    lexdir: PathBuf,
    /// The output directory for the rust files, if not there it will create the folder
    #[arg(short, long, default_value = "./src")]
    outdir: PathBuf,
}

fn local_generate_action(args: &LocalGenerate) -> Result<(), Box<dyn std::error::Error>> {
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

#[derive(Parser, Debug)]
#[command(
    name = "remote",
    about = "Generates Rust types from remote Lexicon ATProto records"
)]
struct DidGenerate {
    /// The remote DID to check against
    #[arg(short, long)]
    did: String,
    /// Namespace for recursion. Example xyz.statusphere will check for schemas under that collection, but xyz.statusphere.graphs will just check under the .graphs of xyz.statusphere
    #[arg(short, long)]
    namespace: PathBuf,
    //TODO i don't think pds is proper here?
    /// The url for your PDS, like https://public.api.bsky.app
    #[arg(short, long, default_value = "https://public.api.bsky.app")]
    pds: String,
}

async fn did_generate_action(args: &DidGenerate) -> Result<(), Box<dyn std::error::Error>> {
    let agent = AtpAgent::new(
        ReqwestClient::new(args.pds.as_str()),
        MemorySessionStore::default(),
    );
    let parsed_did = AtIdentifierDid(DidType::new(args.did.clone())?);
    //Im not sure atrium can do this? may be a manual thing for now?
    // let identity = IdentityResolver::new()
    // let did_doc = agent.api.com.atproto.identity.

    //This endpoint needs your PDS endpoint, for example mine is "https://coral.us-east.host.bsky.network"
    let repos = agent
        .api
        .com
        .atproto
        .repo
        .describe_repo(
            atrium_api::com::atproto::repo::describe_repo::ParametersData { repo: parsed_did }
                .into(),
        )
        .await?;
    println!("{:?}", repos);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(Generate { subcommand }) => match subcommand {
            GenerateCommands::Local(args) => local_generate_action(args),
            GenerateCommands::Remote(args) => did_generate_action(args).await,
        },
    }
}
