use crate::rpc::prelude::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetBlock {
    pub rollup_block_number: u64,
}

impl GetBlock {
    pub const METHOD_NAME: &'static str = stringify!(GetBlock);

    pub async fn handler(
        parameter: RpcParameter,
        _context: Arc<AppState>,
    ) -> Result<RollupBlock, RpcError> {
        let parameter = parameter.parse::<Self>()?;

        RollupBlock::get(parameter.rollup_block_number).map_err(|error| error.into())
    }
}
