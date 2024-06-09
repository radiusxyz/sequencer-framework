use crate::rpc::{prelude::*, util::update_cluster_metadata};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncBuildBlock {
    pub ssal_block_number: SsalBlockNumber,
    pub rollup_block_number: RollupBlockNumber,
    pub previous_block_height: u64,
}

#[async_trait]
impl RpcMethod for SyncBuildBlock {
    type Response = ();

    fn method_name() -> &'static str {
        stringify!(SyncBuildBlock)
    }

    async fn handler(self) -> Result<Self::Response, RpcError> {
        match ClusterMetadata::get() {
            Ok(cluster_metadata) => {
                tracing::info!("{}: {:?}", Self::method_name(), self);
                update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                block_builder::init(
                    cluster_metadata.rollup_block_number(),
                    self.previous_block_height,
                    false,
                );
                Ok(())
            }
            Err(error) => {
                if error.kind() == database::ErrorKind::KeyDoesNotExist {
                    update_cluster_metadata(self.ssal_block_number, self.rollup_block_number)?;
                    Ok(())
                } else {
                    Err(error.into())
                }
            }
        }
    }
}
