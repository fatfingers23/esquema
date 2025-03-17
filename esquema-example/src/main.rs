use crate::lexicons::record::KnownRecord;
use crate::lexicons::xyz;
use crate::lexicons::xyz::statusphere::status::RecordData;
use atrium_api::agent::atp_agent::AtpAgent;
use atrium_api::agent::atp_agent::store::MemorySessionStore;
use atrium_api::types::string::Datetime;
use atrium_api::types::{Collection, LimitedNonZeroU8};
use atrium_xrpc_client::reqwest::ReqwestClient;
use dotenv::dotenv;
use lexicons::xyz::statusphere::Status;

//TODO lock behind feature flags on which to use?
// mod lexicons;
mod lexicons {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let handle = std::env::var("BLUE_SKY_HANDLE").expect("BLUE_SKY_HANDLE must be set");
    let password = std::env::var("BLUE_SKY_PASSWORD").expect("BLUE_SKY_PASSWORD must be set");

    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    let session = agent.login(handle, password).await?;

    let status: KnownRecord = xyz::statusphere::status::RecordData {
        created_at: Datetime::now(),
        status: "🦀".to_string(),
    }
    .into();

    let create_result = agent
        .api
        .com
        .atproto
        .repo
        .create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: Status::NSID.parse()?,
                repo: atrium_api::types::string::AtIdentifier::Did(session.did.clone()),
                rkey: None,
                record: status.into(),
                swap_commit: None,
                validate: None,
            }
            .into(),
        )
        .await?;

    println!("{:?}", create_result.clone());

    let result = agent
        .api
        .com
        .atproto
        .repo
        .list_records(
            atrium_api::com::atproto::repo::list_records::ParametersData {
                collection: Status::NSID.parse()?,
                cursor: None,
                limit: Some(LimitedNonZeroU8::try_from(3u8)?),
                repo: atrium_api::types::string::AtIdentifier::Did(session.did.clone()),
                reverse: None,
                rkey_end: None,
                rkey_start: None,
            }
            .into(),
        )
        .await?;

    for record in result.records.clone() {
        let data: RecordData = record.value.clone().into();
        println!("record uri: {:?}", record.uri);
        println!(
            "atptools: {:?}",
            format!("https://atp.tools/{}", record.uri)
        );
        println!("status: {:?}", data.status);
        println!("created_at: {:?}\n", data.created_at);
    }

    Ok(())
}
