pub use alloy::{
    primitives::Address,
    rpc::types::{Block, Log},
    sol,
    sol_types::SolEvent,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Ssal,
    "src/avs/contract/Ssal.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Avs,
    "src/avs/contract/Avs.json"
);

pub enum SsalEventType {
    NewBlock(Block),
    InitializeCluster((Ssal::InitializeClusterEvent, Log)),
    BlockCommitment((Avs::NewTaskCreated, Log)),
}
