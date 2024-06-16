mod public_key;
mod rpc_address;
mod prelude {
    pub use serde::{Deserialize, Serialize};

    pub use crate::ethereum::{Error, ErrorKind};
}

pub use public_key::*;
pub use rpc_address::*;
pub use signature::*;

ethers::contract::abigen!(Ssal, "src/ethereum/contract/Ssal.json");
