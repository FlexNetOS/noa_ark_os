
use anyhow::Result;
use tokio::net::UnixListener;
use capnp::capability::Promise;
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};

pub mod inference_capnp {
  include!(concat!(env!("OUT_DIR"), "/inference_capnp.rs"));
}

use inference_capnp::inference;

#[derive(Default)]
struct InferenceImpl;

impl inference::Server for InferenceImpl {
  fn predict(
    &mut self,
    params: inference::PredictParams,
    mut results: inference::PredictResults,
  ) -> Promise<(), capnp::Error> {
    let req = pry!(params.get());
    let input = pry!(req.get_req()).get_input().unwrap_or_default();
    // Echo-style demo: output = input, modelHash = fixed placeholder.
    {
      let mut rep = results.get().init_rep();
      rep.set_output(input);
      rep.set_model_hash("demo-model-hash");
    }
    Promise::ok(())
  }
}

#[tokio::main(flavor="multi_thread")]
async fn main() -> Result<()> {
    let sock_path = "/tmp/flex_infer.sock";
    let _ = std::fs::remove_file(sock_path);
    let listener = UnixListener::bind(sock_path)?;
    println!("flex-core: UDS listening on {sock_path}");
    loop {
        let (stream, _) = listener.accept().await?;
        let (reader, writer) = tokio::io::split(stream);
        let rpc_network = twoparty::VatNetwork::new(
            reader, writer, rpc_twoparty_capnp::Side::Server, Default::default());
        let inference_impl = InferenceImpl::default();
        let client: inference::Client = capnp_rpc::new_client(inference_impl);
        let mut rpc_system = RpcSystem::new(Box::new(rpc_network), Some(client.client));
        tokio::spawn(async move {
            let _ = rpc_system.await;
        });
    }
}
