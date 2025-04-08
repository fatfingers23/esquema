use atrium_api::agent::atp_agent::AtpAgent;
use atrium_api::agent::atp_agent::store::MemorySessionStore;
use atrium_api::agent::bluesky::AtprotoServiceType;
use atrium_api::did_doc::Service;
use atrium_api::types::LimitedNonZeroU8;
use atrium_api::types::string::AtIdentifier::{Did as AtIdentifierDid, Handle};
use atrium_api::types::string::{AtIdentifier, Did as DidType};
use atrium_common::resolver::Resolver;

use atrium_identity::{
    did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL},
    handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig, DnsTxtResolver},
};
use atrium_oauth::DefaultHttpClient;
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::{Parser, Subcommand};
use esquema_codegen::genapi;
use hickory_resolver::TokioAsyncResolver;
use log::info;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

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
    /// Generates rust types from a remote pds repository
    Remote(RepoGenerate),
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
struct RepoGenerate {
    /// The owner of the PDS repo
    #[arg(long)]
    handle: String,
    /// Namespace for recursion. Example xyz.statusphere will check for schemas under that collection, but xyz.statusphere.graphs will just check under the .graphs of xyz.statusphere
    #[arg(short, long)]
    namespace: String,
    /// The collection that holds the lexicon schema, it is usually 'com.atproto.lexicon.schema'
    #[arg(short, long, default_value = "com.atproto.lexicon.schema")]
    collection: String,
}

async fn did_generate_action(args: &RepoGenerate) -> Result<(), Box<dyn std::error::Error>> {
    // Currently just constructing in this command but may move to an app state with DI?
    let http_client = Arc::new(DefaultHttpClient::default());
    let did_resolver = CommonDidResolver::new(CommonDidResolverConfig {
        plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
        http_client: Arc::clone(&http_client),
    });

    let handle_resolver = AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
        dns_txt_resolver: HickoryDnsTxtResolver::default(),
        http_client: Arc::clone(&http_client),
    });

    let did = handle_resolver
        .resolve(&atrium_api::types::string::Handle::from_str(
            args.handle.as_str(),
        )?)
        .await?;

    let resolved_did = did_resolver.resolve(&did).await?;
    let pds_url = resolved_did
        .service
        .as_ref()
        .and_then(|services| {
            services
                .iter()
                .find(|service| service.r#type == "AtprotoPersonalDataServer")
                .map(|service| service.service_endpoint.clone())
        })
        .ok_or_else(|| "No valid PDS URL found for this DID")?; // Return error if not found

    //This endpoint needs your PDS endpoint, for example mine is "https://coral.us-east.host.bsky.network"
    let agent = AtpAgent::new(ReqwestClient::new(pds_url), MemorySessionStore::default());

    let records = agent
        .api
        .com
        .atproto
        .repo
        .list_records(
            atrium_api::com::atproto::repo::list_records::ParametersData {
                collection: args.collection.parse()?,
                cursor: None,
                limit: None,
                repo: AtIdentifier::Did(did.clone()),
                reverse: None,
            }
            .into(),
        )
        .await?;

    let record_uri_prefix = format!(
        "at://{}/{}/{}",
        did.as_str(),
        args.collection,
        args.namespace,
    );
    for ref record in &records.records {
        if record.uri.starts_with(record_uri_prefix.as_str()) {
            println!("Found it: {:?}", record);
        }
    }

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

struct HickoryDnsTxtResolver {
    resolver: TokioAsyncResolver,
}

impl Default for HickoryDnsTxtResolver {
    fn default() -> Self {
        Self {
            resolver: TokioAsyncResolver::tokio_from_system_conf()
                .expect("failed to create resolver"),
        }
    }
}

impl DnsTxtResolver for HickoryDnsTxtResolver {
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn Error + Send + Sync + 'static>> {
        Ok(self
            .resolver
            .txt_lookup(query)
            .await?
            .iter()
            .map(|txt| txt.to_string())
            .collect())
    }
}
