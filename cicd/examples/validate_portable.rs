use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let manifest = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "core/config/default_manifest.yaml".into()),
    );
    let bundle_dir = PathBuf::from(args.next().unwrap_or_else(|| "build/portable".into()));
    let format_arg = args.next().unwrap_or_else(|| "oci wasi tar".into());
    let formats = format_arg.split_whitespace().collect::<Vec<_>>();

    let manifest = noa_cicd::validation::validate_kernel_manifest(&manifest)?;
    noa_cicd::validation::verify_portable_bundle(&bundle_dir, &formats)?;

    println!(
        "Validated portable bundle for manifest version {}",
        manifest.version
    );
    Ok(())
}
