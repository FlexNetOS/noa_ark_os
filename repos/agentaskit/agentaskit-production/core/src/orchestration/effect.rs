use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use crate::ledger::Ledger;
use noa_abi::Permit;

pub trait Effect: Send + Sync + 'static {
    type Input: DeserializeOwned + Send + Sync;
    type Output: Serialize + Send + Sync;
    fn name(&self) -> &'static str;
    fn precheck(&self, input: &Self::Input, permit: &Permit, ledger: &dyn Ledger) -> Result<()>;
    fn apply(&self, input: Self::Input, ledger: &dyn Ledger) -> Result<Self::Output>;
}
