use super::prelude::*;

pub async fn handler(
    parameter: RpcParameter,
    _context: Arc<()>,
) -> Result<Vec<Option<String>>, RpcError> {
    let parameter = parameter.parse::<GetSequencerRpcUrlList>()?;
    let database = database()?;

    let sequencer_list: Vec<Option<String>> = parameter
        .sequencer_address_list
        .iter()
        .map(|sequencer_public_key| database.get::<Address, String>(sequencer_public_key).ok())
        .collect();

    Ok(sequencer_list)
}
