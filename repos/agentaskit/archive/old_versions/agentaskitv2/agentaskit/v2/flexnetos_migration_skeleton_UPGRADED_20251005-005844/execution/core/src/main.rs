
use anyhow::Result;

#[tokio::main(flavor="multi_thread")]
async fn main() -> Result<()> {
    println!("flex-core starting (skeleton).");
    // TODO: verify SBOM/signatures; bind UDS; serve Cap'n Proto RPC.
    Ok(())
}
