
use anyhow::{anyhow, Result};
use tokio::net::UnixListener;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, BufRead, BufReader};
use sha2::{Sha256, Digest};
use std::process::Command;
use capnp::capability::Promise;
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};

pub mod inference_capnp {
  include!(concat!(env!("OUT_DIR"), "/inference_capnp.rs"));
}
use inference_capnp::inference;

fn sha256_file(p: &Path) -> Result<String> {
    let mut f = File::open(p)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1<<20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn find_root() -> PathBuf {
    if let Ok(r) = std::env::var("FLEX_ROOT") { return PathBuf::from(r); }
    let mut cur = std::env::current_dir().unwrap();
    for _ in 0..5 {
        if cur.join("sbom").join("sbom.cdx.json").exists() { return cur; }
        if let Some(p) = cur.parent() { cur = p.to_path_buf(); } else { break; }
    }
    std::env::current_dir().unwrap()
}

fn verify_minisign(manifest: &Path) -> Result<()> {
    if let Ok(pubkey) = std::env::var("FLEX_MINISIGN_PUB") {
        if manifest.with_extension("sha256.minisig").exists() {
            let out = Command::new("minisign")
                .arg("-Vm").arg(manifest)
                .arg("-p").arg(pubkey)
                .output();
            if let Ok(o) = out {
                if o.status.success() {
                    println!("[verify] minisign OK");
                    return Ok(());
                } else {
                    return Err(anyhow!("[verify] minisign failed: {}", String::from_utf8_lossy(&o.stderr)));
                }
            }
        }
    }
    println!("[verify] minisign not enforced (no pubkey or signature).");
    Ok(())
}

fn verify_manifest(root: &Path) -> Result<()> {
    let manifest = root.join("artifacts").join("MANIFEST.sha256");
    if !manifest.exists() { return Err(anyhow!("manifest missing at {}", manifest.display())); }
    verify_minisign(&manifest)?;
    let file = File::open(&manifest)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line?;
        if l.trim().is_empty() { continue; }
        let (expected, name) = l.split_once("  ").ok_or_else(|| anyhow!("bad manifest line: {}", l))?;
        let p = root.join(name);
        if !p.exists() { return Err(anyhow!("manifest path missing: {}", p.display())); }
        let got = sha256_file(&p)?;
        if got != expected {
            return Err(anyhow!("hash mismatch: {} (expected {}, got {})", p.display(), expected, got));
        }
    }
    println!("[verify] manifest OK");
    Ok(())
}

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
    let root = find_root();
    verify_manifest(&root)?; // Load-time guard
    let sock_path = "/tmp/flex_infer.sock";
    let _ = std::fs::remove_file(sock_path);
    let listener = UnixListener::bind(sock_path)?;
    println!("flex-core: verified & listening on {sock_path}");
    loop {
        let (stream, _) = listener.accept().await?;
        let (reader, writer) = tokio::io::split(stream);
        let rpc_network = twoparty::VatNetwork::new(reader, writer, rpc_twoparty_capnp::Side::Server, Default::default());
        let inference_impl = InferenceImpl::default();
        let client: inference::Client = capnp_rpc::new_client(inference_impl);
        let mut rpc_system = RpcSystem::new(Box::new(rpc_network), Some(client.client));
        tokio::spawn(async move { let _ = rpc_system.await; });
    }
}
