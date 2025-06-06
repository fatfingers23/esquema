use anyhow::anyhow;
use atrium_api::types::string::Nsid;
use atrium_api::{
    agent::atp_agent::AtpAgent, agent::atp_agent::store::MemorySessionStore,
    types::string::AtIdentifier,
};
use atrium_common::resolver::Resolver;
use atrium_identity::{
    did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL},
    handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig, DnsTxtResolver},
};
use atrium_lex::LexiconDoc;
use atrium_oauth::DefaultHttpClient;
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::{Parser, Subcommand};
use esquema_codegen::{gen_from_lexicon_docs, genapi};
use hickory_resolver::TokioAsyncResolver;
use std::{fs, path::PathBuf, str::FromStr, sync::Arc};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates rust types from ATProto lexicons
    Generate(Generate),
}

#[derive(Parser, Debug)]
#[command(
    name = "generate",
    about = "Generates rust types from ATProto lexicons"
)]
struct Generate {
    /// Type of lexicon generation
    #[command(subcommand)]
    subcommand: GenerateCommands,
}

#[derive(Subcommand, Debug)]
enum GenerateCommands {
    /// Generates rust types from your local JSON lexicon files
    Local(LocalGenerate),
    /// Generates rust types from a remote AT Protocol Lexicon schema record
    Remote(RepoGenerate),
}

#[derive(Parser, Debug)]
#[command(
    name = "local",
    about = "Generates Rust types from local Lexicon JSON files"
)]
struct LocalGenerate {
    /// The directory location of your lexicon JSON files. Works recursively
    #[arg(short, long)]
    lexdir: PathBuf,
    /// The output directory for the rust files, if not there, it will create the folder
    #[arg(short, long)]
    outdir: PathBuf,
    /// If set, the output is a module instead of a library
    #[arg(short, long)]
    module: Option<String>,
}

fn local_generate_action(args: &LocalGenerate) -> anyhow::Result<()> {
    let results =
        genapi(&args.lexdir, &args.outdir, &args.module).map_err(|e| anyhow!(e.to_string()))?;

    for path in &results {
        log::info!(
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
    about = "Generates Rust types from remote Lexicon ATProto Lexicon schema records"
)]
struct RepoGenerate {
    /// The owner of the PDS repo
    #[arg(long)]
    handle: String,
    /// Namespace for recursion.
    /// Example xyz.statusphere will check for all schemas prefixed with that,
    /// but xyz.statusphere.graphs will just check under the .graphs of xyz.statusphere
    #[arg(short, long)]
    namespace: String,
    /// The output directory for the rust files, if not there, it will create the folder
    #[arg(short, long)]
    outdir: PathBuf,
    /// The collection that holds the lexicon schema, it is usually 'com.atproto.lexicon.schema'
    #[arg(short, long, default_value = "com.atproto.lexicon.schema")]
    collection: String,
    /// If set, the output is a module instead of a library
    #[arg(short, long)]
    module: Option<String>,
}

/// Generates local Rust types from AT Protocol lexicon schema records
async fn generate_from_record_action(args: &RepoGenerate) -> anyhow::Result<()> {
    // Currently just constructing in this command but may move to an app state with DI?
    // Seems like over kill unless it ends up being used else where
    let http_client = Arc::new(DefaultHttpClient::default());
    //finds the did document from the users did
    let did_resolver = CommonDidResolver::new(CommonDidResolverConfig {
        plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
        http_client: Arc::clone(&http_client),
    });

    //gets the users did from their handle
    let handle_resolver = AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
        dns_txt_resolver: HickoryDnsTxtResolver::default(),
        http_client: Arc::clone(&http_client),
    });

    let handle = atrium_api::types::string::Handle::from_str(args.handle.as_str())
        .map_err(|e| anyhow!(e.to_string()))?;
    let did = handle_resolver.resolve(&handle).await?;

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
        .ok_or_else(|| anyhow!("No valid PDS URL found for this DID"))?;

    //This endpoint needs your PDS endpoint, for example mine is "https://coral.us-east.host.bsky.network"
    let agent = AtpAgent::new(ReqwestClient::new(pds_url), MemorySessionStore::default());
    let records = agent
        .api
        .com
        .atproto
        .repo
        .list_records(
            atrium_api::com::atproto::repo::list_records::ParametersData {
                collection: Nsid::new(args.collection.clone()).map_err(|e| anyhow!(e))?,
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
    let mut lexicon_docs: Vec<LexiconDoc> = Vec::new();
    for ref record in &records.records {
        if record.uri.starts_with(record_uri_prefix.as_str()) {
            //HACK (slightly)
            //We are using serde_json directly instead of the below because it currently uses .unwrap
            //and we want a friendlier error
            // let doc_result = LexiconDoc::try_from_unknown(record.data.value.clone());
            let json = serde_json::to_vec(&record.data.value)?;
            match serde_json::from_slice::<LexiconDoc>(&json) {
                Ok(doc) => {
                    lexicon_docs.push(doc);
                }
                Err(err) => {
                    log::debug!("{:?}", err);
                    log::error!(
                        "There was an error in deserializing the ATProto record found {}. Is it a valid lexicon schema record?",
                        record.uri
                    );
                }
            }
        }
    }

    let out_dir = PathBuf::from(args.outdir.as_path());
    let results = gen_from_lexicon_docs(lexicon_docs, out_dir, &args.module)
        .map_err(|e| anyhow!(e.to_string()))?;
    for path in &results {
        log::info!(
            "{} ({} bytes)",
            path.as_ref().display(),
            fs::metadata(path.as_ref())?.len()
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(Generate { subcommand }) => match subcommand {
            GenerateCommands::Local(args) => local_generate_action(args),
            GenerateCommands::Remote(args) => generate_from_record_action(args).await,
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
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(self
            .resolver
            .txt_lookup(query)
            .await?
            .iter()
            .map(|txt| txt.to_string())
            .collect())
    }
}
