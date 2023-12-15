use anyhow::Result;

use openid4vc::issuer::AttributeService;
use wallet_server::{pid::attributes::PidAttributeService, server, settings::Settings, store::new_session_store};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    let sessions = new_session_store(settings.store_url.clone()).await?;
    let attr_service = PidAttributeService::new(&settings.issuer.digid).await?;

    // This will block until the server shuts down.
    server::serve(&settings, sessions, attr_service).await?;

    Ok(())
}
