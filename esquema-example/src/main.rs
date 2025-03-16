use atrium_api::agent::atp_agent::AtpAgent;
use atrium_api::agent::atp_agent::store::MemorySessionStore;
use atrium_xrpc_client::reqwest::ReqwestClient;
use dotenv::dotenv;

mod lexicons;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let handle = std::env::var("BLUE_SKY_HANDLE").expect("BLUE_SKY_HANDLE must be set");
    let password = std::env::var("BLUE_SKY_PASSWORD").expect("BLUE_SKY_PASSWORD must be set");

    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    agent.login(handle, password).await?;
    Ok(())
}
