use std::time::Duration;

use reqwest::Client;

use crate::error::Error;

pub async fn health_check(sequencer_address: impl AsRef<str>) -> Result<(), Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| Error::HealthCheck)?;
    client
        .get(sequencer_address.as_ref())
        .send()
        .await
        .map_err(|_| Error::HealthCheck)?;
    Ok(())
}