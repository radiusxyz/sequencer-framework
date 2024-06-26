use super::prelude::*;

pub async fn handler(parameter: RpcParameter, _context: Arc<()>) -> Result<(), RpcError> {
    let parameter = parameter.parse::<Register>()?;
    tracing::info!("{:?}", parameter);

    health_check(&parameter.sequencer_rpc_url).await?;

    database()?
        .put(&parameter.sequencer_address, &parameter.sequencer_rpc_url)
        .map_err(|error| error.into())
}
