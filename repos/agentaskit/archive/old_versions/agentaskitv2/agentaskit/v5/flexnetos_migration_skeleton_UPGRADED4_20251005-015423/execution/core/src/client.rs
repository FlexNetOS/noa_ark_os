use anyhow::Result;
use tokio::net::UnixStream;
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};

pub mod inference_capnp {
  include!(concat!(env!("OUT_DIR"), "/inference_capnp.rs"));
}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<()> {
    let sock_path = "/tmp/flex_infer.sock";
    let stream = UnixStream::connect(sock_path).await?;
    let (reader, writer) = tokio::io::split(stream);
    let network = twoparty::VatNetwork::new(reader, writer, rpc_twoparty_capnp::Side::Client, Default::default());
    let mut rpc_system = RpcSystem::new(Box::new(network), None);
    let client: inference_capnp::inference::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
    tokio::task::spawn_local(rpc_system);

    let mut req = client.predict_request();
    {
        let mut r = req.get().init_req();
        r.set_input(b"hello uds");
        r.set_trace_id("smoke-1");
    }
    let rep = req.send().promise.await?;
    let rep_reader = rep.get()?.get_rep()?;
    println!("client: model={} bytes={}", rep_reader.get_model_hash()?, rep_reader.get_output()?.len());
    Ok(())
}
