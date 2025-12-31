use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use wasmtime::*;
use wasmtime_wasi::{WasiCtxBuilder, WasiCtx};
use serde::Deserialize;

#[derive(Deserialize)]
struct CapHeader { typ: String, alg: String }
#[derive(Deserialize)]
struct CapPayload { sub: String, aud: String, scopes: Vec<String>, iat: u64, exp: u64 }

fn parse_cap(env: &str, secret: &str) -> Result<CapPayload> {
    let mut parts = env.split('.');
    let h = parts.next().ok_or_else(|| anyhow!("bad cap header"))?;
    let p = parts.next().ok_or_else(|| anyhow!("bad cap payload"))?;
    let s = parts.next().ok_or_else(|| anyhow!("bad cap sig"))?;
    let msg = format!("{}.{}", h, p);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let sig = URL_SAFE_NO_PAD.decode(s)?;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(msg.as_bytes());
    mac.verify_slice(&sig).map_err(|_| anyhow!("cap token HMAC mismatch"))?;
    // decode payload
    let payload_json = String::from_utf8(URL_SAFE_NO_PAD.decode(p)?)?;
    let payload: CapPayload = serde_json::from_str(&payload_json)?;
    Ok(payload)
}

fn require_scope(payload: &CapPayload, need: &str) -> Result<()> {
    if payload.scopes.iter().any(|s| s == need) { Ok(()) } else { Err(anyhow!("missing required scope: {need}")) }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { return Err(anyhow!("usage: wasm_host <connector.wat|wasm>")); }
    let module_path = PathBuf::from(&args[1]);
    let name = module_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let cap = std::env::var("FLEX_CAP_TOKEN").map_err(|_| anyhow!("FLEX_CAP_TOKEN missing"))?;
    let secret = std::env::var("FLEX_CONNECTOR_SECRET").unwrap_or_else(|_| "changeme".to_string());
    let payload = parse_cap(&cap, &secret)?;
    require_scope(&payload, &format!("connector:{name}"))?;

    // WASI context with optional preopened dir at /cap
    let mut wasi_builder = WasiCtxBuilder::new().inherit_stdout().inherit_stderr();
    if let Ok(pre) = std::env::var("FLEX_PREOPEN_DIR") {
        use cap_std::{fs::Dir, ambient_authority};
        let dir = Dir::open_ambient_dir(pre, ambient_authority())?;
        wasi_builder = wasi_builder.preopened_dir(dir, "/cap");
    }
    let wasi: WasiCtx = wasi_builder.build();

    let engine = Engine::default();
    let module_bytes = if module_path.extension().and_then(|s| s.to_str()) == Some("wat") {
        wat::parse_file(&module_path)?
    } else { fs::read(&module_path)? };
    let module = Module::new(&engine, &module_bytes)?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasiCtx| s)?;

    let mut store = Store::new(&engine, wasi);
    let instance = linker.instantiate(&mut store, &module)?;
    if let Some(run) = instance.get_typed_func::<(), (), _>(&mut store, "run").ok() {
        run.call(&mut store, ())?;
    }
    println!("[wasm_host] ok: {}", module_path.display());
    Ok(())
}
